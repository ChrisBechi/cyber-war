use super::sha256sum::{emit, Line, Lines, Options, Parser, RECORD_LIMIT};
use crate::{archive, terminal, world::WorldState};
use sha2::{Digest, Sha256};

#[test]
fn sha256_known_vectors_and_chunk_boundaries() {
    for (bytes, expected) in [
        (
            Vec::new(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        ),
        (
            b"abc".to_vec(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        ),
        (
            vec![b'a'; 1_000_000],
            "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0",
        ),
    ] {
        for size in [1, 2, 3, 7, 31, 63, 64, 65, 255, 1024, 4096] {
            let mut hash = Sha256::new();
            for chunk in bytes.chunks(size) {
                hash.update(chunk);
            }
            assert_eq!(format!("{:x}", hash.finalize()), expected);
        }
    }
}

#[test]
fn emit_parse_roundtrips_and_arbitrary_list_bytes_are_deterministic_and_bounded() {
    let hash = Sha256::digest(b"abc");
    for name in [
        "a",
        "space name",
        "tab\tname",
        "new\nline",
        "back\\slash",
        "cr\rname",
        "é",
        "-dash",
        "a)b(c",
    ] {
        for tag in [false, true] {
            let record = emit(
                name.as_bytes(),
                &hash,
                &Options {
                    tag,
                    ..Default::default()
                },
            );
            let Line::Record(parsed) = Parser::default().parse(&record[..record.len() - 1]) else {
                panic!("record {name}");
            };
            assert_eq!(parsed.name, name.as_bytes());
            assert_eq!(parsed.digest, hash[..]);
            let zero = emit(
                name.as_bytes(),
                &hash,
                &Options {
                    tag,
                    zero: true,
                    ..Default::default()
                },
            );
            assert_eq!(zero.last(), Some(&0));
            assert!(zero.windows(name.len()).any(|b| b == name.as_bytes()));
        }
    }
    let mut seed = 97u32;
    for length in [0, 1, 2, 7, 31, 63, 64, 65, 255, 1024, 4096, 32769] {
        let bytes: Vec<_> = (0..length)
            .map(|_| {
                seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                (seed >> 24) as u8
            })
            .collect();
        let mut whole = Parser::default();
        let mut baseline = Vec::new();
        let mut lines = Lines::default();
        lines.block.extend(&bytes);
        lines.eof = true;
        while let Some(line) = lines.next() {
            baseline.push(line.map_or(Line::Malformed, |b| whole.parse(&b)));
        }
        for size in [1, 2, 3, 7, 31, 63, 64, 65, 255, 1024, 4096] {
            let mut lines = Lines::default();
            let mut parser = Parser::default();
            let mut observed = Vec::new();
            for chunk in bytes.chunks(size) {
                lines.block.extend(chunk);
                while let Some(line) = lines.next() {
                    observed.push(line.map_or(Line::Malformed, |b| parser.parse(&b)));
                }
                assert!(lines.buffered() <= RECORD_LIMIT + 4096);
            }
            lines.eof = true;
            while let Some(line) = lines.next() {
                observed.push(line.map_or(Line::Malformed, |b| parser.parse(&b)));
            }
            assert_eq!(observed, baseline);
        }
    }
    let mut lines = Lines::default();
    for _ in 0..1024 {
        lines.block.extend([b'x'; 4096]);
        assert!(lines.next().is_none());
        assert!(lines.buffered() <= RECORD_LIMIT);
    }
    lines.block.extend(b"\na\n");
    assert!(lines.next().unwrap().is_none());
    assert_eq!(lines.next(), Some(Some(b"a".to_vec())));
}

#[test]
fn real_shell_hash_check_and_escaped_roundtrips_across_chunks() {
    let mut w = WorldState::new("kali", "lifeos").unwrap();
    let bytes: Vec<_> = (0..8193).map(|i| (i * 137) as u8).collect();
    let expected = format!("{:x}", Sha256::digest(&bytes));
    for name in ["a", "new\nline", "back\\slash", "é"] {
        archive::write_bytes(
            &mut w,
            &format!("/home/kali/{name}"),
            bytes.clone(),
            "kali",
            bytes.len() as u64,
        )
        .unwrap();
    }
    let mut seed = 97u32;
    let random: Vec<_> = (0..37)
        .map(|_| {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (seed as usize % 4096) + 1
        })
        .collect();
    for pattern in [
        vec![1],
        vec![2],
        vec![3],
        vec![7],
        vec![31],
        vec![63],
        vec![64],
        vec![65],
        vec![255],
        vec![1024],
        vec![4096],
        random,
    ] {
        crate::shell_pipeline::streams::with_read_pattern(&pattern, || {
            let r = terminal::execute(&mut w, "cat a | sha256sum");
            assert_eq!(r.exit_code, 0, "{}", r.stderr);
            assert_eq!(r.stdout, format!("{expected}  -\n"));
            let r = terminal::execute(&mut w, "sha256sum a 'new\nline' 'back\\slash' 'é' > sums");
            assert_eq!(r.exit_code, 0, "{}", r.stderr);
            let r = terminal::execute(&mut w, "cat sums | sha256sum -c -");
            assert_eq!(r.exit_code, 0, "{}", r.stderr);
            assert!(r.stdout.ends_with("é: OK\n"));
            let r = terminal::execute(&mut w, "sha256sum --tag a >> sums");
            assert_eq!(r.exit_code, 0);
            let r = terminal::execute(&mut w, "sha256sum -c sums > result");
            assert_eq!(r.exit_code, 0, "{}", r.stderr);
            assert_eq!(w.vfs.open_handle_count(), 0);
            assert!(w.scheduler.waiting.is_empty());
            assert!(crate::shell_pipeline::streams::last_peak() <= 65536);
        });
    }
}

#[test]
fn producer_beyond_legacy_limit_and_stdin_ownership() {
    let mut w = WorldState::new("kali", "lifeos").unwrap();
    let bytes: Vec<_> = b"word\n".iter().copied().cycle().take(5_000_001).collect();
    let r = terminal::execute(&mut w, "yes word | head -c 5000001 | sha256sum");
    assert_eq!(r.exit_code, 0, "{}", r.stderr);
    assert_eq!(r.stdout, format!("{:x}  -\n", Sha256::digest(bytes)));
    w.vfs.write("/home/kali/list","ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad  -\ne3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  -\n","kali").unwrap();
    w.vfs.write("/home/kali/data", "abc", "kali").unwrap();
    let r = terminal::execute(&mut w, "cat data | sha256sum -c list");
    assert_eq!(r.exit_code, 0, "{}", r.stderr);
    assert_eq!(r.stdout, "-: OK\n-: OK\n");
    let r = terminal::execute(&mut w, "cat list | sha256sum -c");
    assert_eq!(r.exit_code, 1);
    assert!(r.stdout.is_empty());
    assert!(r.stderr.contains("no properly formatted checksum lines"));
    let r = terminal::execute(&mut w, "sha256sum missing | cat");
    assert_eq!(r.exit_code, 0, "shell status belongs to the final consumer");
    assert_eq!(
        crate::shell_pipeline::streams::last_stage_termination(0)
            .unwrap()
            .status(),
        1
    );
    assert_eq!(w.vfs.open_handle_count(), 0);
    assert!(w.scheduler.waiting.is_empty());
}

#[test]
fn tty_wait_and_infinite_producer_signals_release_processes_and_handles() {
    use crate::{
        cli_contract::Event,
        shell::{control, signals::VirtualSignal},
    };
    for script in [
        "sha256sum",
        "cat | sha256sum -c",
        "cat /dev/zero | sha256sum",
        "sha256sum /dev/zero",
    ] {
        for signal in [VirtualSignal::Int, VirtualSignal::Term] {
            let key = format!("sha256sum-signal-{script}-{signal:?}");
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
