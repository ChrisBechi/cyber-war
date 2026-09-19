use super::*;
fn path(name: &str) -> String {
    format!("{HOME}/{name}")
}

#[test]
fn subscriptions_filter_alias_writes_and_acknowledgement_cannot_lose_a_wakeup() {
    let mut fs = VirtualFileSystem::default();
    fs.write(&path("a"), "start", "kali").unwrap();
    fs.link(&path("a"), &path("alias"), "kali").unwrap();
    let ino = fs.stat(&path("a"), "kali").unwrap().ino;
    let inode = fs.subscribe(WatchTarget::Inode(ino)).unwrap();
    let name = fs.subscribe(WatchTarget::Path(path("a"))).unwrap();
    let peer = fs.subscribe(WatchTarget::Inode(ino)).unwrap();
    let revision = fs.watch_revision(inode).unwrap();
    fs.write(&path("unrelated"), "other", "kali").unwrap();
    assert_eq!(fs.watch_revision(inode), Some(revision));
    assert!(fs.watch_event(name).is_none());
    fs.write(&path("alias"), "first", "kali").unwrap();
    let first = fs.watch_event(inode).unwrap();
    assert!(first.change.content);
    assert_eq!(fs.watch_event(name), Some(first));
    assert_eq!(fs.watch_event(peer), Some(first));
    fs.write(&path("a"), "second", "kali").unwrap();
    fs.acknowledge_watch(inode, first.revision);
    let second = fs.watch_event(inode).unwrap();
    assert!(second.revision > first.revision);
    fs.acknowledge_watch(inode, second.revision);
    assert!(fs.watch_event(inode).is_none());
    assert!(fs.watch_event(peer).is_some());
    fs.read(&path("a"), "kali").unwrap();
    assert!(
        fs.watch_event(inode).is_none(),
        "atime must not wake a content observer"
    );
    for watch in [inode, name, peer] {
        fs.unsubscribe(watch);
    }
    assert_eq!(fs.watcher_count(), 0);
}

#[test]
fn namespace_replacement_and_unlinked_descriptors_keep_distinct_identity() {
    let mut fs = VirtualFileSystem::default();
    let name = fs.subscribe(WatchTarget::Path(path("log"))).unwrap();
    fs.write(&path("log"), "old", "kali").unwrap();
    let old = fs.stat(&path("log"), "kali").unwrap().ino;
    let reader = fs
        .open(
            &path("log"),
            OpenFlags {
                read: true,
                ..Default::default()
            },
            0,
            "kali",
        )
        .unwrap();
    let writer = fs
        .open(
            &path("log"),
            OpenFlags {
                write: true,
                append: true,
                ..Default::default()
            },
            0,
            "kali",
        )
        .unwrap();
    let inode = fs.subscribe(WatchTarget::Inode(old)).unwrap();
    fs.write(&path("temp"), "new", "kali").unwrap();
    fs.rename_entry(&path("temp"), &path("log"), "kali")
        .unwrap();
    assert_ne!(fs.stat(&path("log"), "kali").unwrap().ino, old);
    assert!(fs.watch_event(name).unwrap().change.namespace);
    assert!(fs.watch_event(inode).is_none());
    fs.write_handle(writer, b"!").unwrap();
    assert!(fs.watch_event(inode).unwrap().change.content);
    assert_eq!(fs.read_handle(reader, 9).unwrap(), b"old!");
    assert_eq!(fs.read(&path("log"), "kali").unwrap(), "new");
    fs.close(reader).unwrap();
    fs.close(writer).unwrap();
    fs.collect();
    assert!(
        !fs.nodes.inodes.contains_key(&old),
        "a watch must not pin an orphan inode"
    );
    fs.unsubscribe(name);
    fs.unsubscribe(inode);
    fs.check_invariants().unwrap();
}

#[test]
fn truncate_append_and_permission_changes_coalesce_with_bounded_storage() {
    let mut fs = VirtualFileSystem::default();
    fs.mkdir(&path("watched"), "kali").unwrap();
    let target = path("watched/log");
    fs.write(&target, &"x".repeat(100), "kali").unwrap();
    let watch = fs.subscribe(WatchTarget::Path(target.clone())).unwrap();
    fs.truncate(&target, 20, "kali").unwrap();
    fs.truncate(&target, 0, "kali").unwrap();
    let writer = fs
        .open(
            &target,
            OpenFlags {
                write: true,
                append: true,
                ..Default::default()
            },
            0,
            "kali",
        )
        .unwrap();
    for _ in 0..1000 {
        fs.write_handle(writer, b"a").unwrap();
    }
    let event = fs.watch_event(watch).unwrap();
    assert_eq!(event.change.truncated_to, Some(0));
    assert_eq!(fs.read(&target, "kali").unwrap(), "a".repeat(1000));
    assert_eq!(fs.watcher_count(), 1);
    fs.acknowledge_watch(watch, event.revision);
    fs.chmod(&path("watched"), "kali", 0o600).unwrap();
    assert!(fs.watch_event(watch).unwrap().change.metadata);
    fs.close(writer).unwrap();
    fs.unsubscribe(watch);
    assert_eq!(fs.open_handle_count(), 0);
}

#[test]
fn subscriptions_are_runtime_only_and_limits_are_explicit() {
    let mut fs = VirtualFileSystem::default();
    let watch = fs.subscribe(WatchTarget::Path(path("missing"))).unwrap();
    fs.write(&path("missing"), "save", "kali").unwrap();
    let restored: VirtualFileSystem =
        serde_json::from_str(&serde_json::to_string(&fs).unwrap()).unwrap();
    assert_eq!(restored.watcher_count(), 0);
    assert!(restored.watch_revision(watch).is_none());
    assert_eq!(restored.read(&path("missing"), "kali").unwrap(), "save");
    let mut ids = vec![watch];
    for _ in 1..events::WATCH_LIMIT {
        ids.push(fs.subscribe(WatchTarget::Path(path("missing"))).unwrap());
    }
    assert!(matches!(
        fs.subscribe(WatchTarget::Inode(1)),
        Err(GameError::Vfs(Errno::NoSpace))
    ));
    for id in ids {
        fs.unsubscribe(id);
    }
    assert_eq!(fs.watcher_count(), 0);
}

#[test]
fn deterministic_mutation_sequence_preserves_revisions_bounds_and_no_leaks() {
    fn sequence() -> (String, Vec<VfsEvent>) {
        let mut fs = VirtualFileSystem::default();
        let watch = fs.subscribe(WatchTarget::Path(path("a"))).unwrap();
        let mut events = Vec::new();
        for round in 0..100 {
            fs.write(&path("a"), "original", "kali").unwrap();
            fs.write(&path("temp"), &round.to_string(), "kali").unwrap();
            fs.rename_entry(&path("temp"), &path("a"), "kali").unwrap();
            fs.truncate(&path("a"), 0, "kali").unwrap();
            fs.unlink(&path("a"), "kali").unwrap();
            let event = fs.watch_event(watch).unwrap();
            assert!(events
                .last()
                .is_none_or(|old: &VfsEvent| old.revision < event.revision));
            events.push(event);
            fs.acknowledge_watch(watch, event.revision);
            assert!(fs.watch_event(watch).is_none());
            fs.check_invariants().unwrap();
        }
        fs.unsubscribe(watch);
        assert_eq!(fs.watcher_count(), 0);
        (serde_json::to_string(&fs).unwrap(), events)
    }
    assert_eq!(sequence(), sequence());
}

#[test]
fn lookup_watches_trace_symlinks_missing_components_and_permission_failures() {
    let mut fs = VirtualFileSystem::default();
    fs.mkdir(&path("dir"), "kali").unwrap();
    fs.mkdir(&path("dir/sub"), "kali").unwrap();
    fs.write(&path("dir/log"), "text", "kali").unwrap();
    fs.symlink(&path("link"), "dir/sub", "kali").unwrap();
    let dependencies = fs.lookup_dependencies(&path("link/../log"), "kali");
    assert!(dependencies.contains(&path("link")));
    assert!(dependencies.contains(&path("dir/log")));
    assert!(!dependencies.contains(&path("log")));
    let missing = fs.lookup_dependencies(&path("dir/missing/log"), "kali");
    assert!(missing.contains(&path("dir/missing")));
    let watch = fs
        .subscribe(WatchTarget::Path(path("dir/missing/log")))
        .unwrap();
    fs.mkdir(&path("dir/missing"), "kali").unwrap();
    assert!(fs.watch_event(watch).unwrap().change.namespace);
    fs.acknowledge_watch(watch, fs.watch_revision(watch).unwrap());
    fs.chmod(&path("dir"), "kali", 0o000).unwrap();
    assert!(fs.watch_event(watch).unwrap().change.metadata);
    let blocked = fs.lookup_dependencies(&path("link/../log"), "kali");
    assert!(blocked.contains(&path("dir")));
    fs.chmod(&path("dir"), "kali", 0o755).unwrap();
    fs.unsubscribe(watch);
    assert_eq!(fs.watcher_count(), 0);
}

#[test]
fn binary_truncate_preserves_inode_and_coalesces_the_lowest_size() {
    let mut fs = VirtualFileSystem::default();
    let mut blobs = crate::binary::BlobCache::default();
    let file = path("binary");
    let writer = fs
        .open(
            &file,
            OpenFlags {
                write: true,
                create: true,
                append: true,
                ..Default::default()
            },
            0o600,
            "kali",
        )
        .unwrap();
    fs.write_handle_bytes(writer, &[0, 255, 128, 10], &mut blobs)
        .unwrap();
    let ino = fs.stat(&file, "kali").unwrap().ino;
    let watch = fs.subscribe(WatchTarget::Inode(ino)).unwrap();
    fs.truncate_bytes(&file, 2, "kali", &mut blobs).unwrap();
    fs.truncate_bytes(&file, 0, "kali", &mut blobs).unwrap();
    fs.write_handle_bytes(writer, &[255, 0, 128, 10, 255], &mut blobs)
        .unwrap();
    assert_eq!(fs.stat(&file, "kali").unwrap().ino, ino);
    assert_eq!(fs.watch_event(watch).unwrap().change.truncated_to, Some(0));
    let reader = fs
        .open(
            &file,
            OpenFlags {
                read: true,
                ..Default::default()
            },
            0,
            "kali",
        )
        .unwrap();
    assert_eq!(
        fs.read_handle_bytes(reader, 20, &blobs).unwrap(),
        [255, 0, 128, 10, 255]
    );
    fs.close(reader).unwrap();
    fs.close(writer).unwrap();
    fs.unsubscribe(watch);
    fs.check_invariants().unwrap();
    assert_eq!(fs.open_handle_count(), 0);
    assert_eq!(fs.watcher_count(), 0);
}

#[test]
fn byte_file_limit_is_atomic_at_each_boundary() {
    let mut fs = VirtualFileSystem::default();
    let mut blobs = crate::binary::BlobCache::default();
    let file = path("byte-limit");
    fs.write(&file, "", "kali").unwrap();
    let watch = fs.subscribe(WatchTarget::Path(file.clone())).unwrap();
    let limit = crate::binary::MAX_BLOB;
    for size in [limit - 1, limit] {
        fs.truncate_bytes(&file, size, "kali", &mut blobs).unwrap();
        assert_eq!(fs.stat(&file, "kali").unwrap().logical_size(), size as u64);
    }
    let revision = fs.watch_revision(watch).unwrap();
    assert!(fs
        .truncate_bytes(&file, limit + 1, "kali", &mut blobs)
        .is_err());
    assert_eq!(fs.watch_revision(watch), Some(revision));
    assert_eq!(fs.stat(&file, "kali").unwrap().logical_size(), limit as u64);
    fs.unsubscribe(watch);
    assert_eq!(fs.watcher_count(), 0);
    assert_eq!(fs.open_handle_count(), 0);
    fs.check_invariants().unwrap();
}
