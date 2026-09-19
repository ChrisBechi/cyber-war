//! World-local change subscriptions. No kernel watcher, clock, or event history.
//!
//! Subscribe before inspecting the target, then acknowledge only the revision
//! inspected. A later mutation remains pending even if it precedes suspension.
use super::*;

pub const WATCH_LIMIT: usize = 1024;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum WatchTarget {
    Inode(u64),
    /// Absolute namespace path; it need not exist. Ancestor replacement and
    /// permission changes also affect this target. Symlink resolution remains
    /// the caller's operation; consumers may subscribe to several dependencies.
    Path(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct WatchId(u64);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Change {
    pub content: bool,
    pub metadata: bool,
    pub namespace: bool,
    /// Smallest length reached since acknowledgement, retained across a later
    /// append. Consumers can distinguish truncate/append from ordinary growth.
    pub truncated_to: Option<u64>,
}
impl Change {
    fn merge(&mut self, other: Self) {
        self.content |= other.content;
        self.metadata |= other.metadata;
        self.namespace |= other.namespace;
        self.truncated_to = match (self.truncated_to, other.truncated_to) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        };
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VfsEvent {
    pub revision: u64,
    pub change: Change,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Subscription {
    target: WatchTarget,
    revision: u64,
    pending: Option<VfsEvent>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct EventBus {
    revision: u64,
    next: u64,
    subscriptions: BTreeMap<WatchId, Subscription>,
    inodes: BTreeMap<u64, BTreeSet<WatchId>>,
    paths: BTreeMap<String, BTreeSet<WatchId>>,
}
impl EventBus {
    fn subscribe(&mut self, target: WatchTarget) -> GameResult<WatchId> {
        if self.subscriptions.len() >= WATCH_LIMIT {
            return Err(error(Errno::NoSpace));
        }
        if let WatchTarget::Path(path) = &target {
            super::resolve::validate_path(path)?;
            if !path.starts_with('/') {
                return Err(error(Errno::Invalid));
            }
        }
        self.next = self
            .next
            .checked_add(1)
            .ok_or_else(|| error(Errno::NoSpace))?;
        let id = WatchId(self.next);
        match &target {
            WatchTarget::Inode(ino) => {
                self.inodes.entry(*ino).or_default().insert(id);
            }
            WatchTarget::Path(path) => {
                self.paths.entry(path.clone()).or_default().insert(id);
            }
        }
        self.subscriptions.insert(
            id,
            Subscription {
                target,
                revision: self.revision,
                pending: None,
            },
        );
        Ok(id)
    }
    fn unsubscribe(&mut self, id: WatchId) {
        let Some(subscription) = self.subscriptions.remove(&id) else {
            return;
        };
        match subscription.target {
            WatchTarget::Inode(ino) => {
                if let Some(ids) = self.inodes.get_mut(&ino) {
                    ids.remove(&id);
                    if ids.is_empty() {
                        self.inodes.remove(&ino);
                    }
                }
            }
            WatchTarget::Path(path) => {
                if let Some(ids) = self.paths.get_mut(&path) {
                    ids.remove(&id);
                    if ids.is_empty() {
                        self.paths.remove(&path);
                    }
                }
            }
        }
    }
    fn path_ids(&self, path: &str, descendants: bool, ids: &mut BTreeSet<WatchId>) {
        if let Some(exact) = self.paths.get(path) {
            ids.extend(exact);
        }
        if descendants {
            let prefix = format!("{}/", path.trim_end_matches('/'));
            for (_, subscribers) in self
                .paths
                .range(prefix.clone()..)
                .take_while(|(p, _)| p.starts_with(&prefix))
            {
                ids.extend(subscribers);
            }
        }
    }
    fn emit(&mut self, ids: BTreeSet<WatchId>, change: Change) {
        // The revision belongs to the virtual namespace, including mutations
        // without subscribers. Storage remains O(subscriptions), not O(writes).
        self.revision = self.revision.saturating_add(1);
        for id in ids {
            if let Some(subscription) = self.subscriptions.get_mut(&id) {
                subscription.revision = self.revision;
                let event = subscription.pending.get_or_insert(VfsEvent {
                    revision: self.revision,
                    change: Change::default(),
                });
                event.revision = self.revision;
                event.change.merge(change);
            }
        }
    }
    pub(super) fn namespace(&mut self, path: &str) {
        let mut ids = BTreeSet::new();
        self.path_ids(path, true, &mut ids);
        self.emit(
            ids,
            Change {
                namespace: true,
                ..Default::default()
            },
        );
    }
    pub(super) fn inode(&mut self, old: &Inode, new: &Inode, paths: Option<&BTreeSet<String>>) {
        let content = old.content != new.content
            || old.blob != new.blob
            || old.logical_size() != new.logical_size();
        // atime, mtime, link count and directory bookkeeping are not permission
        // changes; reads and unrelated directory entries must not wake readers.
        let metadata = old.kind != new.kind
            || old.mode != new.mode
            || old.uid != new.uid
            || old.gid != new.gid;
        if !content && !metadata {
            return;
        }
        let mut ids = self.inodes.get(&new.ino).cloned().unwrap_or_default();
        for path in paths.into_iter().flatten() {
            self.path_ids(path, metadata, &mut ids);
        }
        self.emit(
            ids,
            Change {
                content,
                metadata,
                namespace: false,
                truncated_to: (new.logical_size() < old.logical_size())
                    .then_some(new.logical_size()),
            },
        );
    }
}

impl VirtualFileSystem {
    pub fn subscribe(&mut self, target: WatchTarget) -> GameResult<WatchId> {
        self.nodes.events.subscribe(target)
    }
    pub fn watch_revision(&self, id: WatchId) -> Option<u64> {
        self.nodes.events.subscriptions.get(&id).map(|s| s.revision)
    }
    pub fn watch_event(&self, id: WatchId) -> Option<VfsEvent> {
        self.nodes
            .events
            .subscriptions
            .get(&id)
            .and_then(|s| s.pending)
    }
    pub fn acknowledge_watch(&mut self, id: WatchId, revision: u64) {
        if let Some(subscription) = self.nodes.events.subscriptions.get_mut(&id) {
            if subscription.pending.is_some_and(|e| e.revision <= revision) {
                subscription.pending = None;
            }
        }
    }
    pub fn unsubscribe(&mut self, id: WatchId) {
        self.nodes.events.unsubscribe(id);
    }
    pub fn watcher_count(&self) -> usize {
        self.nodes.events.subscriptions.len()
    }
    pub(crate) fn release_runtime_closed(&mut self, before: &Self, after: &Self) {
        for id in before
            .handles
            .keys()
            .filter(|id| !after.handles.contains_key(id))
        {
            self.handles.remove(id);
        }
        for id in before
            .nodes
            .events
            .subscriptions
            .keys()
            .filter(|id| !after.nodes.events.subscriptions.contains_key(id))
        {
            self.nodes.events.unsubscribe(*id);
        }
        self.collect();
    }
    /// End-of-session/load cancels processes before discarding runtime handles.
    pub(crate) fn reset_watches(&mut self) {
        self.nodes.events = EventBus {
            next: self.nodes.events.next,
            revision: self.nodes.events.revision,
            ..Default::default()
        };
    }
}
