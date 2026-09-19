use crate::{archive, shell_pipeline::streams, terminal, world::WorldState};

fn bytes(w: &WorldState, path: &str) -> Vec<u8> {
    archive::bytes(w, &format!("/home/kali/{path}"), "kali")
        .unwrap()
        .as_ref()
        .clone()
}
fn clean(w: &WorldState) {
    assert_eq!(w.vfs.open_handle_count(), 0);
    assert!(!w
        .processes
        .iter()
        .any(|p| p.running && ["tee", "cat", "head"].contains(&p.name.as_str())));
    assert!(w.scheduler.signals.is_empty());
}

#[test]
fn fanout_preserves_arbitrary_bytes_across_fragmentation_and_append() {
    let mut seed = 0x1c6u32;
    let data: Vec<u8> = (0..521)
        .map(|_| {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (seed >> 24) as u8
        })
        .collect();
    let mut patterns: Vec<Vec<usize>> = [1, 2, 3, 7, 31, 255, 4096].map(|n| vec![n]).into();
    patterns.push(
        (0..23)
            .map(|_| {
                seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                seed as usize % 4096 + 1
            })
            .collect(),
    );
    for pattern in patterns {
        let mut w = WorldState::new("kali", "lifeos").unwrap();
        w.vfs.write("/home/kali/a", "prefix", "kali").unwrap();
        w.terminal.stdin_bytes = Some(data.clone());
        let result = streams::with_read_pattern(&pattern, || {
            terminal::execute(&mut w, "cat | tee -a a b c > out")
        });
        assert_eq!(result.exit_code, 0, "{}", result.stderr);
        assert_eq!(bytes(&w, "a"), [b"prefix".as_slice(), &data].concat());
        for path in ["b", "c", "out"] {
            assert_eq!(bytes(&w, path), data, "{path}, {pattern:?}");
        }
        clean(&w);
    }
}

#[test]
fn finite_and_infinite_producers_release_bounded_pipes() {
    for script in [
        "cat /dev/zero | tee a b | head -c 8193 > out",
        "head -c 131075 /dev/zero | tee -p a b | head -c 8193 > out",
    ] {
        let mut w = WorldState::new("kali", "lifeos").unwrap();
        let result = terminal::execute(&mut w, script);
        assert_eq!(result.exit_code, 0, "{}", result.stderr);
        assert_eq!(bytes(&w, "out"), vec![0; 8193]);
        assert_eq!(bytes(&w, "a"), bytes(&w, "b"));
        if script.contains("-p") {
            assert_eq!(bytes(&w, "a").len(), 131075);
        }
        assert!(streams::last_peak() <= 64 * 1024);
        clean(&w);
    }
}

#[test]
fn write_failure_preserves_earlier_destinations_and_closes_all_handles() {
    for mode in ["warn", "exit", "exit-nopipe"] {
        let mut w = WorldState::new("kali", "lifeos").unwrap();
        w.terminal.stdin_bytes = Some(b"binary\0\xff".to_vec());
        let result = terminal::execute(&mut w, &format!("tee --output-error={mode} a /dev/full b"));
        assert_eq!(result.exit_code, 1);
        assert_eq!(result.stdout_bytes, b"binary\0\xff");
        assert_eq!(bytes(&w, "a"), b"binary\0\xff");
        assert_eq!(
            bytes(&w, "b"),
            if mode == "warn" {
                b"binary\0\xff".to_vec()
            } else {
                Vec::new()
            }
        );
        clean(&w);
    }
}

#[test]
fn late_virtual_storage_error_retains_prefix_and_sibling_outputs() {
    let mut w = WorldState::new("kali", "lifeos").unwrap();
    let mut tee = super::tee::Tee::new("tee", &["a".into(), "/dev/null".into()], false)
        .unwrap_or_else(|_| panic!("parse"));
    assert!(tee.open(&mut w).0.is_empty());
    tee.data = Some(b"before".to_vec());
    assert!(tee.write_files(&mut w).0.is_empty());
    w.vfs.capacity_bytes = w.vfs.used_bytes();
    tee.data = Some(b"after".to_vec());
    let (errors, fatal) = tee.write_files(&mut w);
    assert!(!errors.is_empty() && !fatal);
    assert_eq!(bytes(&w, "a"), b"before");
    assert!(tee.handles[0].1.is_none() && tee.handles[1].1.is_some());
    tee.close(&mut w).unwrap();
    clean(&w);
}

#[test]
fn ignored_signals_are_process_local_and_never_hide_termination() {
    use crate::shell::signals::{ProcessSignalState, VirtualSignal};
    let a = ProcessSignalState::default();
    let b = ProcessSignalState::default();
    a.ignore(VirtualSignal::Int);
    a.send(VirtualSignal::Int);
    b.send(VirtualSignal::Int);
    assert_eq!(a.take(), None);
    assert_eq!(b.take(), Some(VirtualSignal::Int));
    a.send(VirtualSignal::Term);
    assert_eq!(a.take(), Some(VirtualSignal::Term));
    a.ignore(VirtualSignal::Kill);
    a.send(VirtualSignal::Kill);
    assert_eq!(a.take(), Some(VirtualSignal::Kill));
}
