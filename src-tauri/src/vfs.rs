//! One virtual filesystem shared by commands, UI, missions and package bindings.
use crate::error::{GameError, GameResult};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
mod errors;
#[cfg(test)]
mod event_tests;
mod events;
#[cfg(test)]
mod fidelity_tests;
mod handles;
#[cfg(test)]
mod integration_tests;
mod model;
mod operations;
#[cfg(test)]
mod performance_tests;
mod persistence;
mod resolve;
use errors::error;
pub use errors::Errno;
#[cfg(test)]
pub use events::VfsEvent;
pub use events::{WatchId, WatchTarget};
pub use handles::OpenFlags;
use model::NodeTable;
pub use model::{Inode, VfsNode};
pub use resolve::Follow;
pub const HOME: &str = "/home/kali";
pub const TRASH_ROOT: &str = "/home/kali/.local/share/Trash";
pub const TRASH_FILES: &str = "/home/kali/.local/share/Trash/files";
pub const TRASH_INFO: &str = "/home/kali/.local/share/Trash/info";
const MAX_CONTENT: usize = 1_048_576;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Identity {
    pub uid: u32,
    pub gid: u32,
    pub groups: BTreeSet<u32>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VirtualFileSystem {
    pub nodes: NodeTable,
    clock: u64,
    pub capacity_bytes: u64,
    identities: BTreeMap<String, Identity>,
    pub read_only: bool,
    pub umask: u16,
    handles: BTreeMap<u64, handles::Handle>,
    next_handle: u64,
}
fn default_disk_capacity() -> u64 {
    64 * 1024 * 1024 * 1024
}
fn identity_id(name: &str) -> u32 {
    match name {
        "root" => 0,
        "kali" => 1000,
        "vex" => 1001,
        _ => {
            65536
                + name
                    .bytes()
                    .fold(0u32, |n, b| n.wrapping_mul(31).wrapping_add(b as u32))
                    % 1_000_000
        }
    }
}
impl Inode {
    pub fn storage_size(&self) -> u64 {
        self.blob
            .as_ref()
            .map_or(self.content.len() as u64, |b| b.size as u64)
    }
    pub fn logical_size(&self) -> u64 {
        if self.kind == "directory" || self.kind == "charDevice" {
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
/// Make absolute without erasing semantic '..', '.', or trailing slash.
pub fn normalize(path: &str, cwd: &str) -> GameResult<String> {
    resolve::validate_path(path)?;
    let absolute = if path.starts_with('/') {
        path.to_owned()
    } else {
        format!("{}/{path}", cwd.trim_end_matches('/'))
    };
    resolve::validate_path(&absolute)?;
    Ok(absolute)
}
pub fn parent(path: &str) -> &str {
    path.rsplit_once('/')
        .map(|(p, _)| if p.is_empty() { "/" } else { p })
        .unwrap_or("/")
}
impl Default for VirtualFileSystem {
    fn default() -> Self {
        let mut fs = Self::empty();
        for path in [
            "/",
            "/bin",
            "/boot",
            "/dev",
            "/etc",
            "/home",
            HOME,
            "/opt",
            "/root",
            "/tmp",
            "/usr",
            "/usr/bin",
            "/usr/local",
            "/usr/local/bin",
            "/usr/local/sbin",
            "/usr/sbin",
            "/usr/share",
            "/usr/share/applications",
            "/usr/share/doc",
            "/var",
            "/var/log",
            "/srv",
            "/srv/www",
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
        if let Some(mut n) = fs.nodes.get_mut("/root") {
            n.mode = 0o700;
        }
        if let Some(mut n) = fs.nodes.get_mut("/tmp") {
            n.mode = 0o1777;
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
            if let Some(mut node) = fs.nodes.get_mut(path) {
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
        if let Some(mut node) = fs.nodes.get_mut("/home/kali/Videos/cyber-war-opening.vtt") {
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
            if let Some(mut n) = fs.nodes.get_mut(&path) {
                n.metadata.insert("app".into(), app.into());
            }
        }
        fs.seed("/dev/null", "charDevice", "", "root");
        fs.seed("/dev/zero", "charDevice", "", "root");
        fs.seed("/dev/full", "charDevice", "", "root");
        for path in ["/dev/null", "/dev/zero", "/dev/full"] {
            if let Some(mut n) = fs.nodes.get_mut(path) {
                n.mode = 0o666;
                n.metadata
                    .insert("device".into(), path.rsplit('/').next().unwrap().into());
            }
        }
        fs
    }
}

impl VirtualFileSystem {
    fn empty() -> Self {
        Self {
            nodes: NodeTable::default(),
            clock: 0,
            capacity_bytes: default_disk_capacity(),
            identities: ["root", "kali", "vex"]
                .into_iter()
                .map(|name| {
                    let uid = identity_id(name);
                    (
                        name.into(),
                        Identity {
                            uid,
                            gid: uid,
                            groups: BTreeSet::from([uid]),
                        },
                    )
                })
                .collect(),
            read_only: false,
            umask: 0o022,
            handles: BTreeMap::new(),
            next_handle: 1,
        }
    }
    fn tick(&mut self) -> u64 {
        self.clock = self.clock.saturating_add(1);
        self.clock
    }
    pub fn identity(&self, name: &str) -> Identity {
        self.identities.get(name).cloned().unwrap_or_else(|| {
            let uid = u32::MAX;
            Identity {
                uid,
                gid: uid,
                groups: BTreeSet::from([uid]),
            }
        })
    }
    fn register_identity(&mut self, name: &str) {
        if !self.identities.contains_key(name) {
            let uid = self
                .identities
                .values()
                .map(|i| i.uid)
                .max()
                .unwrap_or(1000)
                + 1;
            self.identities.insert(
                name.into(),
                Identity {
                    uid,
                    gid: uid,
                    groups: BTreeSet::from([uid]),
                },
            );
        }
    }
    pub fn set_identity(&mut self, name: &str, identity: Identity) {
        self.identities.insert(name.into(), identity);
    }
    /// Virtual user database lookup, separate from the process credential name.
    pub fn username_for_uid(&self, uid: u32) -> Option<String> {
        if self.nodes.contains_key("/etc/passwd") {
            return self
                .read("/etc/passwd", "root")
                .ok()?
                .lines()
                .find_map(|line| {
                    let fields: Vec<_> = line.split(':').collect();
                    (fields.len() >= 7 && fields[2].parse::<u32>().ok() == Some(uid))
                        .then(|| fields[0].to_owned())
                });
        }
        self.identities
            .iter()
            .find_map(|(name, identity)| (identity.uid == uid).then(|| name.clone()))
    }
    pub fn allowed(&self, node: &VfsNode, actor: &str, bits: u16) -> bool {
        let who = self.identity(actor);
        if who.uid == 0 {
            return bits & 1 == 0 || node.kind == "directory" || node.mode & 0o111 != 0;
        }
        let shift = if who.uid == node.uid {
            6
        } else if who.gid == node.gid || who.groups.contains(&node.gid) {
            3
        } else {
            0
        };
        (node.mode >> shift) & bits == bits
    }
    fn check_projection(&self, path: &str) -> GameResult<()> {
        if self.read_only {
            return Err(error(Errno::ReadOnly));
        }
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
        self.nodes.used_bytes
    }
    pub fn check_space(&self, path: &str, size: u64) -> GameResult<()> {
        let old = self.nodes.get(path).map_or(0, |n| n.logical_size());
        if self.used_bytes().saturating_sub(old).saturating_add(size) > self.capacity_bytes {
            return Err(error(Errno::NoSpace));
        }
        Ok(())
    }
    pub fn seed(&mut self, path: &str, kind: &str, content: &str, owner: &str) {
        self.register_identity(owner);
        let clock = self.tick();
        let identity = self.identity(owner);
        let inode = Inode {
            ino: 0,
            kind: kind.into(),
            content: content.into(),
            blob: None,
            owner: owner.into(),
            group: owner.into(),
            uid: identity.uid,
            gid: identity.gid,
            mode: if kind == "directory" { 0o755 } else { 0o644 },
            created_at: clock,
            modified_at: clock,
            changed_at: clock,
            accessed_at: clock,
            nlink: 0,
            metadata: BTreeMap::new(),
        };
        self.nodes.insert(
            path.into(),
            VfsNode {
                id: path.into(),
                parent_id: None,
                name: String::new(),
                inode: Arc::new(inode),
            },
        );
        self.collect();
    }
    /// Trusted fixture/mission metadata transformation; all aliases share the canonical inode.
    pub(crate) fn update_all_metadata(&mut self, update: impl FnMut(&mut Inode)) {
        self.nodes.edit_all(update);
    }
    pub(crate) fn collect(&mut self) {
        self.nodes
            .collect(&self.handles.values().map(|h| h.ino).collect());
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
    pub fn directory(&self, path: &str, actor: &str) -> GameResult<()> {
        let n = self.stat(path, actor)?;
        if n.kind != "directory" {
            return Err(error(Errno::NotDirectory));
        }
        if !self.allowed(n, actor, 1) {
            return Err(error(Errno::Access));
        }
        Ok(())
    }
    pub fn child_names(&self, path: &str, actor: &str) -> GameResult<Vec<String>> {
        let node = self.stat(path, actor)?;
        if node.kind != "directory" {
            return Err(error(Errno::NotDirectory));
        }
        if !self.allowed(node, actor, 4) {
            return Err(error(Errno::Access));
        }
        Ok(self
            .nodes
            .directories
            .get(&node.ino)
            .into_iter()
            .flat_map(|d| d.keys().cloned())
            .collect())
    }
    pub fn list(&self, path: &str, actor: &str) -> GameResult<Vec<VfsNode>> {
        let resolved = self.resolve(path, actor, Follow::Yes)?;
        self.directory(&resolved, actor)?;
        self.child_names(&resolved, actor)?
            .into_iter()
            .map(|name| {
                let mut node = self
                    .nodes
                    .get(&format!("{}/{name}", resolved.trim_end_matches('/')))
                    .cloned()
                    .ok_or_else(|| error(Errno::NotFound))?;
                if resolved == TRASH_FILES {
                    if let Some(info) = self.nodes.get(&format!("{TRASH_INFO}/{name}.trashinfo")) {
                        node.metadata
                            .insert("trashOriginalPath".into(), info.content.clone());
                    }
                } else {
                    node.metadata.remove("trashOriginalPath");
                    node.metadata.remove("trashDeletedAt");
                }
                Ok(node)
            })
            .collect()
    }
    pub fn readable(&self, path: &str, actor: &str) -> GameResult<&VfsNode> {
        let n = self.stat(path, actor)?;
        if n.kind == "directory" {
            return Err(error(Errno::IsDirectory));
        }
        if !self.allowed(n, actor, 4) {
            return Err(error(Errno::Access));
        }
        Ok(n)
    }
    pub fn read(&self, path: &str, actor: &str) -> GameResult<String> {
        let n = self.readable(path, actor)?;
        if n.kind == "charDevice" {
            return if n.metadata.get("device").is_some_and(|d| d == "null") {
                Ok(String::new())
            } else {
                Err(domain("device requires a bounded virtual read"))
            };
        }
        if n.blob.is_some() {
            return Err(domain("binary file: text access is not supported"));
        }
        Ok(n.content.clone())
    }
    fn writable_parent(&self, path: &str, actor: &str) -> GameResult<()> {
        if self.read_only {
            return Err(error(Errno::ReadOnly));
        }
        let n = self.stat(parent(path), actor)?;
        if n.kind != "directory" {
            return Err(error(Errno::NotDirectory));
        }
        if !self.allowed(n, actor, 3) {
            return Err(error(Errno::Access));
        }
        Ok(())
    }
    fn sticky(&self, path: &str, actor: &str) -> GameResult<()> {
        let p = self.stat(parent(path), actor)?;
        let n = self.lstat(path, actor)?;
        let uid = self.identity(actor).uid;
        if p.mode & 0o1000 != 0 && uid != 0 && uid != p.uid && uid != n.uid {
            return Err(error(Errno::NotPermitted));
        }
        Ok(())
    }
    fn changed_parent(&mut self, path: &str) {
        let clock = self.tick();
        if let Some(mut n) = self.nodes.get_mut(parent(path)) {
            n.modified_at = clock;
            n.changed_at = clock;
        }
    }
    fn create(
        &mut self,
        path: &str,
        kind: &str,
        content: &str,
        actor: &str,
        mode: u16,
    ) -> GameResult<String> {
        let path = if kind == "directory" {
            if path.chars().all(|c| c == '/') {
                "/"
            } else {
                path.trim_end_matches('/')
            }
        } else {
            path
        };
        let path = self.resolve_missing(path, actor, Follow::No, true)?;
        if self.nodes.contains_key(&path) {
            return Err(error(Errno::Exists));
        }
        self.writable_parent(&path, actor)?;
        if self.nodes.len() >= 10000 {
            return Err(error(Errno::NoSpace));
        }
        self.check_space(&path, content.len() as u64)?;
        let parent_node = self.stat(parent(&path), actor)?.clone();
        self.seed(&path, kind, content, actor);
        if let Some(mut n) = self.nodes.get_mut(&path) {
            n.mode = if kind == "symlink" {
                0o777
            } else {
                mode & !self.umask & 0o7777
            };
            if parent_node.mode & 0o2000 != 0 {
                n.group = parent_node.group.clone();
                n.gid = parent_node.gid;
                if kind == "directory" {
                    n.mode |= 0o2000;
                }
            }
        }
        self.changed_parent(&path);
        Ok(path)
    }
    pub fn symlink(&mut self, path: &str, target: &str, actor: &str) -> GameResult<()> {
        resolve::validate_path(target)?;
        self.create(path, "symlink", target, actor, 0o777)?;
        Ok(())
    }
    pub fn link(&mut self, source: &str, target: &str, actor: &str) -> GameResult<()> {
        let source = self.resolve(source, actor, Follow::No)?;
        let target = self.resolve_missing(target, actor, Follow::No, true)?;
        let original = self.lstat(&source, actor)?.clone();
        if original.kind == "directory" {
            return Err(error(Errno::NotPermitted));
        }
        self.check_projection(&source)?;
        self.writable_parent(&target, actor)?;
        if self.nodes.contains_key(&target) {
            return Err(error(Errno::Exists));
        }
        if self.nodes.len() >= 10000 {
            return Err(error(Errno::NoSpace));
        }
        self.nodes.insert(target.clone(), original);
        let clock = self.tick();
        if let Some(mut n) = self.nodes.get_mut(&source) {
            n.changed_at = clock;
        }
        self.changed_parent(&target);
        Ok(())
    }
    pub fn mkdir(&mut self, path: &str, actor: &str) -> GameResult<()> {
        self.create(path, "directory", "", actor, 0o777)?;
        Ok(())
    }
    pub fn write_checked(
        &mut self,
        path: &str,
        content: &str,
        expected: Option<&str>,
        actor: &str,
    ) -> GameResult<()> {
        match (self.path_exists(path,actor)?,expected){(true,Some(old)) if self.read(path,actor)?==old=>{},(false,None)=>{},_=>return Err(domain("Arquivo mudou desde a abertura. Reabra antes de salvar; seu texto foi preservado no editor."))}
        self.write(path, content, actor)
    }
    pub fn prepare_output(&mut self, path: &str, actor: &str, append: bool) -> GameResult<()> {
        let h = self.open(
            path,
            OpenFlags {
                write: true,
                create: true,
                truncate: !append,
                append,
                ..OpenFlags::default()
            },
            0o666,
            actor,
        )?;
        self.close(h)
    }
    pub fn write(&mut self, path: &str, content: &str, actor: &str) -> GameResult<()> {
        if content.len() > MAX_CONTENT {
            return Err(domain("virtual file limit: 1 MiB"));
        }
        let path = self.resolve_missing(path, actor, Follow::Yes, true)?;
        if !self.nodes.contains_key(&path) {
            self.create(&path, "file", content, actor, 0o666)?;
            return Ok(());
        }
        self.check_projection(&path)?;
        let node = self.stat(&path, actor)?;
        if node.kind == "directory" {
            return Err(error(Errno::IsDirectory));
        }
        if !self.allowed(node, actor, 2) {
            return Err(error(Errno::Access));
        }
        if node.kind == "charDevice" {
            if node.metadata.get("device").is_some_and(|d| d == "full") {
                return Err(error(Errno::NoSpace));
            }
            return if node
                .metadata
                .get("device")
                .is_some_and(|d| d == "null" || d == "zero")
            {
                Ok(())
            } else {
                Err(error(Errno::Invalid))
            };
        }
        self.check_space(&path, content.len() as u64)?;
        let clock = self.tick();
        if let Some(mut n) = self.nodes.get_mut(&path) {
            n.content = content.into();
            n.blob = None;
            n.modified_at = clock;
            n.changed_at = clock;
            if actor != "root" {
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
        let path = self.resolve_missing(path, actor, Follow::Yes, true)?;
        if self
            .nodes
            .get(&path)
            .is_some_and(|n| n.kind == "charDevice")
        {
            return self.write(&path, "", actor);
        }
        self.check_space(&path, blob.size as u64)?;
        self.write(&path, "", actor)?;
        if let Some(mut n) = self.nodes.get_mut(&path) {
            n.metadata.insert("mime".into(), blob.mime.clone());
            n.blob = Some(blob);
        }
        Ok(())
    }
    pub fn chmod(&mut self, path: &str, actor: &str, mode: u16) -> GameResult<()> {
        let path = self.resolve(path, actor, Follow::Yes)?;
        self.check_projection(&path)?;
        let n = self.stat(&path, actor)?;
        if mode > 0o7777 {
            return Err(error(Errno::Invalid));
        }
        if actor != "root" && self.identity(actor).uid != n.uid {
            return Err(error(Errno::NotPermitted));
        }
        let clock = self.tick();
        if let Some(mut n) = self.nodes.get_mut(&path) {
            n.mode = mode;
            n.changed_at = clock;
        }
        Ok(())
    }
    pub fn ownership(
        &mut self,
        path: &str,
        actor: &str,
        owner: Option<&str>,
        group: Option<&str>,
        follow: Follow,
    ) -> GameResult<()> {
        let path = self.resolve(path, actor, follow)?;
        self.check_projection(&path)?;
        if actor == "root" {
            for name in owner.into_iter().chain(group) {
                self.register_identity(name);
            }
        }
        let n = self.lstat(&path, actor)?;
        let who = self.identity(actor);
        let uid = owner.map(|s| self.identity(s).uid);
        let gid = group.map(|s| self.identity(s).gid);
        if who.uid != 0
            && (who.uid != n.uid
                || uid.is_some_and(|id| id != n.uid)
                || gid.is_some_and(|id| id != who.gid && !who.groups.contains(&id)))
        {
            return Err(error(Errno::NotPermitted));
        }
        let clock = self.tick();
        if let Some(mut n) = self.nodes.get_mut(&path) {
            if let Some(o) = owner {
                n.owner = o.into();
                n.uid = uid.unwrap_or(n.uid);
            }
            if let Some(g) = group {
                n.group = g.into();
                n.gid = gid.unwrap_or(n.gid);
            }
            n.mode &= !0o6000;
            n.changed_at = clock;
        }
        Ok(())
    }
    pub fn chown(&mut self, path: &str, actor: &str, owner: &str) -> GameResult<()> {
        self.ownership(path, actor, Some(owner), None, Follow::Yes)
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
        let mut candidate = self.clone();
        candidate.trash_entry(path, actor, recursive)?;
        *self = candidate;
        Ok(())
    }
    fn trash_entry(&mut self, path: &str, actor: &str, recursive: bool) -> GameResult<()> {
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
        let path = self.resolve(path, actor, Follow::No)?;
        let path = path.as_str();
        let source = self.lstat(path, actor)?.clone();
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
        let info = format!(
            "{TRASH_INFO}/{}.trashinfo",
            target.rsplit('/').next().unwrap()
        );
        self.write(&info, path, actor)?;
        if let Some(mut node) = self.nodes.get_mut(&target) {
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
        let node = self.lstat(path, actor)?;
        let info = format!("{TRASH_INFO}/{}.trashinfo", node.name);
        let original = match self.read(&info, actor) {
            Ok(path) => path,
            Err(GameError::Vfs(Errno::NotFound)) => node
                .metadata
                .get("trashOriginalPath")
                .cloned()
                .ok_or_else(|| domain("item da lixeira sem caminho original"))?,
            Err(e) => return Err(e),
        };
        if Self::is_trash_path(&original) || original == "/" {
            return Err(domain("caminho original inválido"));
        }
        if self.nodes.contains_key(&original) {
            return Err(domain("o caminho original já está ocupado"));
        }
        self.writable_parent(&original, actor)?;
        self.transfer(path, &original, actor, true, true)?;
        if self.nodes.contains_key(&info) {
            self.unlink(&info, actor)?;
        }
        if let Some(mut node) = self.nodes.get_mut(&original) {
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
            let info = format!(
                "{TRASH_INFO}/{}.trashinfo",
                path.rsplit('/').next().unwrap()
            );
            if self.nodes.contains_key(&info) {
                self.unlink(&info, actor)?;
            }
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
        let source = self.resolve(source, actor, Follow::No)?;
        let destination = self.resolve_missing(destination, actor, Follow::Yes, true)?;
        let source = source.as_str();
        let destination = destination.as_str();
        let node = self.lstat(source, actor)?;
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
        let fs = VirtualFileSystem::default();
        assert_eq!(
            fs.resolve(
                &normalize("../../../../etc", HOME).unwrap(),
                "kali",
                Follow::Yes
            )
            .unwrap(),
            "/etc"
        );
        for p in ["C:\\Windows", "file:///etc", "\\\\server\\share"] {
            assert!(normalize(p, HOME).unwrap().starts_with(HOME));
        }
        assert!(normalize("a\0b", HOME).is_err());
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
