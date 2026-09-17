//! DEV only. Timings compare virtual command costs, never host Coreutils speed.
use crate::{terminal, world::WorldState};
use serde_json::json;

#[test]
#[ignore = "DEV Coreutils performance capture"]
fn cli_tooling_coreutils_performance() {
    let mut samples = Vec::new();
    for count in [1, 100, 1000, 8000] {
        let mut world = WorldState::new("kali", "lifeos").unwrap();
        world.vfs.mkdir("/home/kali/bench", "kali").unwrap();
        for n in 0..count {
            world
                .vfs
                .write(&format!("/home/kali/bench/f{n:05}"), "sample\n", "kali")
                .unwrap();
        }
        let count_before = world.vfs.nodes.len();
        let started = std::time::Instant::now();
        let result = terminal::execute(&mut world, "du -s bench");
        assert_eq!(result.exit_code, 0, "{}", result.stderr);
        samples.push(json!({"command":"du -s bench","files":count,"entries":count_before,"ms":started.elapsed().as_secs_f64()*1000.0}));
        // A recursive copy needs room for source AND destination entries.
        if count <= 1000 {
            let started = std::time::Instant::now();
            let result = terminal::execute(&mut world, "cp -r bench copied");
            assert_eq!(result.exit_code, 0, "{}", result.stderr);
            samples.push(json!({"command":"cp -r bench copied","files":count,"ms":started.elapsed().as_secs_f64()*1000.0}));
            assert_eq!(terminal::execute(&mut world, "rm -r copied").exit_code, 0);
        }
        let started = std::time::Instant::now();
        assert_eq!(terminal::execute(&mut world, "rm -r bench").exit_code, 0);
        samples.push(json!({"command":"rm -r bench","files":count,"ms":started.elapsed().as_secs_f64()*1000.0}));
    }
    let mut world = WorldState::new("kali", "lifeos").unwrap();
    world
        .vfs
        .write(
            "/home/kali/large",
            &"line 12345678901234\n".repeat(50_000),
            "kali",
        )
        .unwrap();
    for (command, input_bytes) in [
        ("cat large | wc -c", 1_000_000),
        ("sort large > sorted", 1_000_000),
        ("sha256sum large", 1_000_000),
        ("yes | head -n 100000 > bounded", 0),
    ] {
        let started = std::time::Instant::now();
        let result = terminal::execute(&mut world, command);
        assert_eq!(result.exit_code, 0, "{command}: {}", result.stderr);
        samples.push(json!({"command":command,"inputBytes":input_bytes,"ms":started.elapsed().as_secs_f64()*1000.0}));
    }
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../artifacts/coreutils-performance.json");
    std::fs::write(path,serde_json::to_string_pretty(&json!({"profile":"debug; one sample per operation; virtual commands only","samples":samples,"pipeCapacityBytes":65536,"adapterLimitBytes":4*1024*1024})).unwrap()).unwrap();
}
