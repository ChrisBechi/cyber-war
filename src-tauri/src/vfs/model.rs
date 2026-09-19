//! Inode storage plus a read-only path projection for existing UI consumers.
use super::parent;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    ops::{Deref, DerefMut},
    sync::Arc,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct Inode {
    #[serde(default)]
    pub ino: u64,
    pub kind: String,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blob: Option<crate::binary::BlobRef>,
    pub owner: String,
    pub group: String,
    pub mode: u16,
    pub created_at: u64,
    pub modified_at: u64,
    #[serde(default)]
    pub accessed_at: u64,
    #[serde(default)]
    pub changed_at: u64,
    #[serde(default)]
    pub nlink: u64,
    #[serde(default)]
    pub uid: u32,
    #[serde(default)]
    pub gid: u32,
    pub metadata: BTreeMap<String, String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VfsNode {
    pub id: String,
    pub parent_id: Option<String>,
    pub name: String,
    #[serde(flatten)]
    pub(super) inode: Arc<Inode>,
}
impl Deref for VfsNode {
    type Target = Inode;
    fn deref(&self) -> &Inode {
        &self.inode
    }
}
impl DerefMut for VfsNode {
    fn deref_mut(&mut self) -> &mut Inode {
        Arc::make_mut(&mut self.inode)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeTable {
    pub(super) events: super::events::EventBus,
    views: BTreeMap<String, VfsNode>,
    pub(super) inodes: BTreeMap<u64, Arc<Inode>>,
    pub(super) links: BTreeMap<u64, BTreeSet<String>>,
    pub(super) directories: BTreeMap<u64, BTreeMap<String, u64>>,
    subdirectories: BTreeMap<u64, u64>,
    pub(super) next_inode: u64,
    pub(super) used_bytes: u64,
    orphans: BTreeSet<u64>,
}
impl Default for NodeTable {
    fn default() -> Self {
        Self {
            events: Default::default(),
            views: BTreeMap::new(),
            inodes: BTreeMap::new(),
            links: BTreeMap::new(),
            directories: BTreeMap::new(),
            subdirectories: BTreeMap::new(),
            next_inode: 1,
            used_bytes: 0,
            orphans: BTreeSet::new(),
        }
    }
}
impl Deref for NodeTable {
    type Target = BTreeMap<String, VfsNode>;
    fn deref(&self) -> &Self::Target {
        &self.views
    }
}
impl<'a> IntoIterator for &'a NodeTable {
    type Item = (&'a String, &'a VfsNode);
    type IntoIter = std::collections::btree_map::Iter<'a, String, VfsNode>;
    fn into_iter(self) -> Self::IntoIter {
        self.views.iter()
    }
}
// UI/DEV projections retain their old shape; saves use the deduplicated wire model.
impl Serialize for NodeTable {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.views.serialize(s)
    }
}
pub struct NodeEdit<'a> {
    table: &'a mut NodeTable,
    node: VfsNode,
}
impl Deref for NodeEdit<'_> {
    type Target = VfsNode;
    fn deref(&self) -> &VfsNode {
        &self.node
    }
}
impl DerefMut for NodeEdit<'_> {
    fn deref_mut(&mut self) -> &mut VfsNode {
        &mut self.node
    }
}
impl Drop for NodeEdit<'_> {
    fn drop(&mut self) {
        if let Some(old) = self.table.inodes.get(&self.node.ino) {
            if old.owner != self.node.owner
                && ["root", "kali", "vex"].contains(&self.node.owner.as_str())
            {
                self.node.uid = super::identity_id(&self.node.owner);
            }
            if old.group != self.node.group
                && ["root", "kali", "vex"].contains(&self.node.group.as_str())
            {
                self.node.gid = super::identity_id(&self.node.group);
            }
        }
        self.table.replace_inode(self.node.inode.clone());
    }
}
impl NodeTable {
    pub(crate) fn path_for_inode(&self, ino: u64) -> Option<&String> {
        self.links.get(&ino)?.first()
    }

    pub(crate) fn get_mut(&mut self, path: &str) -> Option<NodeEdit<'_>> {
        let node = self.views.get(path)?.clone();
        Some(NodeEdit { table: self, node })
    }
    pub(super) fn replace_inode(&mut self, inode: Arc<Inode>) {
        let ino = inode.ino;
        if let Some(old) = self.inodes.get(&ino) {
            self.events.inode(old, &inode, self.links.get(&ino));
        }
        if let Some(paths) = self.links.get(&ino) {
            for path in paths {
                if let Some(node) = self.views.get_mut(path) {
                    node.inode = inode.clone();
                }
            }
        }
        if let Some(old) = self.inodes.insert(ino, inode.clone()) {
            self.used_bytes = self.used_bytes.saturating_sub(old.logical_size());
        }
        self.used_bytes = self.used_bytes.saturating_add(inode.logical_size());
    }
    pub(super) fn update_links(&mut self, ino: u64) {
        let Some(mut inode) = self.inodes.get(&ino).cloned() else {
            return;
        };
        let count = self.links.get(&ino).map_or(0, BTreeSet::len) as u64;
        if count == 0 {
            self.orphans.insert(ino);
        } else {
            self.orphans.remove(&ino);
        }
        let nlink = if inode.kind == "directory" && count > 0 {
            2 + self.subdirectories.get(&ino).copied().unwrap_or(0)
        } else {
            count
        };
        Arc::make_mut(&mut inode).nlink = nlink;
        self.replace_inode(inode);
    }
    pub(crate) fn insert(&mut self, path: String, mut node: VfsNode) -> Option<VfsNode> {
        let old = self.remove(&path);
        if node.ino == 0 {
            node.ino = self.next_inode;
            self.next_inode += 1;
        } else {
            self.next_inode = self.next_inode.max(node.ino + 1);
        }
        node.id = path.clone();
        node.parent_id = (path != "/").then(|| parent(&path).into());
        node.name = path.rsplit('/').next().unwrap_or("").into();
        if let Some(inode) = self.inodes.get(&node.ino) {
            node.inode = inode.clone();
        }
        self.replace_inode(node.inode.clone());
        self.links.entry(node.ino).or_default().insert(path.clone());
        if node.kind == "directory" {
            self.directories.entry(node.ino).or_default();
        }
        let ino = node.ino;
        let parent_ino = node
            .parent_id
            .as_ref()
            .and_then(|p| self.views.get(p))
            .map(|n| n.ino);
        if let Some(id) = parent_ino {
            self.directories
                .entry(id)
                .or_default()
                .insert(node.name.clone(), ino);
            if node.kind == "directory" {
                *self.subdirectories.entry(id).or_default() += 1;
            }
        }
        self.events.namespace(&path);
        self.views.insert(path, node);
        self.update_links(ino);
        if let Some(id) = parent_ino {
            self.update_links(id);
        }
        old
    }
    pub(crate) fn remove(&mut self, path: &str) -> Option<VfsNode> {
        let node = self.views.remove(path)?;
        self.events.namespace(path);
        if let Some(paths) = self.links.get_mut(&node.ino) {
            paths.remove(path);
        }
        let parent_ino = node
            .parent_id
            .as_ref()
            .and_then(|p| self.views.get(p))
            .map(|n| n.ino);
        if let Some(id) = parent_ino {
            if let Some(entries) = self.directories.get_mut(&id) {
                entries.remove(&node.name);
            }
            if node.kind == "directory" {
                if let Some(count) = self.subdirectories.get_mut(&id) {
                    *count = count.saturating_sub(1);
                }
            }
        }
        self.update_links(node.ino);
        if let Some(id) = parent_ino {
            self.update_links(id);
        }
        Some(node)
    }
    pub(crate) fn retain(&mut self, mut keep: impl FnMut(&String, &VfsNode) -> bool) {
        let remove: Vec<_> = self
            .views
            .iter()
            .filter(|(p, n)| !keep(p, n))
            .map(|(p, _)| p.clone())
            .collect();
        for path in remove.into_iter().rev() {
            self.remove(&path);
        }
    }
    pub(crate) fn edit_all(&mut self, mut edit: impl FnMut(&mut Inode)) {
        let ids: Vec<_> = self.inodes.keys().copied().collect();
        for id in ids {
            let old = self.inodes[&id].clone();
            let mut node = old.clone();
            let next = Arc::make_mut(&mut node);
            edit(next);
            if next.owner != old.owner && ["root", "kali", "vex"].contains(&next.owner.as_str()) {
                next.uid = super::identity_id(&next.owner);
            }
            if next.group != old.group && ["root", "kali", "vex"].contains(&next.group.as_str()) {
                next.gid = super::identity_id(&next.group);
            }
            self.replace_inode(node);
        }
    }
    pub(super) fn collect(&mut self, held: &BTreeSet<u64>) {
        let dead: Vec<_> = self.orphans.difference(held).copied().collect();
        for id in dead {
            if let Some(n) = self.inodes.remove(&id) {
                self.used_bytes = self.used_bytes.saturating_sub(n.logical_size());
            }
            self.orphans.remove(&id);
            self.links.remove(&id);
            self.directories.remove(&id);
            self.subdirectories.remove(&id);
        }
    }
}
