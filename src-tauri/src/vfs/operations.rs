use super::*;
impl VirtualFileSystem {
    pub(crate) fn restore_entry(&mut self, path: &str, node: Option<VfsNode>) {
        if let Some(mut node) = node {
            if node.ino == 0 {
                // Legacy journals have no identity: never invent sharing.
                self.register_identity(&node.owner);
                self.register_identity(&node.group);
                node.uid = self.identity(&node.owner).uid;
                node.gid = self.identity(&node.group).gid;
                node.accessed_at = node.modified_at;
                node.changed_at = node.modified_at;
            } else {
                self.nodes.replace_inode(node.inode.clone());
            }
            self.nodes.insert(path.into(), node);
        } else {
            self.nodes.remove(path);
        }
    }

    pub fn unlink(&mut self, path: &str, actor: &str) -> GameResult<()> {
        if self.lstat(path, actor)?.kind == "directory" {
            return Err(error(Errno::IsDirectory));
        }
        self.remove(path, actor, false)
    }
    pub fn rmdir(&mut self, path: &str, actor: &str) -> GameResult<()> {
        if self.lstat(path, actor)?.kind != "directory" {
            return Err(error(Errno::NotDirectory));
        }
        self.remove(path, actor, false)
    }
    pub fn remove(&mut self, path: &str, actor: &str, recursive: bool) -> GameResult<()> {
        let path = self.resolve(path, actor, Follow::No)?;
        if path == "/" {
            return Err(error(Errno::NotPermitted));
        }
        self.check_projection(&path)?;
        self.writable_parent(&path, actor)?;
        self.sticky(&path, actor)?;
        let prefix = format!("{path}/");
        let children: Vec<_> = self
            .nodes
            .range(prefix.clone()..)
            .take_while(|(p, _)| p.starts_with(&prefix))
            .map(|(p, _)| p.clone())
            .collect();
        if !recursive && !children.is_empty() {
            return Err(error(Errno::NotEmpty));
        }
        for p in &children {
            self.writable_parent(p, actor)?;
            self.sticky(p, actor)?;
        }
        let clock = self.tick();
        for p in children
            .into_iter()
            .rev()
            .chain(std::iter::once(path.clone()))
        {
            if let Some(mut n) = self.nodes.get_mut(&p) {
                n.changed_at = clock;
            }
            self.nodes.remove(&p);
        }
        self.changed_parent(&path);
        self.collect();
        Ok(())
    }
    pub fn rename_entry(&mut self, source: &str, target: &str, actor: &str) -> GameResult<()> {
        let source = self.resolve(source, actor, Follow::No)?;
        let target = self.resolve_missing(target, actor, Follow::No, true)?;
        if source == "/" || target == "/" || target.starts_with(&format!("{source}/")) {
            return Err(error(Errno::Invalid));
        }
        if source == target {
            return Ok(());
        }
        let original = self.lstat(&source, actor)?;
        if self
            .nodes
            .get(&target)
            .is_some_and(|n| n.ino == original.ino)
        {
            return Ok(());
        }
        self.check_projection(&source)?;
        self.check_projection(&target)?;
        self.writable_parent(&source, actor)?;
        self.writable_parent(&target, actor)?;
        self.sticky(&source, actor)?;
        if let Some(existing) = self.nodes.get(&target) {
            self.sticky(&target, actor)?;
            if original.kind == "directory" && existing.kind != "directory" {
                return Err(error(Errno::NotDirectory));
            }
            if original.kind != "directory" && existing.kind == "directory" {
                return Err(error(Errno::IsDirectory));
            }
            if existing.kind == "directory"
                && self
                    .nodes
                    .directories
                    .get(&existing.ino)
                    .is_some_and(|d| !d.is_empty())
            {
                return Err(error(Errno::NotEmpty));
            }
        }
        let prefix = format!("{source}/");
        let entries: Vec<_> = self
            .nodes
            .iter()
            .filter(|(p, _)| **p == source || p.starts_with(&prefix))
            .map(|(p, n)| (p.clone(), n.clone()))
            .collect();
        self.nodes.remove(&target);
        for (old, _) in entries.iter().rev() {
            self.nodes.remove(old);
        }
        for (old, node) in entries {
            let path = format!("{target}{}", &old[source.len()..]);
            self.nodes.insert(path, node);
        }
        let clock = self.tick();
        if let Some(mut node) = self.nodes.get_mut(&target) {
            node.changed_at = clock;
        }
        self.changed_parent(&source);
        self.changed_parent(&target);
        self.collect();
        Ok(())
    }
    pub(crate) fn copy_entry(&mut self, source: &str, target: &str, actor: &str) -> GameResult<()> {
        let original = self.lstat(source, actor)?.clone();
        if original.kind == "file" {
            self.readable(source, actor)?;
        } else if original.kind != "symlink" {
            return Err(error(Errno::Invalid));
        }
        let target = self.resolve_missing(target, actor, Follow::No, true)?;
        if self
            .nodes
            .get(&target)
            .is_some_and(|n| n.ino == original.ino)
        {
            return Err(domain("source and destination are the same file"));
        }
        self.check_projection(&target)?;
        self.check_space(&target, original.logical_size())?;
        if original.kind == "symlink" {
            if self.nodes.contains_key(&target) {
                self.unlink(&target, actor)?;
            }
            self.symlink(&target, &original.content, actor)?;
            return Ok(());
        }
        let existing = self.nodes.contains_key(&target);
        if let Some(blob) = &original.blob {
            self.write_blob(&target, blob.clone(), actor)?;
        } else {
            self.write(&target, &original.content, actor)?;
        }
        let path = self.resolve(&target, actor, Follow::Yes)?;
        if let Some(mut node) = self.nodes.get_mut(&path) {
            node.metadata = original.metadata.clone();
            node.metadata.remove("packageProjection");
            node.metadata.remove("packageOwner");
            if !existing {
                node.mode = original.mode & !self.umask;
            }
        }
        Ok(())
    }
    pub fn transfer(
        &mut self,
        source: &str,
        destination: &str,
        actor: &str,
        moving: bool,
        recursive: bool,
    ) -> GameResult<()> {
        let source = self.resolve(source, actor, Follow::No)?;
        let original = self.lstat(&source, actor)?.clone();
        let target = if let Ok(n) = self.stat(destination, actor) {
            if n.kind == "directory" {
                format!("{}/{}", n.id.trim_end_matches('/'), original.name)
            } else {
                destination.into()
            }
        } else {
            destination.into()
        };
        let target = self.resolve_missing(&target, actor, Follow::No, true)?;
        if source == "/" || source == target || target.starts_with(&format!("{source}/")) {
            return Err(error(Errno::Invalid));
        }
        if self.nodes.contains_key(&target) {
            return Err(error(Errno::Exists));
        }
        if moving {
            return self.rename_entry(&source, &target, actor);
        }
        if original.kind == "directory" && !recursive {
            return Err(domain("omitting directory; use -r"));
        }
        let prefix = format!("{source}/");
        let entries: Vec<_> = self
            .nodes
            .iter()
            .filter(|(p, _)| **p == source || p.starts_with(&prefix))
            .map(|(p, n)| (p.clone(), n.kind.clone()))
            .collect();
        if self.nodes.len() + entries.len() > 10000 {
            return Err(error(Errno::NoSpace));
        }
        let mut candidate = self.clone();
        for (old, kind) in entries {
            let path = format!("{target}{}", &old[source.len()..]);
            if kind == "directory" {
                candidate.directory(&old, actor)?;
                candidate.child_names(&old, actor)?;
                candidate.mkdir(&path, actor)?;
            } else {
                candidate.copy_entry(&old, &path, actor)?;
            }
        }
        *self = candidate;
        Ok(())
    }
    pub fn touch(&mut self, path: &str, actor: &str) -> GameResult<()> {
        if !self.path_exists(path, actor)? {
            self.write(path, "", actor)?;
            return Ok(());
        }
        let path = self.resolve(path, actor, Follow::Yes)?;
        self.check_projection(&path)?;
        let n = self.stat(&path, actor)?;
        if actor != "root" && self.identity(actor).uid != n.uid && !self.allowed(n, actor, 2) {
            return Err(error(Errno::Access));
        }
        let clock = self.tick();
        if let Some(mut n) = self.nodes.get_mut(&path) {
            n.accessed_at = clock;
            n.modified_at = clock;
            n.changed_at = clock;
        }
        Ok(())
    }
    pub fn check_invariants(&self) -> GameResult<()> {
        if self.nodes.get("/").is_none_or(|n| n.kind != "directory") {
            return Err(domain("VFS invariant: root missing"));
        }
        let mut counts = BTreeMap::<u64, u64>::new();
        for (path, node) in &self.nodes {
            resolve::validate_path(path)?;
            if !path.starts_with('/')
                || path != "/"
                    && path
                        .split('/')
                        .skip(1)
                        .any(|s| s.is_empty() || s == "." || s == "..")
                || node.id != *path
            {
                return Err(domain("VFS invariant: invalid directory entry"));
            }
            if node.mode > 0o7777
                || !["file", "directory", "symlink", "charDevice"].contains(&node.kind.as_str())
            {
                return Err(domain("VFS invariant: unsupported kind or mode"));
            }
            if node.kind == "symlink" {
                resolve::validate_path(&node.content)?;
            }
            if node.kind != "file" && node.blob.is_some() {
                return Err(domain("VFS invariant: invalid blob kind"));
            }
            if node.ino == 0
                || self
                    .nodes
                    .inodes
                    .get(&node.ino)
                    .is_none_or(|n| !Arc::ptr_eq(n, &node.inode))
            {
                return Err(domain("VFS invariant: inode projection mismatch"));
            }
            if path != "/" {
                let p = self
                    .nodes
                    .get(parent(path))
                    .ok_or_else(|| domain("VFS invariant: missing parent"))?;
                if p.kind != "directory"
                    || self
                        .nodes
                        .directories
                        .get(&p.ino)
                        .and_then(|d| d.get(&node.name))
                        != Some(&node.ino)
                {
                    return Err(domain("VFS invariant: invalid parent entry"));
                }
            }
            *counts.entry(node.ino).or_default() += 1;
        }
        for (ino, node) in &self.nodes.inodes {
            let count = counts.get(ino).copied().unwrap_or(0);
            let nlink = if node.kind == "directory" && count > 0 {
                if count != 1 {
                    return Err(domain("VFS invariant: directory hardlink"));
                }
                2 + self
                    .nodes
                    .directories
                    .get(ino)
                    .into_iter()
                    .flat_map(|d| d.values())
                    .filter(|id| {
                        self.nodes
                            .inodes
                            .get(id)
                            .is_some_and(|n| n.kind == "directory")
                    })
                    .count() as u64
            } else {
                count
            };
            if nlink != node.nlink || count == 0 && !self.handles.values().any(|h| h.ino == *ino) {
                return Err(domain(format!("VFS invariant: orphan or link count (ino={ino}, entries={count}, expected={nlink}, actual={})",node.nlink)));
            }
            if *ino >= self.nodes.next_inode {
                return Err(domain("VFS invariant: allocator collision"));
            }
        }
        Ok(())
    }
}
