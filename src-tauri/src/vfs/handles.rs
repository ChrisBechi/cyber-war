use super::*;
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OpenFlags {
    pub read: bool,
    pub write: bool,
    pub append: bool,
    pub create: bool,
    pub truncate: bool,
    pub exclusive: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Handle {
    pub ino: u64,
    pub offset: usize,
    pub flags: OpenFlags,
    pub device: Option<String>,
    pub privileged: bool,
}
impl VirtualFileSystem {
    pub fn open(
        &mut self,
        path: &str,
        flags: OpenFlags,
        mode: u16,
        actor: &str,
    ) -> GameResult<u64> {
        if (!flags.read && !flags.write) || (flags.truncate || flags.append) && !flags.write {
            return Err(error(Errno::Invalid));
        }
        if self.handles.len() >= 1024 {
            return Err(error(Errno::NoSpace));
        }
        if flags.create && flags.exclusive && self.path_exists(path, actor)? {
            return Err(error(Errno::Exists));
        }
        let resolved = self.resolve_missing(path, actor, Follow::Yes, flags.create)?;
        let created = !self.nodes.contains_key(&resolved);
        if created {
            self.create(&resolved, "file", "", actor, mode)?;
        }
        let node = self.stat(&resolved, actor)?;
        if node.kind == "directory" {
            return Err(error(Errno::IsDirectory));
        }
        if flags.write {
            self.check_projection(&resolved)?;
        }
        if !created
            && !self.allowed(
                node,
                actor,
                if flags.read { 4 } else { 0 } | if flags.write { 2 } else { 0 },
            )
        {
            return Err(error(Errno::Access));
        }
        if flags.truncate && !created {
            self.write(&resolved, "", actor)?;
        }
        let node = self.stat(&resolved, actor)?;
        let handle = Handle {
            ino: node.ino,
            offset: 0,
            flags,
            device: (node.kind == "charDevice")
                .then(|| node.metadata.get("device").cloned().unwrap_or_default()),
            privileged: self.identity(actor).uid == 0,
        };
        let id = self.next_handle;
        self.next_handle += 1;
        self.handles.insert(id, handle);
        Ok(id)
    }
    pub fn read_handle(&mut self, id: u64, count: usize) -> GameResult<Vec<u8>> {
        self.read_handle_bytes(id, count, &crate::binary::BlobCache::new())
    }
    pub fn read_handle_bytes(
        &mut self,
        id: u64,
        count: usize,
        blobs: &crate::binary::BlobCache,
    ) -> GameResult<Vec<u8>> {
        let h = self
            .handles
            .get_mut(&id)
            .ok_or_else(|| error(Errno::BadDescriptor))?;
        if !h.flags.read {
            return Err(error(Errno::BadDescriptor));
        }
        let count = count.min(crate::binary::MAX_BLOB);
        if let Some(device) = &h.device {
            return match device.as_str() {
                "null" => Ok(Vec::new()),
                "zero" => Ok(vec![0; count]),
                _ => Err(error(Errno::Invalid)),
            };
        }
        let n = self
            .nodes
            .inodes
            .get(&h.ino)
            .ok_or_else(|| error(Errno::BadDescriptor))?;
        let data = if let Some(blob) = &n.blob {
            blobs
                .get(&blob.hash)
                .ok_or_else(|| domain("binary payload unavailable"))?
                .as_slice()
        } else {
            n.content.as_bytes()
        };
        if h.offset >= data.len() {
            return Ok(Vec::new());
        }
        let start = h.offset;
        let end = start.saturating_add(count).min(data.len());
        h.offset = end;
        let bytes = data[start..end].to_vec();
        let ino = h.ino;
        if !bytes.is_empty() && !self.read_only {
            let clock = self.tick();
            let mut inode = self.nodes.inodes[&ino].clone();
            Arc::make_mut(&mut inode).accessed_at = clock;
            self.nodes.replace_inode(inode);
        }
        Ok(bytes)
    }
    pub fn write_handle(&mut self, id: u64, bytes: &[u8]) -> GameResult<usize> {
        std::str::from_utf8(bytes)
            .map_err(|_| domain("virtual text write requires valid UTF-8"))?;
        // This compatibility API has no persistent blob cache. Reject writes
        // that would require a blob before changing the inode or handle offset.
        let handle = self
            .handles
            .get(&id)
            .ok_or_else(|| error(Errno::BadDescriptor))?;
        if handle.device.is_none() && !bytes.is_empty() {
            let node = self
                .nodes
                .inodes
                .get(&handle.ino)
                .ok_or_else(|| error(Errno::BadDescriptor))?;
            if node.blob.is_some() {
                return Err(domain("binary writes require a persistent blob cache"));
            }
            let start = if handle.flags.append {
                node.content.len()
            } else {
                handle.offset
            };
            let end = start
                .checked_add(bytes.len())
                .ok_or_else(|| error(Errno::NoSpace))?;
            if end > MAX_CONTENT {
                return Err(domain("virtual text file limit: 1 MiB"));
            }
            if !node.content.is_char_boundary(start.min(node.content.len()))
                || !node.content.is_char_boundary(end.min(node.content.len()))
            {
                return Err(domain("virtual text write splits UTF-8"));
            }
        }
        self.write_handle_bytes(id, bytes, &mut crate::binary::BlobCache::new())
    }
    pub fn write_handle_bytes(
        &mut self,
        id: u64,
        bytes: &[u8],
        blobs: &mut crate::binary::BlobCache,
    ) -> GameResult<usize> {
        use sha2::{Digest, Sha256};
        if self.read_only {
            return Err(error(Errno::ReadOnly));
        }
        let h = self
            .handles
            .get(&id)
            .ok_or_else(|| error(Errno::BadDescriptor))?
            .clone();
        if !h.flags.write {
            return Err(error(Errno::BadDescriptor));
        }
        if bytes.is_empty() {
            return Ok(0);
        }
        if let Some(device) = &h.device {
            return if device == "null" || device == "zero" {
                Ok(bytes.len())
            } else {
                Err(error(Errno::Invalid))
            };
        }
        let mut n = self
            .nodes
            .inodes
            .get(&h.ino)
            .ok_or_else(|| error(Errno::BadDescriptor))?
            .as_ref()
            .clone();
        let mut data = if let Some(blob) = &n.blob {
            blobs
                .get(&blob.hash)
                .ok_or_else(|| domain("binary payload unavailable"))?
                .as_ref()
                .clone()
        } else {
            n.content.as_bytes().to_vec()
        };
        let start = if h.flags.append { data.len() } else { h.offset };
        let end = start
            .checked_add(bytes.len())
            .ok_or_else(|| error(Errno::NoSpace))?;
        if end > crate::binary::MAX_BLOB {
            return Err(domain("virtual byte file limit: 32 MiB"));
        }
        let old = n.logical_size();
        data.resize(data.len().max(end), 0);
        data[start..end].copy_from_slice(bytes);
        let payload_size = data.len();
        let blob_data = if data.len() <= MAX_CONTENT && std::str::from_utf8(&data).is_ok() {
            n.content = String::from_utf8(data).expect("validated UTF-8");
            n.blob = None;
            None
        } else {
            let hash = format!("{:x}", Sha256::digest(&data));
            n.content.clear();
            n.blob = Some(crate::binary::BlobRef {
                hash: hash.clone(),
                size: data.len(),
                mime: "application/octet-stream".into(),
            });
            Some((hash, Arc::new(data)))
        };
        if self
            .used_bytes()
            .saturating_sub(old)
            .saturating_add(payload_size as u64)
            > self.capacity_bytes
        {
            return Err(error(Errno::NoSpace));
        }
        let clock = self.tick();
        n.modified_at = clock;
        n.changed_at = clock;
        if !h.privileged {
            n.mode &= !0o6000;
        }
        for key in [
            "mediaSource",
            "mime",
            "logicalSize",
            "archiveOriginalSize",
            "archiveDepth",
        ] {
            n.metadata.remove(key);
        }
        self.nodes.replace_inode(Arc::new(n));
        if let Some((hash, data)) = blob_data {
            blobs.insert(hash, data);
        }
        if let Some(handle) = self.handles.get_mut(&id) {
            handle.offset = end;
        }
        Ok(bytes.len())
    }
    pub fn seek(&mut self, id: u64, offset: usize) -> GameResult<()> {
        if offset > MAX_CONTENT {
            return Err(error(Errno::Invalid));
        }
        self.handles
            .get_mut(&id)
            .ok_or_else(|| error(Errno::BadDescriptor))?
            .offset = offset;
        Ok(())
    }
    pub fn close(&mut self, id: u64) -> GameResult<()> {
        self.handles
            .remove(&id)
            .ok_or_else(|| error(Errno::BadDescriptor))?;
        self.collect();
        Ok(())
    }
    pub fn truncate(&mut self, path: &str, size: usize, actor: &str) -> GameResult<()> {
        if size > MAX_CONTENT {
            return Err(domain("virtual file limit: 1 MiB"));
        }
        let path = self.resolve(path, actor, Follow::Yes)?;
        self.check_projection(&path)?;
        let n = self.stat(&path, actor)?;
        if n.kind != "file" {
            return Err(error(Errno::Invalid));
        }
        if !self.allowed(n, actor, 2) {
            return Err(error(Errno::Access));
        }
        if n.blob.is_some() {
            return Err(domain("binary truncate is outside the virtual text subset"));
        }
        let mut data = n.content.as_bytes().to_vec();
        data.resize(size, 0);
        let text =
            String::from_utf8(data).map_err(|_| domain("virtual text truncation splits UTF-8"))?;
        self.write(&path, &text, actor)
    }
}
