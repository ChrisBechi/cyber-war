use super::base64::{Base64, Transform};
use base64::{engine::general_purpose::STANDARD, Engine};

fn hex(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}

#[test]
fn gnu_finite_decode_observations_are_independent_of_chunk_boundaries() {
    // These bytes were adopted from two identical GNU 9.7 executions, not a codec oracle.
    let cases: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/cli/compat/pilot/coreutils-base64.json"
    ))
    .unwrap();
    let mut checked = 0;
    for case in cases.as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        if case.get("io").is_some() {
            continue;
        }
        let valid_wrap = id.contains("/wrap-parse-") && case["expected"]["exitCode"] == 0;
        if !id.contains("/decode-") && !id.contains("/encode-binary-") && !valid_wrap {
            continue;
        }
        let args: Vec<String> = case["argv"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_owned())
            .collect();
        let bytes = case
            .get("stdinHex")
            .and_then(|v| v.as_str())
            .map(hex)
            .unwrap_or_else(|| case["stdin"].as_str().unwrap_or("").as_bytes().to_vec());
        for size in [1, 2, 3, 4, 5, 7, 31, 255, 4096, 0] {
            let mut state = Base64::new("base64", &args, false).unwrap().transform;
            let mut output = Vec::new();
            let mut offset = 0;
            let mut seed = 0x1c5u32;
            while offset < bytes.len() && !state.invalid() {
                seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                let length = if size == 0 {
                    (seed as usize % 513) + 1
                } else {
                    size
                };
                let end = (offset + length).min(bytes.len());
                output.extend(state.push(&bytes[offset..end]));
                offset = end;
            }
            output.extend(state.finish());
            assert_eq!(
                output,
                hex(case["expected"]["stdoutHex"].as_str().unwrap()),
                "{id}, chunk {size}"
            );
            assert_eq!(
                i64::from(state.invalid()),
                case["expected"]["exitCode"].as_i64().unwrap(),
                "{id}, chunk {size}"
            );
        }
        checked += 1;
    }
    assert!(checked >= 100);
}

#[test]
fn wrapping_carries_columns_across_every_input_boundary() {
    let bytes: Vec<u8> = (0..17000).map(|n| (n % 256) as u8).collect();
    let encoded = STANDARD.encode(&bytes);
    for width in [0, 1, 2, 3, 4, 5, 75, 76, 77, 4096] {
        let expected = if width == 0 {
            encoded.as_bytes().to_vec()
        } else {
            encoded
                .as_bytes()
                .chunks(width)
                .flat_map(|c| c.iter().copied().chain(*b"\n"))
                .collect()
        };
        for chunk in [1, 2, 3, 5, 31, 255, 4096] {
            let mut state = Transform::new(false, false, width as u64);
            let mut output: Vec<u8> = bytes.chunks(chunk).flat_map(|c| state.push(c)).collect();
            output.extend(state.finish());
            assert_eq!(output, expected, "wrap {width}, chunk {chunk}");
        }
    }
    assert!(std::mem::size_of::<Transform>() <= 64);
}

#[test]
fn binary_pipeline_partial_redirect_and_bounded_infinite_source() {
    let mut w = crate::world::WorldState::new("kali", "lifeos").unwrap();
    let result = crate::terminal::execute(&mut w, "echo 'Zm9vZg!' | base64 -d > partial");
    assert_eq!(result.exit_code, 1, "{} / {}", result.stdout, result.stderr);
    assert_eq!(result.stderr, "base64: invalid input\n");
    assert_eq!(
        crate::archive::bytes(&w, "/home/kali/partial", "kali")
            .unwrap()
            .as_ref(),
        b"foof"
    );
    let result = crate::terminal::execute(&mut w, "base64 /dev/zero | head -c 100000 > zeros");
    assert_eq!(result.exit_code, 0, "{}", result.stderr);
    assert_eq!(
        crate::archive::bytes(&w, "/home/kali/zeros", "kali")
            .unwrap()
            .len(),
        100000
    );
    assert!(crate::shell_pipeline::streams::last_peak() <= 64 * 1024);
    assert!(!w
        .processes
        .iter()
        .any(|p| p.running && ["base64", "head"].contains(&p.name.as_str())));
    w.packages.installed.remove("coreutils");
    assert_ne!(
        crate::terminal::execute(&mut w, "/usr/bin/base64 --help").exit_code,
        0
    );
}

#[test]
fn arbitrary_decode_is_deterministic_and_keeps_bounded_state() {
    let mut seed = 0x1c5u32;
    for length in 0..512 {
        let data: Vec<_> = (0..length)
            .map(|_| {
                seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                (seed >> 24) as u8
            })
            .collect();
        for ignore in [false, true] {
            let mut single = Transform::new(true, ignore, 0);
            let mut expected = single.push(&data);
            expected.extend(single.finish());
            let mut split = Transform::new(true, ignore, 0);
            let mut actual: Vec<_> = data.chunks(7).flat_map(|c| split.push(c)).collect();
            actual.extend(split.finish());
            assert_eq!(actual, expected);
            assert_eq!(single.invalid(), split.invalid());
        }
    }
}

#[test]
fn interrupted_stdout_matches_gnu_block_and_stdio_boundaries() {
    let cases: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/cli/compat/pilot/coreutils-base64.json"
    ))
    .unwrap();
    let mut checked = 0;
    for case in cases
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["id"].as_str().unwrap().contains("-interrupt-"))
    {
        let args: Vec<_> = case["argv"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_owned())
            .collect();
        let input: Vec<_> = case["interaction"]["steps"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|s| s["hex"].as_str())
            .flat_map(hex)
            .collect();
        for chunk in [1, 7, 255, 4096] {
            let mut stream = Base64::new("base64", &args, false).unwrap();
            let output: Vec<_> = input
                .chunks(chunk)
                .flat_map(|data| stream.push(data))
                .collect();
            assert_eq!(
                output,
                hex(case["expected"]["stdoutHex"].as_str().unwrap()),
                "{}, chunk {chunk}",
                case["id"]
            );
        }
        checked += 1;
    }
    assert!(checked >= 10);
}

#[test]
fn virtual_tty_eof_and_targeted_signals_release_runtime_resources() {
    use crate::{
        cli_contract::Event,
        shell::{control, signals::VirtualSignal},
    };
    for (label, signal, status) in [
        ("eof", None, 0),
        ("int", Some(VirtualSignal::Int), 130),
        ("term", Some(VirtualSignal::Term), 143),
    ] {
        let key = format!("base64-test-{label}");
        let (tx, rx) = std::sync::mpsc::channel();
        let output = tauri::ipc::Channel::new(move |body| {
            let _ = tx.send(body.deserialize::<Event>().unwrap());
            Ok(())
        });
        let registration = control::register_output(&key, Some(output));
        let mut world = crate::world::WorldState::new("kali", "lifeos").unwrap();
        let result = std::thread::scope(|scope| {
            let worker = scope.spawn(|| {
                control::run(&registration, || {
                    crate::terminal::execute(&mut world, "base64 -d")
                })
            });
            let event = rx.recv_timeout(std::time::Duration::from_secs(10));
            if !matches!(event, Ok(Event::WaitingForInput)) {
                control::cancel(&key);
                let _ = worker.join();
                panic!("expected pending stdin, got {event:?}");
            }
            let pids = control::process_ids(&key);
            if let Some(signal) = signal {
                assert!(control::signal(&key, Some(pids[0]), signal));
            } else {
                assert!(control::input(&key, Some("Z\nm9v\n".into())));
                assert!(control::input(&key, None));
            }
            worker.join().unwrap()
        });
        assert_eq!(result.exit_code, status, "{label}: {}", result.stderr);
        if signal.is_none() {
            assert_eq!(result.stdout, "foo");
        }
        assert!(control::process_ids(&key).is_empty());
        assert_eq!(world.vfs.open_handle_count(), 0);
        assert!(world.scheduler.waiting.is_empty());
    }
}
