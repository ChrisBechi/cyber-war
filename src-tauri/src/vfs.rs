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
}

pub fn domain(message: impl Into<String>) -> GameError {
    GameError::Domain(message.into())
}

pub fn normalize(path: &str, cwd: &str) -> GameResult<String> {
    if path.len() > 4096 || path.contains(['\\', ':']) || path.chars().any(char::is_control) {
        return Err(domain("invalid virtual path"));
    }
    let expanded = if path == "~" || path.starts_with("~/") {
        format!("{HOME}{}", &path[1..])
    } else if path.starts_with('/') {
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

    pub fn transfer(
        &mut self,
        source: &str,
        destination: &str,
        actor: &str,
        moving: bool,
        recursive: bool,
    ) -> GameResult<()> {
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
            }
            self.nodes.insert(new_path, n);
        }
        if moving {
            self.remove(source, actor, true)?;
        }
        Ok(())
    }

    pub fn chmod(&mut self, path: &str, actor: &str, mode: u16) -> GameResult<()> {
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
