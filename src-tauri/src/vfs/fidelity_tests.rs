use super::*;
fn p(name: &str) -> String {
    format!("{HOME}/{name}")
}
fn errno<T: std::fmt::Debug>(value: GameResult<T>, expected: Errno) {
    assert!(
        matches!(value,Err(GameError::Vfs(e)) if e==expected),
        "expected {expected:?}, got {value:?}"
    );
}
#[test]
fn inode_identity_links_rename_copy_and_snapshot_are_consistent() {
    let mut fs = VirtualFileSystem::default();
    fs.write(&p("a"), "A", "kali").unwrap();
    let ino = fs.stat(&p("a"), "kali").unwrap().ino;
    fs.link(&p("a"), &p("b"), "kali").unwrap();
    assert!(Arc::ptr_eq(
        &fs.nodes[&p("a")].inode,
        &fs.nodes[&p("b")].inode
    ));
    assert_eq!(fs.stat(&p("b"), "kali").unwrap().nlink, 2);
    fs.write(&p("b"), "changed", "kali").unwrap();
    assert_eq!(fs.read(&p("a"), "kali").unwrap(), "changed");
    fs.rename_entry(&p("a"), &p("renamed"), "kali").unwrap();
    assert_eq!(fs.stat(&p("renamed"), "kali").unwrap().ino, ino);
    fs.copy_entry(&p("b"), &p("copy"), "kali").unwrap();
    assert_ne!(fs.stat(&p("copy"), "kali").unwrap().ino, ino);
    fs.unlink(&p("renamed"), "kali").unwrap();
    assert_eq!(fs.stat(&p("b"), "kali").unwrap().nlink, 1);
    fs.check_invariants().unwrap();
    let json = serde_json::to_string(&fs).unwrap();
    let restored: VirtualFileSystem = serde_json::from_str(&json).unwrap();
    restored.check_invariants().unwrap();
    assert_eq!(restored.stat(&p("b"), "kali").unwrap().ino, ino);
}
#[test]
fn semantic_symlink_traversal_and_dotdot() {
    let mut fs = VirtualFileSystem::default();
    fs.mkdir(&p("a"), "kali").unwrap();
    fs.mkdir(&p("b"), "kali").unwrap();
    fs.mkdir(&p("b/deep"), "kali").unwrap();
    fs.write(&p("b/file"), "target", "kali").unwrap();
    fs.symlink(&p("a/link"), "../b/deep", "kali").unwrap();
    assert_eq!(fs.read(&p("a/link/../file"), "kali").unwrap(), "target");
    assert_eq!(fs.readlink(&p("a/link"), "kali").unwrap(), "../b/deep");
    assert_eq!(fs.lstat(&p("a/link"), "kali").unwrap().kind, "symlink");
    assert_eq!(fs.stat(&p("a/link"), "kali").unwrap().kind, "directory");
    fs.symlink(&p("dangling"), "missing", "kali").unwrap();
    errno(fs.stat(&p("dangling"), "kali"), Errno::NotFound);
    fs.write(&p("dangling"), "created target", "kali").unwrap();
    assert_eq!(fs.read(&p("missing"), "kali").unwrap(), "created target");
    fs.symlink(&p("cycle-a"), "cycle-b", "kali").unwrap();
    fs.symlink(&p("cycle-b"), "cycle-a", "kali").unwrap();
    errno(fs.read(&p("cycle-a"), "kali"), Errno::Loop);
    errno(fs.read(&p("b/file/"), "kali"), Errno::NotDirectory);
    errno(fs.read(&p("b/file/.."), "kali"), Errno::NotDirectory);
    assert_eq!(fs.resolve("/../../", "root", Follow::Yes).unwrap(), "/");
    fs.check_invariants().unwrap();
}
#[test]
fn permissions_search_read_sticky_umask_and_setgid() {
    let mut fs = VirtualFileSystem::default();
    fs.mkdir(&p("dir"), "kali").unwrap();
    fs.write(&p("dir/file"), "data", "kali").unwrap();
    fs.chmod(&p("dir"), "kali", 0o400).unwrap();
    assert_eq!(fs.child_names(&p("dir"), "kali").unwrap(), ["file"]);
    errno(fs.read(&p("dir/file"), "kali"), Errno::Access);
    fs.chmod(&p("dir"), "kali", 0o100).unwrap();
    assert_eq!(fs.read(&p("dir/file"), "kali").unwrap(), "data");
    errno(fs.child_names(&p("dir"), "kali"), Errno::Access);
    fs.write("/tmp/alice", "a", "alice").unwrap();
    errno(fs.unlink("/tmp/alice", "bob"), Errno::NotPermitted);
    fs.unlink("/tmp/alice", "alice").unwrap();
    fs.umask = 0o077;
    fs.write(&p("private"), "", "kali").unwrap();
    fs.mkdir(&p("private-dir"), "kali").unwrap();
    assert_eq!(fs.stat(&p("private"), "kali").unwrap().mode, 0o600);
    assert_eq!(fs.stat(&p("private-dir"), "kali").unwrap().mode, 0o700);
    fs.chmod(&p("private-dir"), "kali", 0o2770).unwrap();
    fs.ownership(&p("private-dir"), "root", None, Some("vex"), Follow::Yes)
        .unwrap();
    fs.chmod(&p("private-dir"), "root", 0o2770).unwrap();
    fs.mkdir(&p("private-dir/child"), "kali").unwrap();
    let child = fs.stat(&p("private-dir/child"), "kali").unwrap();
    assert_eq!(child.gid, 1001);
    assert_eq!(child.mode, 0o2700);
    fs.chmod(&p("private"), "kali", 0).unwrap();
    assert!(!fs.allowed(fs.stat(&p("private"), "root").unwrap(), "root", 1));
    fs.check_invariants().unwrap();
}
#[test]
fn supplementary_groups_and_shared_metadata() {
    let mut fs = VirtualFileSystem::default();
    fs.write("/tmp/group-file", "data", "root").unwrap();
    fs.ownership("/tmp/group-file", "root", None, Some("vex"), Follow::Yes)
        .unwrap();
    fs.chmod("/tmp/group-file", "root", 0o640).unwrap();
    fs.set_identity(
        "reader",
        Identity {
            uid: 2000,
            gid: 2000,
            groups: BTreeSet::from([1001]),
        },
    );
    assert_eq!(fs.read("/tmp/group-file", "reader").unwrap(), "data");
    errno(fs.read("/tmp/group-file", "kali"), Errno::Access);
    fs.link("/tmp/group-file", "/tmp/alias", "root").unwrap();
    fs.chmod("/tmp/alias", "root", 0o644).unwrap();
    assert_eq!(fs.stat("/tmp/group-file", "root").unwrap().mode, 0o644);
    fs.check_invariants().unwrap();
}
#[test]
fn handles_offsets_append_unlink_and_devices() {
    let mut fs = VirtualFileSystem::default();
    fs.write(&p("data"), "abcd", "kali").unwrap();
    let flags = OpenFlags {
        read: true,
        write: true,
        ..Default::default()
    };
    let a = fs.open(&p("data"), flags, 0o666, "kali").unwrap();
    let b = fs.open(&p("data"), flags, 0o666, "kali").unwrap();
    assert_eq!(fs.read_handle(a, 2).unwrap(), b"ab");
    assert_eq!(fs.read_handle(b, 1).unwrap(), b"a");
    fs.write_handle(b, b"X").unwrap();
    assert_eq!(fs.read(&p("data"), "kali").unwrap(), "aXcd");
    let append = fs
        .open(
            &p("data"),
            OpenFlags {
                write: true,
                append: true,
                ..Default::default()
            },
            0,
            "kali",
        )
        .unwrap();
    fs.seek(append, 0).unwrap();
    fs.write_handle(append, b"!").unwrap();
    assert_eq!(fs.read(&p("data"), "kali").unwrap(), "aXcd!");
    fs.unlink(&p("data"), "kali").unwrap();
    fs.write_handle(append, b"?").unwrap();
    assert_eq!(fs.read_handle(a, 20).unwrap(), b"cd!?");
    fs.check_invariants().unwrap();
    fs.close(a).unwrap();
    fs.close(b).unwrap();
    fs.close(append).unwrap();
    fs.check_invariants().unwrap();
    let null = fs.open("/dev/null", flags, 0, "kali").unwrap();
    assert_eq!(fs.write_handle(null, b"discard").unwrap(), 7);
    assert!(fs.read_handle(null, 100).unwrap().is_empty());
    fs.close(null).unwrap();
    let zero = fs
        .open(
            "/dev/zero",
            OpenFlags {
                read: true,
                ..Default::default()
            },
            0,
            "kali",
        )
        .unwrap();
    assert_eq!(fs.read_handle(zero, 17).unwrap(), vec![0; 17]);
    fs.close(zero).unwrap();
}
#[test]
fn migration_preserves_legacy_metadata_and_sharing_roundtrip() {
    let mut fs = VirtualFileSystem::default();
    fs.write(&p("big"), &"x".repeat(500_000), "kali").unwrap();
    fs.symlink(&p("link"), "no-such-target", "kali").unwrap();
    fs.chmod(&p("big"), "kali", 0o640).unwrap();
    let legacy =
        serde_json::json!({"nodes":fs.nodes,"clock":fs.clock,"capacity_bytes":fs.capacity_bytes});
    let mut migrated: VirtualFileSystem = serde_json::from_value(legacy).unwrap();
    migrated.check_invariants().unwrap();
    assert_eq!(migrated.stat(&p("big"), "kali").unwrap().mode, 0o640);
    assert_eq!(
        migrated.readlink(&p("link"), "kali").unwrap(),
        "no-such-target"
    );
    let old = migrated.stat(&p("big"), "kali").unwrap().ino;
    migrated.link(&p("big"), &p("alias"), "kali").unwrap();
    let json = serde_json::to_string(&migrated).unwrap();
    assert!(json.len() < 600_000);
    let mut restored: VirtualFileSystem = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.stat(&p("alias"), "kali").unwrap().ino, old);
    restored.write(&p("alias"), "ok", "kali").unwrap();
    assert_eq!(restored.read(&p("big"), "kali").unwrap(), "ok");
    restored.check_invariants().unwrap();
}
#[test]
fn rename_atomicity_and_linux_names() {
    let mut fs = VirtualFileSystem::default();
    fs.mkdir(&p("dir"), "kali").unwrap();
    fs.mkdir(&p("dir/sub"), "kali").unwrap();
    fs.write(&p("dir/sub/file"), "data", "kali").unwrap();
    let ino = fs.stat(&p("dir/sub/file"), "kali").unwrap().ino;
    let before = fs.clone();
    assert!(fs
        .rename_entry(&p("dir"), &p("dir/sub/invalid"), "kali")
        .is_err());
    assert_eq!(fs, before);
    fs.rename_entry(&p("dir"), &p("new"), "kali").unwrap();
    assert_eq!(fs.stat(&p("new/sub/file"), "kali").unwrap().ino, ino);
    for name in [
        "File",
        "file",
        "FILE",
        "foo:bar",
        "star*",
        "question?",
        "CON",
        "NUL",
        r"C:\Windows\System32",
        r"\\server\share",
        "a\nb",
    ] {
        fs.write(&p(name), name, "kali").unwrap();
        assert_eq!(fs.read(&p(name), "kali").unwrap(), name);
    }
    errno(fs.write(&p("bad\0name"), "", "kali"), Errno::Invalid);
    fs.check_invariants().unwrap();
}
