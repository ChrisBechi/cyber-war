use super::links::Links;
use crate::{terminal, vfs::VirtualFileSystem, world::WorldState};

#[test]
fn filesystem_wave_shares_inodes_alias_writes_and_symlink_resolution() {
    let mut w = WorldState::new("kali", "lifeos").unwrap();
    w.vfs
        .write("/home/kali/source", "original", "kali")
        .unwrap();
    for command in [
        "mkdir -p wave/empty",
        "ln source wave/hard",
        "ln -sr source wave/symbolic",
    ] {
        assert_eq!(terminal::execute(&mut w, command).exit_code, 0, "{command}");
    }
    let inode = w.vfs.stat("/home/kali/source", "kali").unwrap().ino;
    assert_eq!(
        w.vfs.stat("/home/kali/wave/hard", "kali").unwrap().ino,
        inode
    );
    assert_eq!(w.vfs.stat("/home/kali/source", "kali").unwrap().nlink, 2);
    w.vfs
        .write("/home/kali/wave/hard", "changed", "kali")
        .unwrap();
    assert_eq!(w.vfs.read("/home/kali/source", "kali").unwrap(), "changed");
    assert_eq!(
        terminal::execute(&mut w, "readlink wave/symbolic").stdout,
        "../source\n"
    );
    assert_eq!(
        terminal::execute(&mut w, "realpath wave/symbolic").stdout,
        "/home/kali/source\n"
    );
    assert_eq!(terminal::execute(&mut w, "rmdir wave/empty").exit_code, 0);
    w.vfs.unlink("/home/kali/source", "kali").unwrap();
    assert_eq!(
        w.vfs.read("/home/kali/wave/hard", "kali").unwrap(),
        "changed"
    );
    assert_eq!(w.vfs.stat("/home/kali/wave/hard", "kali").unwrap().nlink, 1);
    assert!(w.vfs.stat("/home/kali/wave/symbolic", "kali").is_err());
    w.vfs.check_invariants().unwrap();
}

#[test]
fn link_confirmation_emits_prompt_before_input_and_preserves_same_entry() {
    let mut w = WorldState::new("kali", "lifeos").unwrap();
    w.vfs.write("/home/kali/file", "payload", "kali").unwrap();
    let inode = w.vfs.stat("/home/kali/file", "kali").unwrap().ino;
    let args = ["-i", "file", "file"].map(String::from);
    let mut links = Links::new("ln", &args, false, &mut w).ok().unwrap();
    let prompt = links.advance(&mut w);
    assert!(!prompt.stderr.is_empty());
    assert!(links.waiting);
    assert_eq!(w.vfs.read("/home/kali/file", "kali").unwrap(), "payload");
    assert!(links.answer(&mut w).is_none());
    links.feed(b"y".to_vec());
    assert!(links.answer(&mut w).is_none());
    links.feed(b"\n".to_vec());
    assert_eq!(links.answer(&mut w).unwrap().status, 0);
    assert_eq!(w.vfs.stat("/home/kali/file", "kali").unwrap().ino, inode);
    assert_eq!(w.vfs.read("/home/kali/file", "kali").unwrap(), "payload");
    w.vfs.check_invariants().unwrap();
}

#[test]
fn symlink_text_survives_save_without_becoming_a_resolved_path() {
    let mut fs = VirtualFileSystem::default();
    for (i, target) in [
        "../missing".to_string(),
        "loop".into(),
        "x".repeat(256),
        "x".repeat(4095),
    ]
    .into_iter()
    .enumerate()
    {
        let path = format!("/home/kali/link{i}");
        fs.symlink(&path, &target, "kali").unwrap();
        let restored: VirtualFileSystem =
            serde_json::from_str(&serde_json::to_string(&fs).unwrap()).unwrap();
        assert_eq!(restored.readlink(&path, "kali").unwrap(), target);
        restored.check_invariants().unwrap();
    }
    assert!(fs
        .symlink("/home/kali/too-long", &"x".repeat(4096), "kali")
        .is_err());
    fs.check_invariants().unwrap();
}
