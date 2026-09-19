use super::wc::{Counter, Counts, Names};
use crate::{archive, terminal, world::WorldState};

#[test]
fn counts_are_independent_of_byte_and_multibyte_chunk_boundaries() {
    let mut seed = 97u32;
    for n in [0, 1, 2, 3, 7, 31, 255, 1024, 4096, 8193] {
        let data: Vec<_> = (0..n)
            .map(|_| {
                seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                (seed >> 24) as u8
            })
            .chain("é € 界 é 😀\r\nword\tword\0".bytes())
            .collect();
        for utf8 in [false, true] {
            let mut whole = Counter::new(utf8, false);
            whole.push(&data);
            whole.finish();
            for size in [1, 2, 3, 7, 31, 255, 1024, 4096] {
                let mut split = Counter::new(utf8, false);
                for chunk in data.chunks(size) {
                    split.push(chunk);
                }
                split.finish();
                assert_eq!(split.counts, whole.counts, "{n}/{size}/{utf8}");
            }
            let mut split = Counter::new(utf8, false);
            let mut at = 0;
            while at < data.len() {
                seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                let end = (at + (seed as usize % 511) + 1).min(data.len());
                split.push(&data[at..end]);
                at = end;
            }
            split.finish();
            assert_eq!(split.counts, whole.counts);
        }
    }
}

#[test]
fn locked_c_metrics_distinguish_columns_words_characters_and_newlines() {
    for (data, expected) in [
        (&b"A\0B"[..], [0, 1, 3, 3, 2]),
        (&b"A\tB"[..], [0, 2, 3, 3, 9]),
        (&b"A\x0bB"[..], [0, 2, 3, 3, 2]),
        (&b"A\x0cB"[..], [0, 2, 3, 3, 1]),
        (&b"A\xffB"[..], [0, 1, 3, 3, 2]),
        (&b"abcd\rxy"[..], [0, 2, 7, 7, 4]),
    ] {
        let mut c = Counter::new(false, false);
        c.push(data);
        c.finish();
        assert_eq!(c.counts.values, expected);
    }
    let mut c = Counter::new(true, false);
    c.push(b"a\xe2");
    c.push(b"\x82\xac\xff\n\xe2");
    c.finish();
    assert_eq!(&c.counts.values[..4], &[1, 2, 3, 7]);
}

#[test]
fn totals_saturate_with_explicit_overflow_and_take_maximum_width() {
    let mut a = Counts {
        values: [u64::MAX - 1, 2, 3, u64::MAX, 90],
        ..Default::default()
    };
    a.merge(&Counts {
        values: [2, 3, 4, 1, 7],
        ..Default::default()
    });
    assert_eq!(a.values, [u64::MAX, 5, 7, u64::MAX, 90]);
    assert_eq!(a.overflow, [true, false, false, true]);
}

#[test]
fn nul_filename_parser_is_incremental_and_resource_bounded() {
    let input = "a\0\0new\nline\0é\0tail".as_bytes();
    for size in [1, 2, 3, 7, 31, 255, 1024, 4096] {
        let mut parser = Names::default();
        let mut names = Vec::new();
        for c in input.chunks(size) {
            parser.block.extend(c);
            while let Some(s) = parser.next().unwrap() {
                names.push(s);
            }
        }
        parser.eof = true;
        while let Some(s) = parser.next().unwrap() {
            names.push(s);
        }
        assert_eq!(names, vec!["a", "", "new\nline", "é", "tail"]);
    }
    let mut parser = Names::default();
    parser.block.extend([b'x'; 4097]);
    assert!(parser.next().is_err());
}

#[test]
fn shell_producers_and_filename_lists_share_bounded_streams_and_cleanup() {
    let mut w = WorldState::new("kali", "lifeos").unwrap();
    w.terminal.env.insert("LC_ALL".into(), "C".into());
    w.terminal.exported.insert("LC_ALL".into());
    archive::write_bytes(
        &mut w,
        "/home/kali/data",
        b"one\tX\n\xff\0last".to_vec(),
        "kali",
        12,
    )
    .unwrap();
    w.vfs
        .write("/home/kali/new\nline", "abc\n", "kali")
        .unwrap();
    archive::write_bytes(
        &mut w,
        "/home/kali/list",
        b"data\0new\nline\0".to_vec(),
        "kali",
        14,
    )
    .unwrap();
    for pattern in [
        vec![1],
        vec![2],
        vec![3],
        vec![7],
        vec![31],
        vec![255],
        vec![1024],
        vec![4096],
        vec![13, 1, 97, 3, 255, 2],
    ] {
        crate::shell_pipeline::streams::with_read_pattern(&pattern, || {
            for (flag, expected) in [
                ("-c", "12\n"),
                ("-m", "12\n"),
                ("-l", "1\n"),
                ("-w", "3\n"),
                ("-L", "9\n"),
            ] {
                let result = terminal::execute(&mut w, &format!("cat data | wc {flag}"));
                assert_eq!(result.exit_code, 0, "{}", result.stderr);
                assert_eq!(result.stdout, expected);
            }
            let a = terminal::execute(&mut w, "wc -c --files0-from=list");
            let b = terminal::execute(&mut w, "cat list | wc -c --files0-from=-");
            assert_eq!(a.stdout, "12 data\n 4 'new'$'\\n''line'\n16 total\n");
            assert_eq!(b.stdout, "12 data\n4 'new'$'\\n''line'\n16 total\n");
            assert_eq!(w.vfs.open_handle_count(), 0);
            assert!(w.scheduler.waiting.is_empty());
            assert!(crate::shell_pipeline::streams::last_peak() <= 65536);
        });
    }
    let result = terminal::execute(&mut w, "yes word | head -c 5000001 | wc -c");
    assert_eq!(result.stdout, "5000001\n");
    assert_eq!(result.exit_code, 0);
    assert_eq!(w.vfs.open_handle_count(), 0);
}

#[test]
fn tty_wait_and_infinite_producer_signals_release_processes_and_handles() {
    use crate::{
        cli_contract::Event,
        shell::{control, signals::VirtualSignal},
    };
    for script in [
        "wc -c",
        "cat | wc -w",
        "cat /dev/zero | wc -m",
        "wc -l /dev/zero",
    ] {
        for signal in [VirtualSignal::Int, VirtualSignal::Term] {
            let key = format!("wc-signal-{script}-{signal:?}");
            let (tx, rx) = std::sync::mpsc::channel();
            let output = tauri::ipc::Channel::new(move |body| {
                let _ = tx.send(body.deserialize::<Event>().unwrap());
                Ok(())
            });
            let registration = control::register_output(&key, Some(output));
            let mut w = WorldState::new("kali", "lifeos").unwrap();
            let result = std::thread::scope(|scope| {
                let worker = scope
                    .spawn(|| control::run(&registration, || terminal::execute(&mut w, script)));
                if !script.contains("/dev/zero") {
                    let event = rx.recv_timeout(std::time::Duration::from_secs(10));
                    if !matches!(event, Ok(Event::WaitingForInput)) {
                        control::cancel(&key);
                        let _ = worker.join();
                        panic!("missing pending event {event:?}");
                    }
                    assert!(control::input(&key, Some("data\n".into())));
                }
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
                while control::process_ids(&key).is_empty() && std::time::Instant::now() < deadline
                {
                    std::thread::yield_now();
                }
                // Let the cooperative producer execute before interrupting the whole job.
                if script.contains("/dev/zero") {
                    std::thread::sleep(std::time::Duration::from_millis(30));
                }
                assert!(control::signal(&key, None, signal));
                worker.join().unwrap()
            });
            assert_eq!(
                result.exit_code,
                if signal == VirtualSignal::Int {
                    130
                } else {
                    143
                },
                "{script}: {}",
                result.stderr
            );
            assert!(control::process_ids(&key).is_empty());
            assert_eq!(w.vfs.open_handle_count(), 0);
            assert!(w.scheduler.waiting.is_empty());
        }
    }
}
