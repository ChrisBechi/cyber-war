use crate::{
    terminal,
    vfs::{self, CanonicalMode, Errno, VirtualFileSystem},
    world::WorldState,
};

#[test]
fn canonical_names_preserve_symlink_semantics_and_are_idempotent() {
    let mut fs = VirtualFileSystem::default();
    fs.mkdir("/home/kali/a", "kali").unwrap();
    fs.mkdir("/home/kali/a/b", "kali").unwrap();
    fs.write("/home/kali/file", "content", "kali").unwrap();
    fs.symlink("/home/kali/dir", "a/b", "kali").unwrap();
    for raw in [
        "file",
        "./file",
        "a/../file",
        "dir/../../file",
        "missing/../file",
        "//dir///../b",
    ] {
        let absolute = vfs::absolute(raw, "/home/kali").unwrap();
        for mode in [
            CanonicalMode::Existing,
            CanonicalMode::AllButLast,
            CanonicalMode::Missing,
        ] {
            if let Ok(c) = fs.canonicalize(&absolute, "kali", mode, false) {
                assert_eq!(fs.canonicalize(&c, "kali", mode, false).unwrap(), c);
            }
        }
    }
    assert_eq!(
        fs.canonicalize("/home/kali/dir/..", "kali", CanonicalMode::Existing, false)
            .unwrap(),
        "/home/kali/a"
    );
    fs.check_invariants().unwrap();
}

#[test]
fn readlink_raw_roundtrip_and_canonical_cycle_traversal_are_bounded() {
    let mut w = WorldState::new("kali", "lifeos").unwrap();
    for (i, target) in [
        "missing",
        "./a/../b",
        "/absolute",
        "line\nname",
        "ação",
        "loop",
    ]
    .iter()
    .enumerate()
    {
        let p = format!("/home/kali/link{i}");
        w.vfs.symlink(&p, target, "kali").unwrap();
        let result = terminal::execute(&mut w, &format!("readlink -n {p}"));
        assert_eq!(result.stdout, *target);
        assert_eq!(result.exit_code, 0);
    }
    w.vfs
        .symlink("/home/kali/loop", "loop/child", "kali")
        .unwrap();
    assert!(matches!(
        w.vfs
            .canonicalize("/home/kali/loop", "kali", CanonicalMode::Existing, false),
        Err(crate::error::GameError::Vfs(Errno::Loop))
    ));
    assert!(w
        .vfs
        .canonicalize("/home/kali/loop", "kali", CanonicalMode::Missing, false)
        .is_ok());
    w.vfs.check_invariants().unwrap();
}

#[test]
fn virtual_path_and_component_boundaries_remain_explicit() {
    let fs = VirtualFileSystem::default();
    // Separators preserve byte length without requiring an enormous fixture.
    for len in [4095, 4096, 4097] {
        let p = "/".repeat(len);
        assert_eq!(
            fs.resolve(&p, "kali", vfs::Follow::Yes).is_ok(),
            len <= 4096
        );
        assert_eq!(
            fs.canonicalize(&p, "kali", CanonicalMode::Missing, false)
                .is_ok(),
            len <= 4096
        );
    }
    for len in [254, 255, 256] {
        let path = format!("/home/kali/{}", "x".repeat(len));
        let mut fs = fs.clone();
        assert_eq!(fs.mkdir(&path, "kali").is_ok(), len <= 255);
        fs.check_invariants().unwrap();
    }
}

#[test]
fn relative_paths_recompose_and_logical_mode_preserves_dotdot_order() {
    let mut w = WorldState::new("kali", "lifeos").unwrap();
    w.vfs.mkdir("/home/kali/a", "kali").unwrap();
    w.vfs.mkdir("/home/kali/a/b", "kali").unwrap();
    w.vfs.symlink("/home/kali/dir", "a/b", "kali").unwrap();
    let paths = [
        "/",
        "/home",
        "/home/kali",
        "/home/kali/a",
        "/home/kali/a/b",
        "/home/kali/other",
    ];
    for target in paths {
        for base in paths {
            let relative = vfs::relative_path(target, base);
            let recomposed = vfs::absolute(&relative, base).unwrap();
            assert_eq!(
                w.vfs
                    .canonicalize(&recomposed, "kali", CanonicalMode::Missing, true)
                    .unwrap(),
                target
            );
        }
    }
    let physical = terminal::execute(&mut w, "realpath dir/..");
    let logical = terminal::execute(&mut w, "realpath -L dir/..");
    assert_eq!(physical.stdout, "/home/kali/a\n");
    assert_eq!(logical.stdout, "/home/kali\n");
    assert_ne!(physical.stdout, logical.stdout);
    let relative = terminal::execute(&mut w, "realpath --relative-to=a dir");
    assert_eq!(relative.stdout, "b\n");
    assert!(!vfs::path_prefix("/home/kali/a", "/home/kali/ab"));
    w.vfs.check_invariants().unwrap();
}
