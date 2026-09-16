//! Development-only measurements; never linked into the production binary.
use super::*;
use serde_json::json;
use std::time::Instant;
fn measure(iterations: usize, mut work: impl FnMut()) -> serde_json::Value {
    work();
    let started = Instant::now();
    for _ in 0..iterations {
        work();
    }
    let elapsed = started.elapsed().as_secs_f64() * 1000.0;
    json!({"iterations":iterations,"totalMs":elapsed,"meanMs":elapsed/iterations as f64})
}
#[test]
#[ignore = "development shell performance capture"]
fn cli_tooling_shell_performance() {
    let simple = measure(10000, || {
        std::hint::black_box(syntax::parse("echo \"$HOME\" | cat > out"));
    });
    let script = "VALUE=$(echo one);true && echo \"$VALUE\" | cat\n".repeat(256);
    let moderate = measure(100, || {
        std::hint::black_box(syntax::parse(&script));
    });
    let mut w = WorldState::new("kali", "pc").unwrap();
    let pipeline = measure(10, || {
        let output = crate::terminal::execute(&mut w, "yes x|head -n 100000|wc -l");
        assert_eq!(output.stdout, "100000\n");
        assert_eq!(output.exit_code, 0);
    });
    let peak = crate::shell_pipeline::streams::last_peak();
    let mut glob = Vec::new();
    for requested in [100, 1000, 10000] {
        w.vfs = crate::vfs::VirtualFileSystem::default();
        w.vfs.mkdir("/home/kali/bench", "kali").unwrap();
        w.terminal.cwd = "/home/kali/bench".into();
        let actual = requested.min(10000 - w.vfs.nodes.len());
        for index in 0..actual {
            w.vfs
                .write(&format!("/home/kali/bench/file-{index:05}.log"), "", "kali")
                .unwrap();
        }
        let stats = measure(5, || {
            let output = crate::terminal::execute(&mut w, "echo *.log");
            assert_eq!(output.exit_code, 0);
            assert_eq!(output.stdout.split_whitespace().count(), actual);
        });
        glob.push(json!({"requestedEntries":requested,"actualEntries":actual,"vfsNodes":w.vfs.nodes.len(),"execution":stats}));
    }
    let result = json!({"schemaVersion":1,"profile":"cargo test / debug; wall-clock; no host command execution","simpleParse":simple,"moderateParse":{"sourceBytes":script.len(),"measurement":moderate},"threeStage100000Lines":pipeline,"pipeCapacityBytes":65536,"observedPeakPipeBytes":peak,"glob":glob});
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../artifacts/shell-performance.json");
    std::fs::write(path, serde_json::to_string_pretty(&result).unwrap() + "\n").unwrap();
    println!("{result}");
}
