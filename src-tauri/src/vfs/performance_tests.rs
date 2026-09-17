use super::*;
use serde_json::json;
use std::time::Instant;
fn measurement(iterations: usize, mut action: impl FnMut()) -> serde_json::Value {
    let start = Instant::now();
    for _ in 0..iterations {
        action();
    }
    json!({"iterations":iterations,"meanMs":start.elapsed().as_secs_f64()*1000.0/iterations as f64})
}
#[test]
#[ignore = "DEV virtual filesystem benchmark"]
fn cli_tooling_vfs_performance() {
    let mut samples = Vec::new();
    for requested in [1000, 10000] {
        let mut fs = VirtualFileSystem::default();
        fs.mkdir("/home/kali/bench", "kali").unwrap();
        let actual = requested.min(10000 - fs.nodes.len());
        let start = Instant::now();
        for i in 0..actual {
            fs.write(&format!("/home/kali/bench/f{i:05}"), "payload", "kali")
                .unwrap();
        }
        let create_ms = start.elapsed().as_secs_f64() * 1000.0;
        let enumeration = measurement(25, || {
            assert_eq!(
                fs.child_names("/home/kali/bench", "kali").unwrap().len(),
                actual
            );
        });
        let repeated_stat = measurement(10000, || {
            std::hint::black_box(fs.stat("/home/kali/bench/f00000", "kali").unwrap());
        });
        let snapshot = serde_json::to_string(&fs).unwrap();
        let save = measurement(3, || {
            std::hint::black_box(serde_json::to_string(&fs).unwrap());
        });
        let load = measurement(3, || {
            let restored: VirtualFileSystem = serde_json::from_str(&snapshot).unwrap();
            restored.check_invariants().unwrap();
        });
        fs.check_invariants().unwrap();
        samples.push(json!({"requested":requested,"actual":actual,"entries":fs.nodes.len(),"createMs":create_ms,"enumeration":enumeration,"stat":repeated_stat,"save":save,"load":load,"snapshotBytes":snapshot.len()}));
    }
    let mut fs = VirtualFileSystem::default();
    let mut deep = String::from(HOME);
    for _ in 0..120 {
        deep.push_str("/d");
        fs.mkdir(&deep, "kali").unwrap();
    }
    let deep_stat = measurement(1000, || {
        std::hint::black_box(fs.stat(&deep, "kali").unwrap());
    });
    fs.write("/home/kali/end", "target", "kali").unwrap();
    for i in (0..40).rev() {
        fs.symlink(
            &format!("/home/kali/s{i}"),
            &if i == 39 {
                "end".into()
            } else {
                format!("s{}", i + 1)
            },
            "kali",
        )
        .unwrap();
    }
    let symlink = measurement(1000, || {
        assert_eq!(fs.read("/home/kali/s0", "kali").unwrap(), "target");
    });
    let result = json!({"schemaVersion":1,"profile":"cargo test / debug; bounded virtual VFS; wall clock; no host program execution","samples":samples,"depth120":deep_stat,"symlinks40":symlink});
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../artifacts/vfs-performance.json");
    std::fs::write(path, serde_json::to_string_pretty(&result).unwrap() + "\n").unwrap();
    println!("{result}");
}
