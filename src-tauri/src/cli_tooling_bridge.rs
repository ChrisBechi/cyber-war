//! Test-only bridge. Fixed workspace paths; never compiled into the game.
#[cfg(not(test))]
compile_error!("CLI development bridge must never be compiled into production.");

use crate::{cli_contract::Tty, terminal, world::WorldState};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;

fn artifact(name: &str, value: &Value) {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../artifacts");
    std::fs::create_dir_all(&dir).expect("create development artifacts");
    std::fs::write(dir.join(name), serde_json::to_string_pretty(value).unwrap())
        .expect("write development artifact");
}

#[test]
#[ignore = "development registry export; invoked explicitly by cli:inventory"]
fn cli_tooling_registry_export() {
    let mut packages = Vec::new();
    for repo in crate::packages::repository::REPOSITORIES.values() {
        for (_, entry) in &repo.entries {
            packages.push(json!(entry.package));
        }
    }
    packages.sort_by_key(|p| format!("{}:{}", p["name"], p["version"]));
    artifact(
        "cli-runtime-registry.json",
        &json!({
            "schemaVersion": 1,
            "native": terminal::COMMANDS,
            "shellOnly": crate::shell::SHELL_ONLY,
            "names": *crate::command_registry::NAMES,
            "packages": packages,
            "baselinePackages": crate::packages::repository::BASELINE.iter().map(|p| p.0).collect::<Vec<_>>(),
            "dynamicBindings": ["builtin", "catalog:*", "netscan", "wireless", "iot"]
        }),
    );
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Request {
    schema_version: u8,
    cases: Vec<Case>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Case {
    id: String,
    command: String,
    argv: Vec<String>,
    script: Option<String>,
    stdin: Option<String>,
    env: BTreeMap<String, String>,
    cwd: String,
    tty: Tty,
    fixture: Fixture,
    #[serde(default)]
    input_events: Vec<crate::cli_contract::Input>,
}
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Fixture {
    #[serde(default)]
    files: BTreeMap<String, String>,
    #[serde(default)]
    directories: Vec<String>,
    #[serde(default)]
    modes: BTreeMap<String, u16>,
    #[serde(default)]
    setup: Vec<String>,
}
fn snapshot(w: &WorldState) -> Value {
    json!({"vfs": w.vfs.nodes, "processes": w.processes, "network": w.network,
        "packages": w.packages, "cwd": w.terminal.cwd, "user": w.terminal.user})
}
#[test]
#[ignore = "development case capture; invoked explicitly by cli:compat"]
fn cli_tooling_capture() {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../artifacts/cli-case-request.json");
    let request: Request = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(request.schema_version, 2);
    let mut results = Vec::new();
    for case in request.cases {
        let mut w = WorldState::new("kali", "lifeos").unwrap();
        for dir in case.fixture.directories {
            w.vfs.mkdir(&dir, "kali").unwrap();
        }
        for (path, text) in case.fixture.files {
            w.vfs.write(&path, &text, "kali").unwrap();
        }
        for (path, mode) in case.fixture.modes {
            w.vfs.chmod(&path, "kali", mode).unwrap();
        }
        for setup in case.fixture.setup {
            let result = terminal::execute(&mut w, &setup);
            assert_eq!(result.exit_code, 0, "{} setup: {}", case.id, result.stderr);
        }
        w.vfs.directory(&case.cwd, "kali").unwrap();
        w.terminal.cwd = case.cwd;
        w.terminal.env = case.env;
        w.terminal.stdin = case.stdin;
        w.terminal.presentation.columns = case.tty.columns;
        w.terminal.io.stdin_tty = case.tty.is_tty;
        w.terminal.io.stdout_tty = case.tty.is_tty;
        w.terminal.io.stderr_tty = case.tty.is_tty;
        // Rows/ANSI/interactive are transport contract metadata; existing CLI
        // engines currently consume columns and descriptor TTY state only.
        let tty = json!({"isTTY":case.tty.is_tty,"rows":case.tty.rows,"columns":case.tty.columns,"ansiSupport":case.tty.ansi_support,"interactive":case.tty.interactive});
        let before = snapshot(&w);
        let control = crate::shell::control::register(&case.id);
        for input in &case.input_events {
            match input {
                crate::cli_contract::Input::Stdin(text) => {
                    assert!(crate::shell::control::input(&case.id, Some(text.clone())))
                }
                crate::cli_contract::Input::Eof => {
                    assert!(crate::shell::control::input(&case.id, None))
                }
                crate::cli_contract::Input::Cancel => crate::shell::control::cancel(&case.id),
            }
        }
        let work = || {
            if let Some(script) = case.script {
                terminal::execute(&mut w, &script)
            } else {
                let parts = std::iter::once(case.command).chain(case.argv).collect();
                terminal::execute_parts(&mut w, Ok(parts))
            }
        };
        let result = if case.input_events.is_empty() {
            work()
        } else {
            crate::shell::control::run(&control, work)
        };
        drop(control);
        results.push(json!({"id":case.id,"stdout":result.stdout,"stderr":result.stderr,
            "exitCode":result.exit_code,"ordered":result.ordered,"before":before,"after":snapshot(&w),"tty":tty}));
    }
    artifact(
        "cli-case-actual.json",
        &json!({"schemaVersion":2,"cases":results}),
    );
}

#[test]
fn cli_compat_registry_facade_covers_aliases_and_save_availability() {
    let mut w = WorldState::new("kali", "lifeos").unwrap();
    assert!(crate::command_registry::NAMES.contains("."));
    assert!(crate::command_registry::NAMES.contains("airmon-ng"));
    assert!(crate::command_registry::available(&w, "cat"));
    assert!(!crate::command_registry::available(&w, "netscan"));
    w.packages.installed.remove("coreutils");
    assert!(!crate::command_registry::available(&w, "cat"));
    assert!(!serde_json::to_string(&w)
        .unwrap()
        .contains("verificationSummary"));
    let tool = crate::software::by_command("airmon-ng").unwrap();
    assert_eq!(tool.package, "aircrack-ng");
}
