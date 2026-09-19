use super::*;
use serde::{Deserializer, Serializer};
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Snapshot {
    format_version: u32,
    entries: BTreeMap<String, u64>,
    inodes: BTreeMap<u64, Inode>,
    next_inode: u64,
    clock: u64,
    #[serde(default = "default_disk_capacity")]
    capacity_bytes: u64,
    #[serde(default)]
    identities: BTreeMap<String, Identity>,
    #[serde(default)]
    read_only: bool,
}
#[derive(Deserialize)]
struct Legacy {
    nodes: BTreeMap<String, VfsNode>,
    clock: u64,
    #[serde(default = "default_disk_capacity")]
    capacity_bytes: u64,
}
impl Serialize for VirtualFileSystem {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        Snapshot {
            format_version: 2,
            entries: self.nodes.iter().map(|(p, n)| (p.clone(), n.ino)).collect(),
            inodes: self
                .nodes
                .inodes
                .iter()
                .filter(|(_, n)| n.nlink > 0)
                .map(|(id, n)| (*id, (**n).clone()))
                .collect(),
            next_inode: self.nodes.next_inode,
            clock: self.clock,
            capacity_bytes: self.capacity_bytes,
            identities: self.identities.clone(),
            read_only: self.read_only,
        }
        .serialize(s)
    }
}
impl<'de> Deserialize<'de> for VirtualFileSystem {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(d)?;
        let mut fs = Self::empty();
        if value.get("formatVersion").is_some() {
            let wire: Snapshot = serde_json::from_value(value).map_err(serde::de::Error::custom)?;
            if wire.format_version != 2 {
                return Err(serde::de::Error::custom("unsupported VFS snapshot version"));
            }
            if wire.entries.len() > 10000 || wire.inodes.len() > 10000 {
                return Err(serde::de::Error::custom("VFS entry limit exceeded"));
            }
            fs.clock = wire.clock;
            fs.capacity_bytes = wire.capacity_bytes;
            fs.identities = wire.identities;
            fs.read_only = wire.read_only;
            let mut entries: Vec<_> = wire.entries.into_iter().collect();
            entries.sort_by_key(|(p, _)| (p.matches('/').count(), p.clone()));
            for (path, ino) in entries {
                let inode = wire.inodes.get(&ino).ok_or_else(|| {
                    serde::de::Error::custom("directory entry references missing inode")
                })?;
                if ino == 0 || ino == u64::MAX || inode.ino != ino {
                    return Err(serde::de::Error::custom("invalid inode identity"));
                }
                fs.nodes.insert(
                    path.clone(),
                    VfsNode {
                        id: path,
                        parent_id: None,
                        name: String::new(),
                        inode: std::sync::Arc::new(inode.clone()),
                    },
                );
            }
            for (ino, node) in &wire.inodes {
                if fs
                    .nodes
                    .inodes
                    .get(ino)
                    .is_none_or(|n| n.nlink != node.nlink)
                {
                    return Err(serde::de::Error::custom("inconsistent inode link count"));
                }
            }
            if wire.next_inode == u64::MAX || wire.next_inode < fs.nodes.next_inode {
                return Err(serde::de::Error::custom(
                    "inode allocator overlaps saved identities",
                ));
            }
            fs.nodes.next_inode = wire.next_inode;
        } else {
            let legacy: Legacy = serde_json::from_value(value).map_err(serde::de::Error::custom)?;
            if legacy.nodes.len() > 10000 {
                return Err(serde::de::Error::custom("VFS entry limit exceeded"));
            }
            fs.clock = legacy.clock;
            fs.capacity_bytes = legacy.capacity_bytes;
            let mut entries: Vec<_> = legacy.nodes.into_iter().collect();
            entries.sort_by_key(|(p, _)| (p.matches('/').count(), p.clone()));
            for (path, mut node) in entries {
                fs.register_identity(&node.owner);
                fs.register_identity(&node.group);
                node.ino = 0;
                node.changed_at = node.modified_at;
                node.accessed_at = node.modified_at;
                node.uid = fs.identity(&node.owner).uid;
                node.gid = fs.identity(&node.group).gid;
                fs.nodes.insert(path, node);
            }
            // Empty network fixture filesystems are populated by VirtualNetwork::initial.
            if fs.nodes.is_empty() {
                return Ok(fs);
            }
            if fs.nodes.get("/dev").is_some_and(|n| n.kind == "directory") {
                for name in ["null", "zero", "full"] {
                    let path = format!("/dev/{name}");
                    if !fs.nodes.contains_key(&path) {
                        fs.seed(&path, "charDevice", "", "root");
                        if let Some(mut n) = fs.nodes.get_mut(&path) {
                            n.mode = 0o666;
                            n.metadata.insert("device".into(), name.into());
                        }
                    }
                }
            }
        }
        fs.check_invariants().map_err(serde::de::Error::custom)?;
        Ok(fs)
    }
}
