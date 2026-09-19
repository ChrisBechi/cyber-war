use super::{head::Selection, tail::Transform};

#[test]
fn tail_selection_is_independent_of_chunk_boundaries_and_binary_encoding() {
    let mut data: Vec<u8> = (0..=255).cycle().take(2051).collect();
    data.extend_from_slice(b"\nlast");
    for delimiter in [0, b'\n'] {
        for bytes in [false, true] {
            for start in [false, true] {
                for count in [0, 1, 2, 9, 1024, u64::MAX] {
                    let selection = Selection {
                        bytes,
                        negative: false,
                        count,
                    };
                    let expected = if bytes {
                        let index = if start {
                            count.saturating_sub(1).min(data.len() as u64) as usize
                        } else {
                            data.len().saturating_sub(count as usize)
                        };
                        data[index..].to_vec()
                    } else {
                        let lines: Vec<_> = data.split_inclusive(|b| *b == delimiter).collect();
                        let index = if start {
                            count.saturating_sub(1).min(lines.len() as u64) as usize
                        } else {
                            lines.len().saturating_sub(count as usize)
                        };
                        lines[index..].concat()
                    };
                    for chunk in [1, 2, 7, 63, 1024, 4096] {
                        let mut transform = Transform::new(selection, start, delimiter);
                        let mut actual = Vec::new();
                        for part in data.chunks(chunk) {
                            actual.extend(transform.push(part));
                        }
                        actual.extend(transform.finish());
                        assert_eq!(
                            actual, expected,
                            "start={start},bytes={bytes},count={count},chunk={chunk}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn finite_tail_seeks_suffixes_and_handles_symlink_dotdot() {
    let mut world = crate::world::WorldState::new("kali", "pc").unwrap();
    world.vfs.mkdir("/home/kali/dir", "kali").unwrap();
    world.vfs.mkdir("/home/kali/dir/sub", "kali").unwrap();
    world
        .vfs
        .write("/home/kali/dir/log", "x\nlast\n", "kali")
        .unwrap();
    world
        .vfs
        .symlink("/home/kali/link", "dir/sub", "kali")
        .unwrap();
    let result = crate::terminal::execute(&mut world, "tail -n1 link/../log");
    assert_eq!(result.stdout, "last\n");
    assert_eq!(result.exit_code, 0);
    let result = crate::terminal::execute(&mut world, "yes | tail -n+2 | head -n3");
    assert_eq!(result.stdout, "y\ny\ny\n");
    assert_eq!(result.exit_code, 0);
    assert_eq!(world.vfs.open_handle_count(), 0);
    assert_eq!(world.vfs.watcher_count(), 0);
}

#[test]
fn tail_follow_notices_closed_pipe_even_without_bytes_to_write() {
    let mut world = crate::world::WorldState::new("kali", "pc").unwrap();
    world.vfs.write("/home/kali/log", "x\n", "kali").unwrap();
    let result = crate::terminal::execute(&mut world, "tail -n0 -f log | head -n0");
    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.is_empty());
    assert_eq!(world.vfs.open_handle_count(), 0);
    assert_eq!(world.vfs.watcher_count(), 0);
    assert!(world.scheduler.waiting.is_empty());
}

#[test]
fn tail_suffix_limit_is_controlled_at_each_boundary() {
    let limit = 4 * 1024 * 1024;
    for size in [limit - 1, limit, limit + 1] {
        let args = vec!["-c".into(), size.to_string()];
        let mut tail = match super::tail::Tail::new("tail", &args, false) {
            Ok(tail) => tail,
            Err(_) => panic!("valid tail count"),
        };
        for bytes in vec![255; size].chunks(4096) {
            tail.feed(Some(bytes));
        }
        assert_eq!(tail.status, i32::from(size > limit), "{size}");
        let mut world = crate::world::WorldState::new("kali", "pc").unwrap();
        tail.close(&mut world).unwrap();
        assert_eq!(world.vfs.watcher_count(), 0);
        assert_eq!(world.vfs.open_handle_count(), 0);
    }
}

#[test]
fn tail_output_capture_reserves_diagnostic_space_at_each_boundary() {
    let limit = 4 * 1024 * 1024 - 1024;
    for size in [limit - 1, limit, limit + 1] {
        let mut world = crate::world::WorldState::new("kali", "pc").unwrap();
        crate::archive::write_bytes(
            &mut world,
            "/home/kali/large",
            vec![b'x'; size],
            "kali",
            size as u64,
        )
        .unwrap();
        let result = crate::terminal::execute(&mut world, "tail -c+1 large");
        if size <= limit {
            assert_eq!(result.exit_code, 0);
            assert_eq!(result.stdout_bytes.len(), size);
        } else {
            assert_eq!(result.exit_code, 1);
            assert!(result.stderr.contains("4 MiB"));
            assert!(result.stdout_bytes.len() <= limit);
        }
        assert_eq!(
            world
                .vfs
                .stat("/home/kali/large", "kali")
                .unwrap()
                .logical_size(),
            size as u64
        );
        assert_eq!(world.vfs.watcher_count(), 0);
        assert_eq!(world.vfs.open_handle_count(), 0);
    }
}

fn drain_follow(
    tail: &mut super::tail::Tail,
    world: &mut crate::world::WorldState,
) -> (Vec<u8>, Vec<u8>) {
    let mut out = Vec::new();
    let mut err = Vec::new();
    for _ in 0..1000 {
        match tail.step(world, None, 17).unwrap() {
            super::tail::Step::Output(fd, bytes) => {
                if fd == 1 {
                    out.extend(bytes)
                } else {
                    err.extend(bytes)
                }
            }
            super::tail::Step::Progress => {}
            super::tail::Step::Wait | super::tail::Step::Done => return (out, err),
            super::tail::Step::Input => panic!("regular file unexpectedly requested stdin"),
        }
    }
    panic!("follow failed to suspend within a bounded number of steps");
}

#[test]
fn tail_symlink_recheck_uses_shared_virtual_deadline_and_releases_timer() {
    let mut world = crate::world::WorldState::new("kali", "pc").unwrap();
    world.vfs.write("/home/kali/log", "old", "kali").unwrap();
    world.vfs.symlink("/home/kali/link", "log", "kali").unwrap();
    let args = ["-n0", "-F", "-s2", "--max-unchanged-stats=1", "link"].map(String::from);
    let mut tail =
        super::tail::Tail::new("tail", &args, false).unwrap_or_else(|_| panic!("valid options"));
    assert_eq!(drain_follow(&mut tail, &mut world), (vec![], vec![]));
    assert_eq!(world.scheduler.timer_count(), 0);
    world
        .vfs
        .write("/home/kali/replacement", "new", "kali")
        .unwrap();
    world
        .vfs
        .symlink("/home/kali/newlink", "replacement", "kali")
        .unwrap();
    world
        .vfs
        .rename_entry("/home/kali/newlink", "/home/kali/link", "kali")
        .unwrap();
    assert_eq!(drain_follow(&mut tail, &mut world), (vec![], vec![]));
    let wait = tail.wait_set(&world);
    assert_eq!(world.scheduler.timer_count(), 1);
    assert!(!wait.ready(&world));
    assert!(world.scheduler.advance_idle(&wait));
    assert_eq!(
        world.scheduler.now,
        4 * crate::shell::wait::TICKS_PER_SECOND
    );
    let (out, err) = drain_follow(&mut tail, &mut world);
    assert_eq!(out, b"new");
    assert!(String::from_utf8(err)
        .unwrap()
        .contains("has been replaced"));
    assert_eq!(world.scheduler.timer_count(), 0);
    tail.close(&mut world).unwrap();
    assert_eq!(world.vfs.watcher_count(), 0);
    assert_eq!(world.vfs.open_handle_count(), 0);
}

#[test]
fn tail_gnu_regressions_beyond_eof_and_coalesced_truncate_growth() {
    for (args, initial, replacement, first, later, diagnostic) in [
        (vec!["-c+8", "-f", "log"], "ab", None, "ab", "", true),
        (
            vec!["-c2", "-f", "log"],
            "old-data",
            Some("xxxxxxxxx"),
            "ta",
            "x",
            false,
        ),
        (
            vec!["-c2", "-f", "log"],
            "old-data",
            Some("xx"),
            "ta",
            "xx",
            true,
        ),
    ] {
        let mut world = crate::world::WorldState::new("kali", "pc").unwrap();
        world.vfs.write("/home/kali/log", initial, "kali").unwrap();
        let args: Vec<_> = args.into_iter().map(String::from).collect();
        let mut tail = super::tail::Tail::new("tail", &args, false)
            .unwrap_or_else(|_| panic!("valid options"));
        let (out, mut err) = drain_follow(&mut tail, &mut world);
        assert_eq!(out, first.as_bytes());
        if let Some(replacement) = replacement {
            world.vfs.truncate("/home/kali/log", 0, "kali").unwrap();
            world
                .vfs
                .write("/home/kali/log", replacement, "kali")
                .unwrap();
            let (out, next_err) = drain_follow(&mut tail, &mut world);
            assert_eq!(out, later.as_bytes());
            err.extend(next_err);
        }
        assert_eq!(!err.is_empty(), diagnostic);
        if diagnostic {
            assert_eq!(err, b"tail: log: file truncated\n");
        }
        tail.close(&mut world).unwrap();
        assert_eq!(world.vfs.watcher_count(), 0);
        assert_eq!(world.vfs.open_handle_count(), 0);
    }
}
