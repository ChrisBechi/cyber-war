use super::*;
use crate::{terminal, world::WorldState};
fn p(name: &str) -> String {
    format!("{HOME}/{name}")
}
#[test]
fn random_sequences_preserve_content_identity_and_link_counts() {
    let mut fs = VirtualFileSystem::default();
    let mut model = BTreeMap::<String, (u64, String)>::new();
    let mut seed = 0x41dce7u64;
    let mut next = 1;
    for step in 0..2000 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let a = p(&format!("f{}", (seed >> 32) % 32));
        let b = p(&format!("f{}", (seed >> 16) % 32));
        match seed % 5 {
            0 => {
                let text = format!("data-{step}");
                fs.write(&a, &text, "kali").unwrap();
                if let Some((id, _)) = model.get(&a).cloned() {
                    for (ino, value) in model.values_mut() {
                        if *ino == id {
                            *value = text.clone();
                        }
                    }
                } else {
                    model.insert(a, (next, text));
                    next += 1;
                }
            }
            1 if model.contains_key(&a) && !model.contains_key(&b) => {
                fs.link(&a, &b, "kali").unwrap();
                model.insert(b, model[&a].clone());
            }
            2 if model.contains_key(&a) && !model.contains_key(&b) => {
                fs.rename_entry(&a, &b, "kali").unwrap();
                let value = model.remove(&a).unwrap();
                model.insert(b, value);
            }
            3 if model.contains_key(&a) => {
                fs.unlink(&a, "kali").unwrap();
                model.remove(&a);
            }
            4 if model.contains_key(&a) && !model.contains_key(&b) => {
                fs.copy_entry(&a, &b, "kali").unwrap();
                model.insert(b, (next, model[&a].1.clone()));
                next += 1;
            }
            _ => {}
        }
        fs.check_invariants().unwrap();
        let mut identities = BTreeMap::new();
        for (path, (id, text)) in &model {
            let node = fs.stat(path, "kali").unwrap();
            assert_eq!(&node.content, text);
            assert_eq!(
                node.nlink,
                model.values().filter(|(i, _)| i == id).count() as u64
            );
            if let Some(previous) = identities.insert(*id, node.ino) {
                assert_eq!(previous, node.ino);
            }
        }
        assert_eq!(
            identities.values().copied().collect::<BTreeSet<_>>().len(),
            identities.len()
        );
        if step % 97 == 0 {
            fs = serde_json::from_str(&serde_json::to_string(&fs).unwrap()).unwrap();
        }
    }
}
#[test]
fn permission_matrix_owner_group_other_and_root_execute() {
    let mut fs = VirtualFileSystem::default();
    fs.write(&p("matrix"), "x", "kali").unwrap();
    fs.set_identity(
        "member",
        Identity {
            uid: 2000,
            gid: 2000,
            groups: BTreeSet::from([1000]),
        },
    );
    for mode in 0..=0o777 {
        fs.chmod(&p("matrix"), "kali", mode).unwrap();
        let n = fs.stat(&p("matrix"), "kali").unwrap();
        for (actor, shift) in [("kali", 6), ("member", 3), ("vex", 0)] {
            for bits in 1..8 {
                assert_eq!(fs.allowed(n, actor, bits), ((mode >> shift) & bits) == bits);
            }
        }
        assert!(fs.allowed(n, "root", 6));
        assert_eq!(fs.allowed(n, "root", 1), mode & 0o111 != 0);
    }
}
#[test]
fn device_aliases_handles_limits_and_failed_operations_are_atomic() {
    let mut fs = VirtualFileSystem::default();
    fs.link("/dev/null", &p("null"), "kali").unwrap();
    fs.link("/dev/zero", &p("zero"), "kali").unwrap();
    let h = fs
        .open(
            &p("zero"),
            OpenFlags {
                read: true,
                ..Default::default()
            },
            0,
            "kali",
        )
        .unwrap();
    assert_eq!(fs.read_handle(h, 7).unwrap(), vec![0; 7]);
    fs.close(h).unwrap();
    assert_eq!(fs.read(&p("null"), "kali").unwrap(), "");
    let h = fs
        .open(
            &p("closed-mode"),
            OpenFlags {
                write: true,
                create: true,
                truncate: true,
                ..Default::default()
            },
            0,
            "kali",
        )
        .unwrap();
    fs.write_handle(h, b"allowed through open handle").unwrap();
    fs.close(h).unwrap();
    let before = fs.clone();
    assert!(fs
        .open(
            &p("closed-mode"),
            OpenFlags {
                write: true,
                ..Default::default()
            },
            0,
            "kali"
        )
        .is_err());
    assert_eq!(fs, before);
    let mut handles = Vec::new();
    for _ in 0..1024 {
        handles.push(
            fs.open(
                "/dev/null",
                OpenFlags {
                    write: true,
                    ..Default::default()
                },
                0,
                "kali",
            )
            .unwrap(),
        );
    }
    let before = fs.clone();
    assert!(fs
        .open(
            &p("uncreated"),
            OpenFlags {
                write: true,
                create: true,
                ..Default::default()
            },
            0o666,
            "kali"
        )
        .is_err());
    assert_eq!(fs, before);
    for h in handles {
        fs.close(h).unwrap();
    }
    fs.read_only = true;
    let before = fs.clone();
    assert!(fs.write(&p("null"), "x", "root").is_err());
    assert_eq!(fs, before);
}
#[test]
fn capacity_includes_unlinked_open_inode_and_timestamps() {
    let mut fs = VirtualFileSystem::default();
    fs.write(&p("a"), "0123456789", "kali").unwrap();
    let h = fs
        .open(
            &p("a"),
            OpenFlags {
                read: true,
                write: true,
                ..Default::default()
            },
            0,
            "kali",
        )
        .unwrap();
    let before = fs.stat(&p("a"), "kali").unwrap().clone();
    fs.read_handle(h, 1).unwrap();
    assert!(fs.stat(&p("a"), "kali").unwrap().accessed_at > before.accessed_at);
    assert_eq!(
        fs.stat(&p("a"), "kali").unwrap().modified_at,
        before.modified_at
    );
    let used = fs.used_bytes();
    fs.capacity_bytes = used;
    fs.unlink(&p("a"), "kali").unwrap();
    assert_eq!(fs.used_bytes(), used);
    assert!(fs.write(&p("b"), "x", "kali").is_err());
    fs.close(h).unwrap();
    assert_eq!(fs.used_bytes(), used - 10);
    fs.write(&p("b"), "x", "kali").unwrap();
    fs.check_invariants().unwrap();
}
#[test]
fn journal_restoration_keeps_shared_identity_and_metadata() {
    let mut fs = VirtualFileSystem::default();
    fs.write(&p("a"), "before", "kali").unwrap();
    fs.link(&p("a"), &p("alias"), "kali").unwrap();
    let before = fs.lstat(&p("a"), "kali").unwrap().clone();
    fs.write(&p("alias"), "temporary", "kali").unwrap();
    fs.restore_entry(&p("a"), Some(before.clone()));
    fs.collect();
    fs.check_invariants().unwrap();
    assert_eq!(fs.read(&p("alias"), "kali").unwrap(), "before");
    assert_eq!(fs.stat(&p("alias"), "kali").unwrap().ino, before.ino);
}
#[test]
fn shared_shell_ui_projection_and_five_slots() {
    let mut game =
        crate::service::GameService::new(rusqlite::Connection::open_in_memory().unwrap()).unwrap();
    let mut inodes = Vec::new();
    for slot in 1..=5 {
        game.new_game(slot, "kali", "lifeos", false).unwrap();
        game.mutate(
            |_, w, _| {
                let output = terminal::execute(
                    w,
                    "echo original > a; ln a alias; ln -s a shortcut; echo edited > alias",
                );
                assert_eq!(output.exit_code, 0, "{}", output.stderr);
                assert_eq!(
                    w.vfs
                        .list(HOME, "kali")?
                        .iter()
                        .find(|n| n.name == "a")
                        .unwrap()
                        .content,
                    "edited\n"
                );
                w.vfs.write(&p("slot"), &slot.to_string(), "kali")?;
                Ok(())
            },
            true,
        )
        .unwrap();
        inodes.push(game.world().unwrap().vfs.stat(&p("a"), "kali").unwrap().ino);
    }
    for slot in 1..=5 {
        game.load(slot, false).unwrap();
        let w = game.world().unwrap();
        w.vfs.check_invariants().unwrap();
        assert_eq!(w.vfs.read(&p("slot"), "kali").unwrap(), slot.to_string());
        assert_eq!(w.vfs.read(&p("shortcut"), "kali").unwrap(), "edited\n");
        assert_eq!(
            w.vfs.stat(&p("alias"), "kali").unwrap().ino,
            inodes[(slot - 1) as usize]
        );
    }
}
#[test]
fn resolver_fuzz_and_snapshot_corruption_are_bounded() {
    let fs = VirtualFileSystem::default();
    let alphabet = [
        "/", ".", "..", "home", "kali", "missing", "file:", "\\\\", "\0", "é",
    ];
    let mut seed = 17u64;
    for _ in 0..1500 {
        let mut path = String::new();
        for _ in 0..24 {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            path.push_str(alphabet[(seed as usize) % alphabet.len()]);
            path.push('/');
        }
        let _ = fs.resolve(&path, "kali", Follow::Yes);
        fs.check_invariants().unwrap();
    }
    let snapshot = serde_json::to_value(&fs).unwrap();
    for mutation in 0..4 {
        let mut value = snapshot.clone();
        match mutation {
            0 => value["entries"]["/home/kali"] = serde_json::json!(u64::MAX),
            1 => value["nextInode"] = serde_json::json!(1),
            2 => value["formatVersion"] = serde_json::json!(99),
            _ => {
                value["inodes"].as_object_mut().unwrap().remove("1");
            }
        }
        assert!(serde_json::from_value::<VirtualFileSystem>(value).is_err());
    }
}
#[test]
fn renamed_cwd_and_remote_inode_domains_remain_independent() {
    let mut w = WorldState::new("kali", "lifeos").unwrap();
    crate::terminal_sessions::open_at(&mut w, "other", Some(HOME), false).unwrap();
    let out = terminal::execute(
        &mut w,
        "mkdir dir; cd dir; mv /home/kali/dir /home/kali/new; pwd",
    );
    assert_eq!(out.stdout, "/home/kali/new\n");
    assert_eq!(out.exit_code, 0, "{}", out.stderr);
    let local = w.vfs.clone();
    for host in w.network.hosts.values_mut() {
        if host.files.nodes.contains_key(HOME) {
            host.files.write(&p("remote"), "remote", "root").unwrap();
        }
    }
    assert_eq!(w.vfs, local);
    w.validate().unwrap();
}

#[test]
fn trash_tracks_directory_entries_independently_for_hardlinks() {
    let mut fs = VirtualFileSystem::default();
    fs.write(&p("a"), "data", "kali").unwrap();
    fs.link(&p("a"), &p("b"), "kali").unwrap();
    let ino = fs.stat(&p("a"), "kali").unwrap().ino;
    fs.move_to_trash(&p("a"), "kali", false).unwrap();
    fs.move_to_trash(&p("b"), "kali", false).unwrap();
    let entries = fs.list(TRASH_FILES, "kali").unwrap();
    for n in entries {
        assert_eq!(n.metadata["trashOriginalPath"], p(&n.name));
    }
    let wire = serde_json::to_string(&fs).unwrap();
    fs = serde_json::from_str(&wire).unwrap();
    fs.restore_from_trash(&format!("{TRASH_FILES}/a"), "kali")
        .unwrap();
    fs.restore_from_trash(&format!("{TRASH_FILES}/b"), "kali")
        .unwrap();
    assert_eq!(fs.stat(&p("a"), "kali").unwrap().ino, ino);
    assert_eq!(fs.stat(&p("b"), "kali").unwrap().ino, ino);
    assert_eq!(fs.stat(&p("a"), "kali").unwrap().nlink, 2);
    fs.check_invariants().unwrap();
}
#[test]
fn seek_beyond_eof_does_not_reset_offset_and_chmod_links_terminate() {
    let mut fs = VirtualFileSystem::default();
    fs.write(&p("a"), "abc", "kali").unwrap();
    let h = fs
        .open(
            &p("a"),
            OpenFlags {
                read: true,
                write: true,
                ..Default::default()
            },
            0,
            "kali",
        )
        .unwrap();
    fs.seek(h, 5).unwrap();
    assert!(fs.read_handle(h, 2).unwrap().is_empty());
    fs.write_handle(h, b"z").unwrap();
    fs.close(h).unwrap();
    assert_eq!(fs.read(&p("a"), "kali").unwrap(), "abc\0\0z");
    let mut w = WorldState::new("kali", "lifeos").unwrap();
    let result = terminal::execute(&mut w, "mkdir dir; ln -s . dir/loop; chmod -R 750 dir");
    assert_eq!(result.exit_code, 0, "{}", result.stderr);
}

#[test]
fn shell_umasks_do_not_leak_into_other_terminals_or_desktop() {
    let mut w = WorldState::new("kali", "lifeos").unwrap();
    crate::terminal_sessions::open(&mut w, "A").unwrap();
    crate::terminal_sessions::open(&mut w, "B").unwrap();
    crate::terminal_sessions::with_session(&mut w, Some("A"), |w| {
        assert_eq!(
            terminal::execute(w, "umask 077; touch private").exit_code,
            0
        );
        Ok(())
    })
    .unwrap();
    crate::terminal_sessions::with_session(&mut w, Some("B"), |w| {
        assert_eq!(terminal::execute(w, "touch other").exit_code, 0);
        Ok(())
    })
    .unwrap();
    w.vfs.write(&p("desktop"), "GUI", "kali").unwrap();
    assert_eq!(w.vfs.stat(&p("private"), "kali").unwrap().mode, 0o600);
    assert_eq!(w.vfs.stat(&p("other"), "kali").unwrap().mode, 0o644);
    assert_eq!(w.vfs.stat(&p("desktop"), "kali").unwrap().mode, 0o644);
    crate::terminal_sessions::with_session(&mut w, Some("A"), |w| {
        assert_eq!(terminal::execute(w, "touch private-again").exit_code, 0);
        Ok(())
    })
    .unwrap();
    assert_eq!(w.vfs.stat(&p("private-again"), "kali").unwrap().mode, 0o600);
}
