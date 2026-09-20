use crate::{terminal, vfs::VirtualFileSystem, world::WorldState};

#[test]
fn mkdir_parents_is_idempotent_and_preserves_partial_effects() {
    let mut w = WorldState::new("kali", "lifeos").unwrap();
    w.vfs.write("/home/kali/file", "unchanged", "kali").unwrap();
    let first = terminal::execute(&mut w, "mkdir -pv a/b/c");
    assert_eq!(first.exit_code, 0);
    let inode = w.vfs.stat("/home/kali/a/b/c", "kali").unwrap().ino;
    let again = terminal::execute(&mut w, "mkdir -pm000 a/b/c");
    assert_eq!(again.exit_code, 0);
    let node = w.vfs.stat("/home/kali/a/b/c", "kali").unwrap();
    assert_eq!(node.ino, inode);
    assert_eq!(node.mode, 0o755);
    assert_eq!(
        terminal::execute(&mut w, "mkdir -p partial/../file/child later").exit_code,
        1
    );
    assert!(w.vfs.stat("/home/kali/partial", "kali").is_ok());
    assert!(w.vfs.stat("/home/kali/later", "kali").is_ok());
    assert_eq!(w.vfs.read("/home/kali/file", "kali").unwrap(), "unchanged");
    w.vfs.check_invariants().unwrap();
}

#[test]
fn evaluated_directory_mode_restores_umask_on_error() {
    let mut fs = VirtualFileSystem::default();
    fs.umask = 0o077;
    assert!(fs
        .mkdir_mode("/home/kali/missing/child", "kali", 0o777, None)
        .is_err());
    assert_eq!(fs.umask, 0o077);
    fs.mkdir("/home/kali/ordinary", "kali").unwrap();
    assert_eq!(fs.stat("/home/kali/ordinary", "kali").unwrap().mode, 0o700);
    fs.check_invariants().unwrap();
}

#[test]
fn mkdir_rmdir_share_directory_identity_and_link_counts() {
    let mut w = WorldState::new("kali", "lifeos").unwrap();
    let before = w.vfs.stat("/home/kali", "kali").unwrap().nlink;
    assert_eq!(terminal::execute(&mut w, "mkdir -p wave/a/b").exit_code, 0);
    assert_eq!(w.vfs.stat("/home/kali", "kali").unwrap().nlink, before + 1);
    w.vfs
        .symlink("/home/kali/alias", "wave/a/b", "kali")
        .unwrap();
    assert_eq!(terminal::execute(&mut w, "rmdir alias/").exit_code, 1);
    assert!(w.vfs.stat("/home/kali/wave/a/b", "kali").is_ok());
    assert_eq!(terminal::execute(&mut w, "rmdir -p wave/a/b").exit_code, 0);
    assert_eq!(w.vfs.stat("/home/kali", "kali").unwrap().nlink, before);
    assert_eq!(
        w.vfs.readlink("/home/kali/alias", "kali").unwrap(),
        "wave/a/b"
    );
    assert!(w.vfs.stat("/home/kali/alias", "kali").is_err());
    w.vfs.check_invariants().unwrap();
}
