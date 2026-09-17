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
    invocation: Option<String>,
    transport: Option<String>,
    process: Option<ProcessFixture>,
    argv: Vec<String>,
    script: Option<String>,
    #[serde(default)]
    roundtrip: bool,
    stdin: Option<String>,
    stdin_hex: Option<String>,
    env: BTreeMap<String, String>,
    cwd: String,
    tty: Tty,
    fixture: Fixture,
    #[serde(default)]
    input_events: Vec<crate::cli_contract::Input>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProcessFixture {
    actor: Option<String>,
    uid: Option<u32>,
    #[serde(default)]
    username: UsernameFixture,
    environment: Option<Vec<(String, String)>>,
}
#[derive(Default)]
enum UsernameFixture {
    #[default]
    Inherit,
    Lookup(Option<String>),
}
impl<'de> Deserialize<'de> for UsernameFixture {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Option::<String>::deserialize(deserializer).map(Self::Lookup)
    }
}
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Fixture {
    #[serde(default)]
    files: BTreeMap<String, String>,
    #[serde(default)]
    bytes: BTreeMap<String, String>,
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
fn unhex(text: &str) -> Vec<u8> {
    text.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
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
        for (path, hex) in case.fixture.bytes {
            let bytes = unhex(&hex);
            let size = bytes.len() as u64;
            crate::archive::write_bytes(&mut w, &path, bytes, "kali", size).unwrap();
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
        if let Some(process) = case.process {
            if let Some(actor) = process.actor {
                w.terminal.user = actor;
            }
            if let Some(uid) = process.uid {
                w.vfs.set_identity(
                    &w.terminal.user,
                    crate::vfs::Identity {
                        uid,
                        gid: uid,
                        groups: std::collections::BTreeSet::from([uid]),
                    },
                );
            }
            if let UsernameFixture::Lookup(username) = process.username {
                let uid = w.vfs.identity(&w.terminal.user).uid;
                let passwd = username.map_or_else(String::new, |name| {
                    format!("{name}:x:{uid}:{uid}::/home/kali:/bin/sh\n")
                });
                w.vfs.write("/etc/passwd", &passwd, "root").unwrap();
            }
            if let Some(env) = process.environment {
                w.terminal.shell.environment_order = env.iter().map(|(k, _)| k.clone()).collect();
                let defaults = terminal::virtual_env(&w, &w.terminal.user);
                w.terminal.shell.unset.extend(
                    defaults
                        .keys()
                        .filter(|k| !env.iter().any(|(n, _)| n == *k))
                        .cloned(),
                );
                w.terminal.env = env.iter().cloned().collect();
                w.terminal.exported = env.iter().map(|(k, _)| k.clone()).collect();
            }
        }
        w.terminal.stdin = case.stdin;
        w.terminal.stdin_bytes = case.stdin_hex.as_deref().map(unhex);
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
            } else if case.transport.as_deref().is_some_and(|t| t != "direct") {
                let quote = |value: &str| format!("'{}'", value.replace('\'', "'\\''"));
                let command = std::iter::once(case.invocation.unwrap_or(case.command))
                    .chain(case.argv)
                    .map(|s| quote(&s))
                    .collect::<Vec<_>>()
                    .join(" ");
                let script = if case.transport.as_deref() == Some("pipe") {
                    format!("{command} | /usr/bin/cat")
                } else {
                    format!("{command} > /home/kali/reference-output")
                };
                terminal::execute(&mut w, &script)
            } else {
                let parts = std::iter::once(case.invocation.unwrap_or(case.command))
                    .chain(case.argv)
                    .collect();
                terminal::execute_parts(&mut w, Ok(parts))
            }
        };
        let started = std::time::Instant::now();
        let result = if case.input_events.is_empty() {
            work()
        } else {
            crate::shell::control::run(&control, work)
        };
        drop(control);
        if case.roundtrip {
            let json = serde_json::to_string(&w.vfs).unwrap();
            w.vfs = serde_json::from_str(&json).unwrap();
        }
        w.vfs.collect();
        w.vfs.check_invariants().unwrap();
        results.push(json!({"id":case.id,"stdout":result.stdout,"stdoutHex": result.stdout_bytes.iter().map(|b| format!("{b:02x}")).collect::<String>(),"durationMs":started.elapsed().as_secs_f64()*1000.0,"stderr":result.stderr,
            "stderrHex":result.stderr_bytes.iter().map(|b| format!("{b:02x}")).collect::<String>(),"exitCode":result.exit_code,"ordered":result.ordered,"before":before,"after":snapshot(&w),"tty":tty}));
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
