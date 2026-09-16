use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::error::{GameError, GameResult};

pub const HOME: &str = "/home/kali";
pub const TRASH_ROOT: &str = "/home/kali/.local/share/Trash";
pub const TRASH_FILES: &str = "/home/kali/.local/share/Trash/files";
pub const TRASH_INFO: &str = "/home/kali/.local/share/Trash/info";
const MAX_CONTENT: usize = 1_048_576;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VfsNode {
    pub id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub kind: String,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blob: Option<crate::binary::BlobRef>,
    pub owner: String,
    pub group: String,
    pub mode: u16,
    pub created_at: u64,
    pub modified_at: u64,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VirtualFileSystem {
    pub nodes: BTreeMap<String, VfsNode>,
    clock: u64,
    #[serde(default = "default_disk_capacity")]
    pub capacity_bytes: u64,
}

fn default_disk_capacity() -> u64 {
    64 * 1024 * 1024 * 1024
}
impl VfsNode {
    pub fn storage_size(&self) -> u64 {
        self.blob
            .as_ref()
            .map_or(self.content.len() as u64, |b| b.size as u64)
    }
    pub fn logical_size(&self) -> u64 {
        if self.kind == "directory" {
            return 0;
        }
        self.metadata
            .get("logicalSize")
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or_else(|| self.storage_size())
            .max(self.storage_size())
    }
}

pub fn domain(message: impl Into<String>) -> GameError {
    GameError::Domain(message.into())
}

pub fn normalize(path: &str, cwd: &str) -> GameResult<String> {
    if path.len() > 4096 || path.contains(['\\', ':']) || path.chars().any(char::is_control) {
        return Err(domain("invalid virtual path"));
    }
    // Tilde expansion belongs to the shell, where quote information exists.
    let expanded = if path.starts_with('/') {
        path.to_owned()
    } else {
        format!("{cwd}/{path}")
    };
    let mut parts = Vec::new();
    for part in expanded.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            part => parts.push(part),
        }
    }
    Ok(format!("/{}", parts.join("/")))
}

pub fn parent(path: &str) -> &str {
    path.rsplit_once('/')
        .map(|(p, _)| if p.is_empty() { "/" } else { p })
        .unwrap_or("/")
}

impl Default for VirtualFileSystem {
    fn default() -> Self {
        let mut fs = Self {
            nodes: BTreeMap::new(),
            clock: 0,
            capacity_bytes: default_disk_capacity(),
        };
        for path in [
            "/", "/bin", "/boot", "/dev", "/etc", "/home", HOME, "/opt", "/root", "/tmp", "/usr",
            "/var", "/var/log", "/srv", "/srv/www",
        ] {
            fs.seed(
                path,
                "directory",
                "",
                if path == HOME { "kali" } else { "root" },
            );
        }
        for name in [
            "Desktop",
            "Documents",
            "Downloads",
            "Music",
            "Pictures",
            "Videos",
            "projects",
            "tools",
        ] {
            fs.seed(&format!("{HOME}/{name}"), "directory", "", "kali");
        }
        for path in [
            "/home/kali/.local",
            "/home/kali/.local/share",
            TRASH_ROOT,
            TRASH_FILES,
            TRASH_INFO,
        ] {
            fs.seed(path, "directory", "", "kali");
        }
        if let Some(n) = fs.nodes.get_mut("/root") {
            n.mode = 0o700;
        }
        if let Some(n) = fs.nodes.get_mut("/tmp") {
            n.mode = 0o777;
        }
        fs.seed(
            "/etc/os-release",
            "file",
            "NAME=LifeOS\nVERSION=1.0\n",
            "root",
        );
        fs.seed("/home/kali/Documents/primeiros-passos.txt", "file", "Bem-vindo ao LifeOS.\nExplore seus arquivos com pwd, ls e cd.\nCrie uma nota: echo \"Finalmente.\" > Documents/notes.txt\nAbra o mensageiro: Gregory está por aqui.\n", "kali");
        for (path, mime, source) in [
            (
                "/home/kali/Music/cyber-war-opening.ogg",
                "audio/ogg",
                "/assets/audio/cyber-war-opening-music.ogg",
            ),
            (
                "/home/kali/Videos/cyber-war-opening.webm",
                "video/webm",
                "/assets/video/cyber-war-opening.webm",
            ),
            (
                "/home/kali/Pictures/kali-waves.png",
                "image/png",
                "/assets/kali-waves.png",
            ),
            (
                "/home/kali/Pictures/kali-maze.jpg",
                "image/jpeg",
                "/assets/kali-maze.jpg",
            ),
            (
                "/home/kali/Pictures/kali-cubes2.jpg",
                "image/jpeg",
                "/assets/kali-cubes2.jpg",
            ),
        ] {
            fs.seed(path, "file", "", "kali");
            if let Some(node) = fs.nodes.get_mut(path) {
                node.metadata.insert("mime".into(), mime.into());
                node.metadata.insert("mediaSource".into(), source.into());
            }
        }
        fs.seed(
            "/home/kali/Videos/cyber-war-opening.vtt",
            "file",
            "WEBVTT\n\n00:00:00.000 --> 00:00:04.000\nCYBER WAR\n\n00:00:04.000 --> 00:00:08.000\nA virtual investigation begins.\n",
            "kali",
        );
        if let Some(node) = fs.nodes.get_mut("/home/kali/Videos/cyber-war-opening.vtt") {
            node.metadata.insert("mime".into(), "text/vtt".into());
        }
        for (name, app) in [
            ("Terminal", "terminal"),
            ("Arquivos", "files"),
            ("Navegador", "browser"),
            ("Mensagens", "messages"),
            ("HackPad", "editor"),
            ("CodeLab", "codelab"),
        ] {
            let path = format!("{HOME}/Desktop/{name}.desktop");
            fs.seed(&path, "file", app, "kali");
            if let Some(n) = fs.nodes.get_mut(&path) {
                n.metadata.insert("app".into(), app.into());
            }
        }
        fs
    }
}

impl VirtualFileSystem {
    fn check_projection(&self, path: &str) -> GameResult<()> {
        let prefix = format!("{path}/");
        if self
            .nodes
            .get(path)
            .is_some_and(|n| n.metadata.contains_key("packageProjection"))
            || self
                .nodes
                .range(prefix.clone()..)
                .take_while(|(p, _)| p.starts_with(&prefix))
                .any(|(_, n)| n.metadata.contains_key("packageProjection"))
        {
            return Err(domain(
                "package database projection is read-only; use apt/dpkg",
            ));
        }
        Ok(())
    }
    pub fn used_bytes(&self) -> u64 {
        self.nodes
            .values()
            .fold(0u64, |n, e| n.saturating_add(e.logical_size()))
    }
    pub fn check_space(&self, path: &str, size: u64) -> GameResult<()> {
        let old = self.nodes.get(path).map_or(0, VfsNode::logical_size);
        if self.used_bytes().saturating_sub(old).saturating_add(size) > self.capacity_bytes {
            return Err(domain("No space left on virtual device"));
        }
        Ok(())
    }
    pub fn symlink(&mut self, path: &str, target: &str, actor: &str) -> GameResult<()> {
        crate::archive::ArchiveSafetyLimits::default().path(target)?;
        if self.nodes.contains_key(path) {
            return Err(domain("file exists"));
        }
        self.write(path, target, actor)?;
        self.nodes.get_mut(path).unwrap().kind = "symlink".into();
        Ok(())
    }
    pub fn ensure_trash(&mut self) {
        for path in [
            "/home/kali/.local",
            "/home/kali/.local/share",
            TRASH_ROOT,
            TRASH_FILES,
            TRASH_INFO,
        ] {
            if !self.nodes.contains_key(path) {
                self.seed(path, "directory", "", "kali");
            }
        }
    }

    /// Optimistic editor save: stale buffers cannot overwrite terminal changes.
    pub fn write_checked(
        &mut self,
        path: &str,
        content: &str,
        expected: Option<&str>,
        actor: &str,
    ) -> GameResult<()> {
        match (self.nodes.get(path), expected) {
            (Some(_), Some(previous)) if self.read(path,actor)? == previous => {},
            (None, None) => {},
            _ => return Err(domain("Arquivo mudou desde a abertura. Reabra antes de salvar; seu texto foi preservado no editor.")),
        }
        self.write(path, content, actor)
    }
    pub fn seed(&mut self, path: &str, kind: &str, content: &str, owner: &str) {
        self.clock += 1;
        self.nodes.insert(
            path.into(),
            VfsNode {
                id: path.into(),
                parent_id: (path != "/").then(|| parent(path).into()),
                name: path.rsplit('/').next().unwrap_or("").into(),
                kind: kind.into(),
                content: content.into(),
                blob: None,
                owner: owner.into(),
                group: owner.into(),
                mode: if kind == "directory" { 0o755 } else { 0o644 },
                created_at: self.clock,
                modified_at: self.clock,
                metadata: BTreeMap::new(),
            },
        );
    }

    fn allowed(node: &VfsNode, actor: &str, bits: u16) -> bool {
        actor == "root"
            || ((node.mode
                >> if actor == node.owner {
                    6
                } else if actor == node.group {
                    3
                } else {
                    0
                })
                & bits)
                == bits
    }

    pub fn stat(&self, path: &str, actor: &str) -> GameResult<&VfsNode> {
        let mut ancestor = parent(path);
        loop {
            let n = self
                .nodes
                .get(ancestor)
                .ok_or_else(|| domain("parent does not exist"))?;
            if n.kind != "directory" {
                return Err(domain("not a directory (symlink traversal is not allowed)"));
            }
            if !Self::allowed(n, actor, 1) {
                return Err(domain("permission denied"));
            }
            if ancestor == "/" {
                break;
            }
            ancestor = parent(ancestor);
        }
        self.nodes
            .get(path)
            .ok_or_else(|| domain(format!("{path}: no such file or directory")))
    }

    pub fn directory(&self, path: &str, actor: &str) -> GameResult<()> {
        let n = self.stat(path, actor)?;
        if n.kind != "directory" {
            return Err(domain("not a directory"));
        }
        if !Self::allowed(n, actor, 1) {
            return Err(domain("permission denied"));
        }
        Ok(())
    }

    /// Existence with traversal checks from root, so an absent deeper ancestor
    /// cannot hide a permission error (notably rm -f /root/missing/child).
    pub fn path_exists(&self, path: &str, actor: &str) -> GameResult<bool> {
        self.directory("/", actor)?;
        let mut current = String::new();
        for component in parent(path).split('/').filter(|part| !part.is_empty()) {
            current.push('/');
            current.push_str(component);
            if !self.nodes.contains_key(&current) {
                return Ok(false);
            }
            self.directory(&current, actor)?;
        }
        Ok(self.nodes.contains_key(path))
    }

    /// Names only, in byte order; shell globbing must not clone file bodies.
    pub fn child_names(&self, path: &str, actor: &str) -> GameResult<Vec<String>> {
        self.directory(path, actor)?;
        if !Self::allowed(self.stat(path, actor)?, actor, 4) {
            return Err(domain("permission denied"));
        }
        let prefix = format!("{}/", path.trim_end_matches('/'));
        Ok(self
            .nodes
            .range(prefix.clone()..)
            .take_while(|(key, _)| key.starts_with(&prefix))
            .filter(|(_, node)| node.parent_id.as_deref() == Some(path))
            .map(|(key, _)| key[prefix.len()..].to_string())
            .collect())
    }

    pub fn list(&self, path: &str, actor: &str) -> GameResult<Vec<VfsNode>> {
        self.directory(path, actor)?;
        if !Self::allowed(self.stat(path, actor)?, actor, 4) {
            return Err(domain("permission denied"));
        }
        Ok(self
            .nodes
            .values()
            .filter(|n| n.parent_id.as_deref() == Some(path))
            .cloned()
            .collect())
    }

    pub fn read(&self, path: &str, actor: &str) -> GameResult<String> {
        let n = self.readable(path, actor)?;
        if n.blob.is_some() {
            return Err(domain("binary file: text access is not supported"));
        }
        Ok(n.content.clone())
    }

    pub fn readable(&self, path: &str, actor: &str) -> GameResult<&VfsNode> {
        let n = self.stat(path, actor)?;
        if n.kind != "file" {
            return Err(domain("is a directory"));
        }
        if !Self::allowed(n, actor, 4) {
            return Err(domain("permission denied"));
        }
        Ok(n)
    }

    fn writable_parent(&self, path: &str, actor: &str) -> GameResult<()> {
        self.directory(parent(path), actor)?;
        if !Self::allowed(self.stat(parent(path), actor)?, actor, 3) {
            return Err(domain("permission denied"));
        }
        Ok(())
    }

    /// Open a virtual shell output target before running the command. Append
    /// validates write permission without requiring read permission or touching mtime.
    pub fn prepare_output(&mut self, path: &str, actor: &str, append: bool) -> GameResult<()> {
        if self.nodes.contains_key(path) {
            let node = self.stat(path, actor)?;
            if node.kind != "file" {
                return Err(domain("is a directory"));
            }
            if !Self::allowed(node, actor, 2) {
                return Err(domain("permission denied"));
            }
            if append {
                if node.blob.is_some() {
                    return Err(domain("cannot append text to a binary file"));
                }
                return Ok(());
            }
        }
        self.write(path, "", actor)
    }

    pub fn write(&mut self, path: &str, content: &str, actor: &str) -> GameResult<()> {
        self.check_projection(path)?;
        self.check_space(path, content.len() as u64)?;
        if content.len() > MAX_CONTENT {
            return Err(domain("virtual file limit: 1 MiB"));
        }
        if self.nodes.contains_key(path) {
            let n = self.stat(path, actor)?;
            if n.kind != "file" {
                return Err(domain("is a directory"));
            }
            if !Self::allowed(n, actor, 2) {
                return Err(domain("permission denied"));
            }
            self.clock += 1;
            if let Some(n) = self.nodes.get_mut(path) {
                n.content = content.into();
                n.blob = None;
                n.metadata.remove("mediaSource");
                n.metadata.remove("mime");
                n.metadata.remove("logicalSize");
                n.metadata.remove("archiveOriginalSize");
                n.metadata.remove("archiveDepth");
                n.modified_at = self.clock;
            }
        } else {
            self.writable_parent(path, actor)?;
            if self.nodes.len() >= 10000 {
                return Err(domain("virtual filesystem capacity reached"));
            }
            self.seed(path, "file", content, actor);
        }
        Ok(())
    }

    pub fn write_blob(
        &mut self,
        path: &str,
        blob: crate::binary::BlobRef,
        actor: &str,
    ) -> GameResult<()> {
        blob.validate()?;
        self.check_space(path, blob.size as u64)?;
        self.write(path, "", actor)?;
        if let Some(node) = self.nodes.get_mut(path) {
            node.metadata.insert("mime".into(), blob.mime.clone());
            node.blob = Some(blob);
        }
        Ok(())
    }

    pub fn mkdir(&mut self, path: &str, actor: &str) -> GameResult<()> {
        if self.nodes.len() >= 10000 {
            return Err(domain("virtual filesystem capacity reached"));
        }
        if self.nodes.contains_key(path) {
            return Err(domain("file exists"));
        }
        self.writable_parent(path, actor)?;
        self.seed(path, "directory", "", actor);
        Ok(())
    }

    pub fn remove(&mut self, path: &str, actor: &str, recursive: bool) -> GameResult<()> {
        self.check_projection(path)?;
        if path == "/" {
            return Err(domain("cannot remove virtual root"));
        }
        self.stat(path, actor)?;
        self.writable_parent(path, actor)?;
        let prefix = format!("{path}/");
        if !recursive && self.nodes.keys().any(|p| p.starts_with(&prefix)) {
            return Err(domain("directory not empty; use -r"));
        }
        self.nodes
            .retain(|p, _| p != path && !p.starts_with(&prefix));
        Ok(())
    }

    pub fn is_trash_path(path: &str) -> bool {
        path == TRASH_ROOT
            || path == TRASH_FILES
            || path == TRASH_INFO
            || path.starts_with(&format!("{TRASH_FILES}/"))
            || path.starts_with(&format!("{TRASH_INFO}/"))
    }

    pub fn is_trash_container(path: &str) -> bool {
        matches!(
            path,
            "/home/kali/.local" | "/home/kali/.local/share" | TRASH_ROOT | TRASH_FILES | TRASH_INFO
        )
    }

    /// Move a user item to the virtual trash, preserving its original path in metadata.
    pub fn move_to_trash(&mut self, path: &str, actor: &str, recursive: bool) -> GameResult<()> {
        self.ensure_trash();
        if path == "/"
            || Self::is_trash_path(path)
            || path == "/home/kali/.local"
            || path == "/home/kali/.local/share"
        {
            return Err(domain(
                "não é possível enviar a lixeira ou a raiz para a lixeira",
            ));
        }
        let source = self.stat(path, actor)?.clone();
        if source.kind == "directory" && !recursive {
            return Err(domain("omitting directory; use -r"));
        }
        let mut target = format!("{TRASH_FILES}/{}", source.name);
        if self.nodes.contains_key(&target) {
            let (stem, extension) = source
                .name
                .rsplit_once('.')
                .filter(|(stem, _)| !stem.is_empty())
                .map(|(stem, ext)| (stem.to_owned(), format!(".{ext}")))
                .unwrap_or_else(|| (source.name.clone(), String::new()));
            let mut index = 1;
            loop {
                target = format!("{TRASH_FILES}/{stem} ({index}){extension}");
                if !self.nodes.contains_key(&target) {
                    break;
                }
                index += 1;
            }
        }
        self.transfer(path, &target, actor, true, recursive)?;
        if let Some(node) = self.nodes.get_mut(&target) {
            node.metadata
                .insert("trashOriginalPath".into(), path.into());
            node.metadata
                .insert("trashDeletedAt".into(), self.clock.to_string());
        }
        Ok(())
    }

    /// Restore a direct child of the trash to the path recorded when it was deleted.
    pub fn restore_from_trash(&mut self, path: &str, actor: &str) -> GameResult<()> {
        self.ensure_trash();
        if !path.starts_with(&format!("{TRASH_FILES}/"))
            || path[TRASH_FILES.len() + 1..].contains('/')
        {
            return Err(domain("selecione um item diretamente dentro da lixeira"));
        }
        let original = self
            .stat(path, actor)?
            .metadata
            .get("trashOriginalPath")
            .cloned()
            .ok_or_else(|| domain("item da lixeira sem caminho original"))?;
        if Self::is_trash_path(&original) || original == "/" {
            return Err(domain("caminho original inválido"));
        }
        if self.nodes.contains_key(&original) {
            return Err(domain("o caminho original já está ocupado"));
        }
        self.writable_parent(&original, actor)?;
        self.transfer(path, &original, actor, true, true)?;
        if let Some(node) = self.nodes.get_mut(&original) {
            node.metadata.remove("trashOriginalPath");
            node.metadata.remove("trashDeletedAt");
        }
        Ok(())
    }

    pub fn empty_trash(&mut self, actor: &str) -> GameResult<()> {
        self.ensure_trash();
        let entries: Vec<String> = self
            .nodes
            .values()
            .filter(|node| node.parent_id.as_deref() == Some(TRASH_FILES))
            .map(|node| node.id.clone())
            .collect();
        for path in entries {
            self.remove(&path, actor, true)?;
        }
        Ok(())
    }

    /// Pick an unused sibling name, preserving normal and compound extensions.
    pub fn available_path(&self, path: &str, directory: bool) -> String {
        if !self.nodes.contains_key(path) {
            return path.into();
        }
        let name = path.rsplit('/').next().unwrap_or(path);
        let prefix = &path[..path.len() - name.len()];
        let extension_at = if directory {
            name.len()
        } else {
            [".tar.gz", ".tar.bz2", ".tar.xz"]
                .iter()
                .find(|suffix| name.to_ascii_lowercase().ends_with(**suffix))
                .map(|suffix| name.len() - suffix.len())
                .unwrap_or_else(|| {
                    name.rfind('.')
                        .filter(|&index| index > 0)
                        .unwrap_or(name.len())
                })
        };
        let (stem, extension) = name.split_at(extension_at);
        let stem = stem
            .rsplit_once(" (")
            .filter(|(_, count)| {
                count.strip_suffix(')').is_some_and(|count| {
                    !count.is_empty() && count.bytes().all(|byte| byte.is_ascii_digit())
                })
            })
            .map(|(base, _)| base)
            .unwrap_or(stem);
        for count in 1..=self.nodes.len() + 1 {
            let candidate = format!("{prefix}{stem} ({count}){extension}");
            if !self.nodes.contains_key(&candidate) {
                return candidate;
            }
        }
        unreachable!("there are more candidate names than existing nodes")
    }

    pub fn copy_unique(&mut self, source: &str, destination: &str, actor: &str) -> GameResult<()> {
        let node = self.stat(source, actor)?;
        let target = if self
            .nodes
            .get(destination)
            .is_some_and(|node| node.kind == "directory")
        {
            format!("{}/{}", destination.trim_end_matches('/'), node.name)
        } else {
            destination.into()
        };
        if target.starts_with(&format!("{source}/")) || source == "/" {
            return Err(domain("invalid copy/move destination"));
        }
        let target = self.available_path(&target, node.kind == "directory");
        self.transfer(source, &target, actor, false, true)
    }

    /// Copy one regular file or link to an exact path. CLI directory traversal
    /// lives above this layer; GUI collision naming keeps using copy_unique.
    pub(crate) fn copy_entry(&mut self, source: &str, target: &str, actor: &str) -> GameResult<()> {
        let original = self.stat(source, actor)?.clone();
        if original.kind == "file" {
            self.readable(source, actor)?;
        } else if original.kind != "symlink" {
            return Err(domain("unsupported file type"));
        }
        self.check_projection(target)?;
        let existing = self.nodes.get(target).cloned();
        if let Some(existing) = &existing {
            self.stat(target, actor)?;
            if existing.kind == "directory" || (original.kind == "file" && existing.kind != "file")
            {
                return Err(domain("destination is not a regular file"));
            }
            if original.kind == "symlink" {
                self.writable_parent(target, actor)?;
            } else if !Self::allowed(existing, actor, 2) {
                return Err(domain("permission denied"));
            }
        } else {
            self.writable_parent(target, actor)?;
            if self.nodes.len() >= 10000 {
                return Err(domain("virtual filesystem capacity reached"));
            }
        }
        self.check_space(target, original.logical_size())?;
        self.clock += 1;
        let mut copied = original;
        let existing = existing.filter(|_| copied.kind == "file");
        copied.id = target.into();
        copied.parent_id = Some(parent(target).into());
        copied.name = target.rsplit('/').next().unwrap_or("").into();
        copied.owner = existing
            .as_ref()
            .map_or_else(|| actor.into(), |n| n.owner.clone());
        copied.group = existing
            .as_ref()
            .map_or_else(|| actor.into(), |n| n.group.clone());
        copied.mode = existing.as_ref().map_or(copied.mode & !0o022, |n| n.mode);
        copied.created_at = existing.as_ref().map_or(self.clock, |n| n.created_at);
        copied.modified_at = self.clock;
        copied.metadata.remove("packageProjection");
        copied.metadata.remove("packageOwner");
        self.nodes.insert(target.into(), copied);
        Ok(())
    }

    /// Rename within one virtual filesystem. Like rename(2), permissions are
    /// checked on parents, not on the contents of the moved file/directory.
    /// Validate everything before removing an existing destination.
    pub(crate) fn rename_entry(
        &mut self,
        source: &str,
        target: &str,
        actor: &str,
    ) -> GameResult<()> {
        if source == "/"
            || target == "/"
            || source == target
            || target.starts_with(&format!("{source}/"))
        {
            return Err(domain("invalid copy/move destination"));
        }
        let original = self.stat(source, actor)?;
        self.check_projection(source)?;
        self.check_projection(target)?;
        self.writable_parent(source, actor)?;
        self.writable_parent(target, actor)?;
        if let Some(existing) = self.nodes.get(target) {
            if (original.kind == "directory") != (existing.kind == "directory") {
                return Err(domain("incompatible source and destination types"));
            }
            if existing.kind == "directory"
                && self
                    .nodes
                    .keys()
                    .any(|p| p.starts_with(&format!("{target}/")))
            {
                return Err(domain("Directory not empty"));
            }
        }
        let prefix = format!("{source}/");
        let entries: Vec<_> = self
            .nodes
            .iter()
            .filter(|(path, _)| path.as_str() == source || path.starts_with(&prefix))
            .map(|(path, node)| (path.clone(), node.clone()))
            .collect();
        self.nodes.remove(target);
        for (old, mut node) in entries {
            let path = format!("{target}{}", &old[source.len()..]);
            node.id = path.clone();
            node.parent_id = Some(parent(&path).into());
            node.name = path.rsplit('/').next().unwrap_or("").into();
            self.nodes.remove(&old);
            self.nodes.insert(path, node);
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
        if moving {
            self.check_projection(source)?;
        }
        let source_node = self.stat(source, actor)?.clone();
        let target = if self
            .nodes
            .get(destination)
            .is_some_and(|n| n.kind == "directory")
        {
            format!("{}/{}", destination.trim_end_matches('/'), source_node.name)
        } else {
            destination.into()
        };
        if source == "/" || target == source || target.starts_with(&format!("{source}/")) {
            return Err(domain("invalid copy/move destination"));
        }
        if source_node.kind == "directory" && !recursive && !moving {
            return Err(domain("omitting directory; use -r"));
        }
        self.writable_parent(&target, actor)?;
        if self.nodes.contains_key(&target) {
            return Err(domain("destination already exists"));
        }
        let entries: Vec<_> = self
            .nodes
            .iter()
            .filter(|(p, _)| *p == source || p.starts_with(&format!("{source}/")))
            .map(|(p, n)| (p.clone(), n.clone()))
            .collect();
        if !moving && self.nodes.len() + entries.len() > 10000 {
            return Err(domain("virtual filesystem capacity reached"));
        }
        for (p, n) in &entries {
            self.stat(p, actor)?;
            if !Self::allowed(n, actor, if n.kind == "directory" { 5 } else { 4 }) {
                return Err(domain("permission denied"));
            }
        }
        if moving {
            self.writable_parent(source, actor)?;
        }
        for (p, mut n) in entries {
            let new_path = format!("{target}{}", &p[source.len()..]);
            n.id = new_path.clone();
            n.parent_id = Some(parent(&new_path).into());
            n.name = new_path.rsplit('/').next().unwrap_or("").into();
            if !moving {
                n.owner = actor.into();
                n.group = actor.into();
                n.metadata.remove("packageProjection");
                n.metadata.remove("packageOwner");
            }
            self.nodes.insert(new_path, n);
        }
        if moving {
            self.remove(source, actor, true)?;
        }
        Ok(())
    }

    pub fn chmod(&mut self, path: &str, actor: &str, mode: u16) -> GameResult<()> {
        self.check_projection(path)?;
        let n = self.stat(path, actor)?;
        if mode > 0o777 || (actor != "root" && actor != n.owner) {
            return Err(domain("invalid mode or permission denied"));
        }
        if let Some(n) = self.nodes.get_mut(path) {
            n.mode = mode;
        }
        Ok(())
    }

    pub fn chown(&mut self, path: &str, actor: &str, owner: &str) -> GameResult<()> {
        self.check_projection(path)?;
        self.stat(path, actor)?;
        if actor != "root" || !["root", "kali", "vex"].contains(&owner) {
            return Err(domain("permission denied or unknown user"));
        }
        if let Some(n) = self.nodes.get_mut(path) {
            n.owner = owner.into();
            n.group = owner.into();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn copies_get_numbered_names_without_overwriting_or_changing_extensions() {
        let mut fs = VirtualFileSystem::default();
        for name in ["notes.txt", "firmware.tar.gz", ".env", "README", "ação.txt"] {
            let path = format!("{HOME}/{name}");
            fs.write(&path, "original", "kali").unwrap();
            fs.copy_unique(&path, HOME, "kali").unwrap();
            fs.copy_unique(&path, HOME, "kali").unwrap();
            assert_eq!(fs.read(&path, "kali").unwrap(), "original");
        }
        for name in [
            "notes (1).txt",
            "notes (2).txt",
            "firmware (1).tar.gz",
            ".env (1)",
            "README (1)",
            "ação (2).txt",
        ] {
            assert_eq!(
                fs.read(&format!("{HOME}/{name}"), "kali").unwrap(),
                "original"
            );
        }
        fs.copy_unique(&format!("{HOME}/notes (1).txt"), HOME, "kali")
            .unwrap();
        assert!(fs.nodes.contains_key(&format!("{HOME}/notes (3).txt")));
        fs.mkdir(&format!("{HOME}/folder.v1"), "kali").unwrap();
        fs.write(&format!("{HOME}/folder.v1/child.txt"), "child", "kali")
            .unwrap();
        fs.copy_unique(&format!("{HOME}/folder.v1"), HOME, "kali")
            .unwrap();
        assert_eq!(
            fs.read(&format!("{HOME}/folder.v1 (1)/child.txt"), "kali")
                .unwrap(),
            "child"
        );
        assert!(fs
            .copy_unique(
                &format!("{HOME}/folder.v1"),
                &format!("{HOME}/folder.v1/nested"),
                "kali"
            )
            .is_err());
    }
    #[test]
    fn paths_remain_virtual() {
        assert_eq!(normalize("../../../../etc", HOME).expect("path"), "/etc");
        for p in ["C:\\Windows", "file:///etc", "\\\\server\\share", "a\0b"] {
            assert!(normalize(p, HOME).is_err());
        }
    }
    #[test]
    fn permission_and_mutation_semantics() {
        let mut fs = VirtualFileSystem::default();
        assert!(fs.write("/etc/private", "x", "kali").is_err());
        fs.write("/home/kali/notes.txt", "first", "kali")
            .expect("write");
        fs.transfer(
            "/home/kali/notes.txt",
            "/home/kali/Documents",
            "kali",
            true,
            false,
        )
        .expect("move");
        assert_eq!(
            fs.read("/home/kali/Documents/notes.txt", "kali")
                .expect("read"),
            "first"
        );
        assert!(fs.read("/home/kali/notes.txt", "kali").is_err());
        fs.chmod("/home/kali/Documents", "kali", 0o600)
            .expect("chmod");
        assert!(fs.read("/home/kali/Documents/notes.txt", "kali").is_err());
        assert!(fs.remove("/", "root", true).is_err());
    }

    #[test]
    fn stale_editor_preserves_terminal_changes() {
        let mut fs = VirtualFileSystem::default();
        let path = "/home/kali/Documents/shared.txt";
        fs.write_checked(path, "opened", None, "kali")
            .expect("create");
        fs.write(path, "changed in terminal", "kali")
            .expect("terminal");
        assert!(fs
            .write_checked(path, "stale editor", Some("opened"), "kali")
            .is_err());
        assert!(fs.write_checked(path, "overwrite", None, "kali").is_err());
        assert_eq!(fs.read(path, "kali").expect("read"), "changed in terminal");
        fs.write_checked(path, "merged", Some("changed in terminal"), "kali")
            .expect("fresh save");
    }

    #[test]
    fn trash_moves_restores_and_permanently_removes_virtual_items() {
        let mut fs = VirtualFileSystem::default();
        let path = "/home/kali/Documents/primeiros-passos.txt";
        fs.move_to_trash(path, "kali", true).expect("trash");
        assert!(!fs.nodes.contains_key(path));
        let trashed = format!("{TRASH_FILES}/primeiros-passos.txt");
        assert_eq!(
            fs.nodes
                .get(&trashed)
                .and_then(|node| node.metadata.get("trashOriginalPath")),
            Some(&path.to_string())
        );
        assert!(fs.read(&trashed, "kali").is_ok());
        fs.restore_from_trash(&trashed, "kali").expect("restore");
        assert!(fs.nodes.contains_key(path));
        assert!(!fs.nodes.contains_key(&trashed));
        fs.move_to_trash(path, "kali", true).expect("trash again");
        fs.empty_trash("kali").expect("empty");
        assert!(!fs
            .nodes
            .keys()
            .any(|p| p.starts_with(&format!("{TRASH_FILES}/"))));
    }

    #[test]
    fn trash_uses_unique_names_and_restores_directories_recursively() {
        let mut fs = VirtualFileSystem::default();
        fs.write("/home/kali/Documents/report.txt", "one", "kali")
            .expect("create");
        fs.write("/home/kali/Downloads/report.txt", "two", "kali")
            .expect("create");
        fs.move_to_trash("/home/kali/Documents/report.txt", "kali", true)
            .expect("first trash");
        fs.move_to_trash("/home/kali/Downloads/report.txt", "kali", true)
            .expect("second trash");
        assert!(fs.nodes.contains_key(&format!("{TRASH_FILES}/report.txt")));
        assert!(fs
            .nodes
            .contains_key(&format!("{TRASH_FILES}/report (1).txt")));

        fs.mkdir("/home/kali/projects/trash-folder", "kali")
            .expect("directory");
        fs.write(
            "/home/kali/projects/trash-folder/inside.txt",
            "inside",
            "kali",
        )
        .expect("child");
        fs.move_to_trash("/home/kali/projects/trash-folder", "kali", true)
            .expect("directory trash");
        let trashed_dir = format!("{TRASH_FILES}/trash-folder");
        assert!(fs.nodes.contains_key(&format!("{trashed_dir}/inside.txt")));
        fs.restore_from_trash(&trashed_dir, "kali")
            .expect("directory restore");
        assert_eq!(
            fs.read("/home/kali/projects/trash-folder/inside.txt", "kali")
                .expect("restored child"),
            "inside"
        );
    }
}
