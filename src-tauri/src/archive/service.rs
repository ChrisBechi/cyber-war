use super::*;
use crate::{
    binary::BlobRef,
    vfs::{normalize, parent, VfsNode, VirtualFileSystem},
    world::WorldState,
};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveEvent {
    pub kind: String,
    pub path: String,
    pub destination: Option<String>,
    pub entries: Vec<String>,
    pub format: Option<ArchiveFormat>,
}
pub fn event(
    world: &mut WorldState,
    kind: &str,
    path: &str,
    destination: Option<&str>,
    entries: Vec<String>,
    format: Option<ArchiveFormat>,
) {
    world.archive_events.push(ArchiveEvent {
        kind: kind.into(),
        path: path.into(),
        destination: destination.map(String::from),
        entries,
        format,
    });
    if world.archive_events.len() > 256 {
        world.archive_events.remove(0);
    }
}
pub fn record_failure(
    world: &mut WorldState,
    path: &str,
    error: &str,
    format: Option<ArchiveFormat>,
) {
    let kind = if error.contains("password") {
        "ARCHIVE_PASSWORD_FAILED"
    } else if [
        "invalid header",
        "unsupported format",
        "truncated",
        "checksum",
        "CRC",
        "corrupt",
        "decompression",
        "central directory",
    ]
    .iter()
    .any(|s| error.contains(s))
    {
        "ARCHIVE_CORRUPTED"
    } else {
        return;
    };
    let path = normalize(path, &world.terminal.cwd).unwrap_or_else(|_| path.into());
    event(world, kind, &path, None, Vec::new(), format);
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveInspection {
    pub path: String,
    pub format: ArchiveFormat,
    pub integrity: ArchiveIntegrity,
    pub encrypted: bool,
    pub compressed_size: u64,
    pub storage_size: u64,
    pub original_size: u64,
    pub modified_at: u64,
    pub created_at: u64,
    pub entries: Vec<ArchiveEntry>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum Overwrite {
    #[default]
    Ask,
    Replace,
    Skip,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractOptions {
    pub destination: String,
    #[serde(default)]
    pub overwrite: Overwrite,
    #[serde(default)]
    pub selected: Vec<String>,
    #[serde(default)]
    pub decisions: BTreeMap<String, bool>,
}

#[derive(Default)]
pub struct ArchiveService {
    pub limits: ArchiveSafetyLimits,
}

pub fn bytes(world: &WorldState, path: &str, actor: &str) -> GameResult<Arc<Vec<u8>>> {
    let node = world.fs()?.readable(path, actor)?;
    if let Some(blob) = &node.blob {
        world
            .blobs
            .get(&blob.hash)
            .cloned()
            .ok_or_else(|| domain("archive: binary payload unavailable"))
    } else {
        Ok(Arc::new(node.content.as_bytes().to_vec()))
    }
}
pub fn write_bytes(
    world: &mut WorldState,
    path: &str,
    data: Vec<u8>,
    actor: &str,
    logical: u64,
) -> GameResult<()> {
    let mime = detect(&data)
        .map(ArchiveFormat::mime)
        .unwrap_or("application/octet-stream");
    let reference = BlobRef {
        hash: format!("{:x}", Sha256::digest(&data)),
        size: data.len(),
        mime: mime.into(),
    };
    world
        .fs()?
        .check_space(path, logical.max(data.len() as u64))?;
    world.fs_mut()?.write_blob(path, reference.clone(), actor)?;
    world
        .fs_mut()?
        .nodes
        .get_mut(path)
        .unwrap()
        .metadata
        .insert(
            "logicalSize".into(),
            logical.max(data.len() as u64).to_string(),
        );
    world.blobs.insert(reference.hash, Arc::new(data));
    Ok(())
}

impl ArchiveService {
    fn collect(
        &self,
        world: &WorldState,
        cwd: &str,
        inputs: &[String],
        recursive: bool,
        actor: &str,
        output: &str,
    ) -> GameResult<Vec<StoredEntry>> {
        let mut pending = Vec::new();
        for input in inputs {
            let p = normalize(input, cwd)?;
            if p == cwd {
                for child in world.fs()?.list(cwd, actor)? {
                    pending.push((child.id, child.name));
                }
                continue;
            }
            let relative = if input.starts_with('/') {
                p.trim_start_matches('/').to_string()
            } else {
                p.strip_prefix(&format!("{}/", cwd.trim_end_matches('/')))
                    .ok_or_else(|| domain("archive: source must be inside the working directory"))?
                    .to_string()
            };
            pending.push((p, relative));
        }
        let mut entries = BTreeMap::new();
        let mut storage = 0u64;
        while let Some((p, relative)) = pending.pop() {
            if p == output {
                continue;
            }
            let relative = self.limits.path(&relative)?;
            let n = world.fs()?.stat(&p, actor)?;
            let kind = n.kind.clone();
            if entries.contains_key(&relative) {
                continue;
            }
            if entries.len() >= self.limits.max_entries {
                return Err(domain("archive: entry count limit exceeded"));
            }
            let data = if kind == "file" {
                bytes(world, &p, actor)?.as_ref().clone()
            } else {
                Vec::new()
            };
            storage += data.len() as u64;
            if storage > self.limits.max_storage_size {
                return Err(domain("archive: storage size limit exceeded"));
            }
            let metadata = EntryMetadata {
                media_source: n.metadata.get("mediaSource").cloned(),
                logical_size: n.logical_size(),
                modified_at: n.modified_at,
                owner: n.owner.clone(),
                group: n.group.clone(),
                mime: n
                    .blob
                    .as_ref()
                    .map(|b| b.mime.clone())
                    .or_else(|| n.metadata.get("mime").cloned()),
                archive_depth: n
                    .metadata
                    .get("archiveDepth")
                    .and_then(|n| n.parse().ok())
                    .unwrap_or(0),
            };
            let info = ArchiveEntry {
                path: relative.clone(),
                kind: kind.clone(),
                original_size: metadata.logical_size,
                compressed_size: None,
                permissions: n.mode & 0o777,
                modified_at: n.modified_at,
                owner: Some(n.owner.clone()),
                group: Some(n.group.clone()),
                crc: None,
                link_target: (kind == "symlink").then(|| n.content.clone()),
                encrypted: false,
            };
            entries.insert(
                relative.clone(),
                StoredEntry {
                    info,
                    data,
                    metadata: Some(metadata),
                },
            );
            if kind == "directory" {
                let children = world.fs()?.list(&p, actor)?;
                if recursive {
                    for child in children {
                        pending.push((child.id, format!("{relative}/{}", child.name)));
                    }
                }
            }
        }
        let entries: Vec<_> = entries.into_values().collect();
        if entries.is_empty() {
            return Err(domain("archive: nothing to do"));
        }
        self.limits
            .entries(&entries, self.limits.max_storage_size)?;
        Ok(entries)
    }
    #[allow(clippy::too_many_arguments)] // Explicit VFS actor and per-operation codec options.
    pub fn create_archive(
        &self,
        world: &mut WorldState,
        path: &str,
        inputs: &[String],
        format: ArchiveFormat,
        recursive: bool,
        password: Option<&str>,
        actor: &str,
    ) -> GameResult<ArchiveInspection> {
        let path = normalize(path, &world.terminal.cwd)?;
        let mut entries =
            self.collect(world, &world.terminal.cwd, inputs, recursive, actor, &path)?;
        if format == ArchiveFormat::Zip && world.fs()?.nodes.contains_key(&path) {
            let old = bytes(world, &path, actor)?;
            let previous = codec::decode(&old, format, password, true, &self.limits)?;
            if previous.iter().any(|e| e.info.encrypted) && password.is_none() {
                return Err(domain("archive: password required"));
            }
            let mut merged: BTreeMap<_, _> = previous
                .into_iter()
                .map(|e| (e.info.path.clone(), e))
                .collect();
            for e in entries {
                merged.insert(e.info.path.clone(), e);
            }
            entries = merged.into_values().collect();
        }
        let original: u64 = entries.iter().map(|e| e.info.original_size).sum();
        let data = codec::encode(&entries, format, password, &self.limits)?;
        let storage = data.len() as u64;
        let logical = storage.saturating_add(
            entries
                .iter()
                .map(|e| {
                    let sparse = e.info.original_size.saturating_sub(e.data.len() as u64);
                    if format == ArchiveFormat::Tar {
                        sparse
                    } else {
                        profile(&e.info.path).compressed_size(sparse)
                    }
                })
                .sum::<u64>(),
        );
        let mut candidate = world.clone();
        write_bytes(&mut candidate, &path, data, actor, logical)?;
        let node = candidate.fs_mut()?.nodes.get_mut(&path).unwrap();
        node.metadata
            .insert("archiveOriginalSize".into(), original.to_string());
        let result = self.inspect_archive(&candidate, &path, actor)?;
        event(
            &mut candidate,
            "ARCHIVE_CREATED",
            &path,
            None,
            entries.iter().map(|e| e.info.path.clone()).collect(),
            Some(format),
        );
        jobs::check_cancel()?;
        *world = candidate;
        Ok(result)
    }
    pub fn inspect_archive(
        &self,
        world: &WorldState,
        path: &str,
        actor: &str,
    ) -> GameResult<ArchiveInspection> {
        let path = normalize(path, &world.terminal.cwd)?;
        let data = bytes(world, &path, actor)?;
        let mut format =
            detect(&data).ok_or_else(|| domain("archive: unsupported format or invalid header"))?;
        let entries = if format.is_stream() {
            // Read only enough to classify the inner stream before choosing the index parser.
            let prefix = codec::read_bounded(codec::decoder(&data, format)?.take(512), 512)?;
            if detect(&prefix) == Some(ArchiveFormat::Tar) {
                format = match format {
                    ArchiveFormat::Gzip => ArchiveFormat::TarGzip,
                    ArchiveFormat::Bzip2 => ArchiveFormat::TarBzip2,
                    _ => ArchiveFormat::TarXz,
                };
                codec::decode(&data, format, None, false, &self.limits)?
            } else {
                Vec::new()
            }
        } else {
            codec::decode(&data, format, None, false, &self.limits)?
        };
        let node = world.fs()?.stat(&path, actor)?;
        let encrypted = entries.iter().any(|e| e.info.encrypted);
        Ok(ArchiveInspection {
            path,
            format,
            integrity: if encrypted {
                ArchiveIntegrity::Encrypted
            } else {
                ArchiveIntegrity::Partial
            },
            encrypted,
            compressed_size: node.logical_size(),
            storage_size: data.len() as u64,
            original_size: node
                .metadata
                .get("archiveOriginalSize")
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(|| entries.iter().map(|e| e.info.original_size).sum()),
            modified_at: node.modified_at,
            created_at: node.created_at,
            entries: entries.into_iter().map(|e| e.info).collect(),
        })
    }
    pub fn list_archive(
        &self,
        world: &WorldState,
        path: &str,
        actor: &str,
    ) -> GameResult<Vec<ArchiveEntry>> {
        Ok(self.inspect_archive(world, path, actor)?.entries)
    }
    pub fn test_archive(
        &self,
        world: &WorldState,
        path: &str,
        password: Option<&str>,
        actor: &str,
    ) -> GameResult<Vec<ArchiveEntry>> {
        let info = self.inspect_archive(world, path, actor)?;
        let data = bytes(world, &info.path, actor)?;
        if info.format.is_stream() {
            codec::read_bounded(
                codec::decoder(&data, info.format)?,
                self.stream_limit(data.len()),
            )?;
            Ok(Vec::new())
        } else {
            Ok(
                codec::decode(&data, info.format, password, true, &self.limits)?
                    .into_iter()
                    .map(|e| e.info)
                    .collect(),
            )
        }
    }
    pub fn conflicts(
        &self,
        world: &WorldState,
        path: &str,
        options: &ExtractOptions,
        actor: &str,
    ) -> GameResult<Vec<String>> {
        let dest = normalize(&options.destination, &world.terminal.cwd)?;
        Ok(self
            .list_archive(world, path, actor)?
            .into_iter()
            .filter(|e| {
                selected(&e.path, &options.selected)
                    && e.kind != "directory"
                    && world.fs().is_ok_and(|f| {
                        f.nodes.contains_key(&format!(
                            "{}/{p}",
                            dest.trim_end_matches('/'),
                            p = e.path
                        ))
                    })
            })
            .map(|e| e.path)
            .filter(|p| !options.decisions.contains_key(p))
            .collect())
    }
    pub fn extract_archive(
        &self,
        world: &mut WorldState,
        path: &str,
        options: &ExtractOptions,
        password: Option<&str>,
        actor: &str,
    ) -> GameResult<Vec<String>> {
        let info = self.inspect_archive(world, path, actor)?;
        if info.format.is_stream() {
            return Err(domain(
                "archive: decompress this single stream with gunzip/bunzip2/unxz",
            ));
        }
        let dest = normalize(&options.destination, &world.terminal.cwd)?;
        let data = bytes(world, &info.path, actor)?;
        let entries = codec::decode(&data, info.format, password, true, &self.limits)?;
        let depth: u32 = world
            .fs()?
            .stat(&info.path, actor)?
            .metadata
            .get("archiveDepth")
            .and_then(|n| n.parse().ok())
            .unwrap_or(0);
        if depth >= 8 {
            return Err(domain("archive: nested archive depth limit exceeded"));
        }
        let mut candidate = world.clone();
        mkdirs(candidate.fs_mut()?, &dest, actor)?;
        let mut written = Vec::new();
        let mut directories = Vec::new();
        let media_reference = regex::Regex::new(
            r"^/assets/[a-zA-Z0-9_/-]+\.(png|jpe?g|webp|gif|ogg|mp3|wav|mp4|webm)$",
        )
        .unwrap();
        for entry in &entries {
            if !selected(&entry.info.path, &options.selected) {
                continue;
            }
            let output = normalize(&entry.info.path, &dest)?;
            if !output.starts_with(&format!("{}/", dest.trim_end_matches('/'))) {
                return Err(domain("archive: entry escapes destination"));
            }
            if output == info.path {
                return Err(domain("archive: cannot overwrite the source archive"));
            }
            mkdirs(candidate.fs_mut()?, parent(&output), actor)?;
            let e = &entry.info;
            if e.kind == "directory" {
                let existing = candidate.fs()?.nodes.contains_key(&output);
                mkdirs(candidate.fs_mut()?, &output, actor)?;
                // Merging an archive must not chmod somebody else's directory.
                if !existing
                    || actor == "root"
                    || candidate.fs()?.stat(&output, actor)?.owner == actor
                {
                    directories.push((output, e));
                }
                continue;
            }
            let exists = candidate.fs()?.nodes.contains_key(&output);
            if exists {
                match options
                    .decisions
                    .get(&e.path)
                    .copied()
                    .or(match options.overwrite {
                        Overwrite::Replace => Some(true),
                        Overwrite::Skip => Some(false),
                        Overwrite::Ask => None,
                    }) {
                    Some(false) => continue,
                    None => return Err(domain(format!("archive: overwrite required: {}", e.path))),
                    Some(true) => {
                        if candidate.fs()?.stat(&output, actor)?.kind != "file" {
                            return Err(domain(
                                "archive: refusing to replace directory or symlink",
                            ));
                        }
                    }
                }
            }
            candidate.fs()?.check_space(&output, e.original_size)?;
            if e.kind == "symlink" {
                if exists {
                    candidate.fs_mut()?.remove(&output, actor, false)?;
                }
                candidate.fs_mut()?.symlink(
                    &output,
                    e.link_target.as_deref().unwrap_or(""),
                    actor,
                )?;
            } else if let Ok(text) = std::str::from_utf8(&entry.data) {
                if text.len() <= 1_048_576
                    && detect(&entry.data).is_none()
                    && entry
                        .metadata
                        .as_ref()
                        .and_then(|m| m.mime.as_ref())
                        .is_none_or(|_| {
                            entry
                                .metadata
                                .as_ref()
                                .is_some_and(|m| m.media_source.is_some())
                        })
                {
                    candidate.fs_mut()?.write(&output, text, actor)?;
                } else {
                    write_bytes(
                        &mut candidate,
                        &output,
                        entry.data.clone(),
                        actor,
                        e.original_size,
                    )?;
                }
            } else {
                write_bytes(
                    &mut candidate,
                    &output,
                    entry.data.clone(),
                    actor,
                    e.original_size,
                )?;
            }
            let n = candidate.fs_mut()?.nodes.get_mut(&output).unwrap();
            restore_metadata(n, e, actor);
            n.metadata
                .insert("logicalSize".into(), e.original_size.to_string());
            n.metadata
                .insert("archiveDepth".into(), (depth + 1).to_string());
            if let Some(mime) = entry.metadata.as_ref().and_then(|m| m.mime.as_ref()) {
                n.metadata.insert("mime".into(), mime.clone());
                if let Some(b) = &mut n.blob {
                    b.mime = mime.clone();
                    b.validate()?;
                }
            }
            if let Some(source) = entry
                .metadata
                .as_ref()
                .and_then(|m| m.media_source.as_ref())
            {
                // Bundled content references use the same passive asset allowlist
                // as the media viewer; archive metadata cannot introduce URLs.
                if !media_reference.is_match(source) {
                    return Err(domain("archive: invalid bundled media reference"));
                }
                n.metadata.insert("mediaSource".into(), source.clone());
            }
            written.push(e.path.clone());
        }
        for (path, e) in directories.into_iter().rev() {
            restore_metadata(candidate.fs_mut()?.nodes.get_mut(&path).unwrap(), e, actor);
        }
        if info.encrypted {
            event(
                &mut candidate,
                "ARCHIVE_PASSWORD_SUCCESS",
                &info.path,
                None,
                Vec::new(),
                Some(info.format),
            );
        }
        event(
            &mut candidate,
            "ARCHIVE_EXTRACTED",
            &info.path,
            Some(&dest),
            written.clone(),
            Some(info.format),
        );
        jobs::check_cancel()?;
        *world = candidate;
        Ok(written)
    }
    fn stream_limit(&self, compressed: usize) -> u64 {
        self.limits.max_storage_size.min(
            (compressed as u64)
                .max(1)
                .saturating_mul(self.limits.max_compression_ratio),
        )
    }
    #[allow(clippy::too_many_arguments)] // Mirrors the implemented stream command options.
    pub fn compress_file(
        &self,
        world: &mut WorldState,
        input: &str,
        format: ArchiveFormat,
        keep: bool,
        force: bool,
        stdout: bool,
        actor: &str,
    ) -> GameResult<Vec<u8>> {
        if !format.is_stream() {
            return Err(domain("archive: expected stream compression"));
        }
        let input = normalize(input, &world.terminal.cwd)?;
        let node = world.fs()?.readable(&input, actor)?.clone();
        let data = bytes(world, &input, actor)?;
        let data = if node.logical_size() > data.len() as u64 {
            virtual_stream(&node, &data)?
        } else {
            data.as_ref().clone()
        };
        let compressed = codec::compress(&data, format)?;
        if stdout {
            return Ok(compressed);
        }
        let output = format!("{input}{}", format.suffix());
        if world.fs()?.nodes.contains_key(&output) && !force {
            return Err(domain("archive: output already exists (use -f)"));
        }
        let mut candidate = world.clone();
        let logical = compressed.len() as u64
            + profile(&input)
                .compressed_size(node.logical_size().saturating_sub(node.storage_size()));
        write_bytes(&mut candidate, &output, compressed, actor, logical)?;
        if !keep {
            candidate.fs_mut()?.remove(&input, actor, false)?;
        }
        event(
            &mut candidate,
            "ARCHIVE_CREATED",
            &output,
            None,
            vec![input],
            Some(format),
        );
        jobs::check_cancel()?;
        *world = candidate;
        Ok(Vec::new())
    }
    #[allow(clippy::too_many_arguments)]
    pub fn decompress_file(
        &self,
        world: &mut WorldState,
        input: &str,
        format: ArchiveFormat,
        keep: bool,
        force: bool,
        stdout: bool,
        actor: &str,
    ) -> GameResult<Vec<u8>> {
        self.decompress_file_to(world, input, format, keep, force, stdout, actor, None)
    }
    #[allow(clippy::too_many_arguments)]
    pub fn decompress_file_to(
        &self,
        world: &mut WorldState,
        input: &str,
        format: ArchiveFormat,
        keep: bool,
        force: bool,
        stdout: bool,
        actor: &str,
        destination: Option<&str>,
    ) -> GameResult<Vec<u8>> {
        let input = normalize(input, &world.terminal.cwd)?;
        let data = bytes(world, &input, actor)?;
        if detect(&data) != Some(format) {
            return Err(domain(format!("archive: not in {:?} format", format)));
        }
        let decoded = codec::read_bounded(
            codec::decoder(&data, format)?,
            self.stream_limit(data.len()),
        )?;
        let (decoded, logical) = unpack_virtual_stream(decoded)?;
        if stdout {
            if logical > decoded.len() as u64 {
                return Err(domain(
                    "archive: sparse virtual content cannot be materialized on stdout",
                ));
            }
            event(
                world,
                "ARCHIVE_ENTRY_READ",
                &input,
                None,
                Vec::new(),
                Some(format),
            );
            return Ok(decoded);
        }
        let default_output = decompressed_name(&input, format)?;
        let output = if let Some(destination) = destination {
            normalize(default_output.rsplit('/').next().unwrap(), destination)?
        } else {
            default_output
        };
        if output == input {
            return Err(domain("archive: cannot overwrite the source stream"));
        }
        if world.fs()?.nodes.contains_key(&output) && !force {
            return Err(domain("archive: output already exists (use -f)"));
        }
        let mut candidate = world.clone();
        mkdirs(candidate.fs_mut()?, parent(&output), actor)?;
        candidate.fs()?.check_space(&output, logical)?;
        match std::str::from_utf8(&decoded) {
            Ok(text) if text.len() <= 1_048_576 && detect(&decoded).is_none() => {
                candidate.fs_mut()?.write(&output, text, actor)?
            }
            _ => write_bytes(&mut candidate, &output, decoded, actor, logical)?,
        }
        candidate
            .fs_mut()?
            .nodes
            .get_mut(&output)
            .unwrap()
            .metadata
            .insert("logicalSize".into(), logical.to_string());
        if !keep {
            candidate.fs_mut()?.remove(&input, actor, false)?;
        }
        event(
            &mut candidate,
            "ARCHIVE_EXTRACTED",
            &input,
            Some(&output),
            vec![output.clone()],
            Some(format),
        );
        jobs::check_cancel()?;
        *world = candidate;
        Ok(Vec::new())
    }
}

use std::io::Read;
fn selected(path: &str, selected: &[String]) -> bool {
    selected.is_empty()
        || selected
            .iter()
            .any(|p| path == p || path.starts_with(&format!("{p}/")))
}
fn mkdirs(fs: &mut VirtualFileSystem, path: &str, actor: &str) -> GameResult<()> {
    let mut p = String::new();
    for part in path.split('/').filter(|s| !s.is_empty()) {
        p.push('/');
        p.push_str(part);
        if fs.nodes.contains_key(&p) {
            fs.directory(&p, actor)?;
        } else {
            fs.mkdir(&p, actor)?;
        }
    }
    Ok(())
}
fn restore_metadata(n: &mut VfsNode, e: &ArchiveEntry, actor: &str) {
    n.mode = e.permissions & 0o777;
    n.modified_at = e.modified_at;
    if actor == "root" {
        if let Some(o) = &e.owner {
            n.owner = o.clone();
        }
        if let Some(g) = &e.group {
            n.group = g.clone();
        }
    }
}
pub fn decompressed_name(input: &str, format: ArchiveFormat) -> GameResult<String> {
    for suffix in match format {
        ArchiveFormat::Gzip => &[".tar.gz", ".tgz"][..],
        ArchiveFormat::Bzip2 => &[".tar.bz2", ".tbz2"][..],
        _ => &[".tar.xz", ".txz"][..],
    } {
        if let Some(base) = input.strip_suffix(suffix) {
            return Ok(format!("{base}.tar"));
        }
    }
    input
        .strip_suffix(format.suffix())
        .map(String::from)
        .ok_or_else(|| domain("archive: unknown suffix; use -c to read stdout"))
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CompressionProfile {
    Text,
    SourceCode,
    Log,
    Database,
    Image,
    Video,
    Executable,
    AlreadyCompressed,
    RandomData,
}
impl CompressionProfile {
    fn compressed_size(self, size: u64) -> u64 {
        size.saturating_mul(match self {
            Self::Text => 35,
            Self::SourceCode => 30,
            Self::Log => 15,
            Self::Database => 20,
            Self::Image | Self::Video => 98,
            Self::Executable => 65,
            Self::AlreadyCompressed | Self::RandomData => 100,
        }) / 100
    }
}
fn profile(path: &str) -> CompressionProfile {
    match path.rsplit('.').next().unwrap_or("") {
        "log" => CompressionProfile::Log,
        "sql" | "db" => CompressionProfile::Database,
        "rs" | "py" | "js" | "ts" | "sh" => CompressionProfile::SourceCode,
        "jpg" | "jpeg" | "png" => CompressionProfile::Image,
        "mp4" | "webm" => CompressionProfile::Video,
        "exe" | "bin" => CompressionProfile::Executable,
        "gz" | "zip" | "xz" | "bz2" => CompressionProfile::AlreadyCompressed,
        _ => CompressionProfile::Text,
    }
}
const STREAM_MAGIC: &[u8] = b"CYBERWAR-SPARSE-V1\0";
fn virtual_stream(node: &VfsNode, data: &[u8]) -> GameResult<Vec<u8>> {
    let mut out = STREAM_MAGIC.to_vec();
    out.extend_from_slice(&node.logical_size().to_le_bytes());
    out.extend_from_slice(data);
    Ok(out)
}
fn unpack_virtual_stream(data: Vec<u8>) -> GameResult<(Vec<u8>, u64)> {
    if data.starts_with(STREAM_MAGIC) {
        let header = STREAM_MAGIC.len();
        let size = u64::from_le_bytes(
            data.get(header..header + 8)
                .ok_or_else(|| domain("archive: invalid sparse header"))?
                .try_into()
                .unwrap(),
        );
        if size > ArchiveSafetyLimits::default().max_uncompressed_size
            || size < (data.len() - header - 8) as u64
        {
            return Err(domain("archive: sparse size limit exceeded"));
        }
        Ok((data[header + 8..].to_vec(), size))
    } else {
        let size = data.len() as u64;
        Ok((data, size))
    }
}
