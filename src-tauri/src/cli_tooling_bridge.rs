//! Test-only bridge. Fixed workspace paths; never compiled into the game.
#[cfg(not(test))]
compile_error!("CLI development bridge must never be compiled into production.");

use crate::{cli_contract::Tty, terminal, world::WorldState};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
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
    io: Option<IoFixture>,
    interaction: Option<InteractionFixture>,
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
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct IoFixture {
    stdin_path: Option<String>,
    stdout_path: Option<String>,
    stderr_path: Option<String>,
    #[serde(default)]
    append: bool,
    #[serde(default)]
    closed_consumer: bool,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct InteractionFixture {
    schema_version: u8,
    #[serde(default)]
    writers: Vec<String>,
    steps: Vec<InteractionStep>,
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
enum InteractionStep {
    StopWriter {
        writer: String,
        signal: Option<crate::shell::signals::VirtualSignal>,
    },
    MutateBatch {
        steps: Vec<InteractionStep>,
    },
    Wait,
    Await {
        #[serde(default, rename = "stdoutBytes")]
        stdout_bytes: usize,
        #[serde(default, rename = "stderrBytes")]
        stderr_bytes: usize,
    },
    Mutate {
        operation: String,
        path: String,
        target: Option<String>,
        hex: Option<String>,
        size: Option<usize>,
        mode: Option<u16>,
    },
    Write {
        hex: String,
    },
    Expect {
        #[serde(rename = "stdoutHex")]
        stdout_hex: String,
    },
    Eof,
    Signal {
        signal: crate::shell::signals::VirtualSignal,
        #[serde(rename = "waitFor")]
        wait_for: Option<String>,
    },
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
    #[serde(default)]
    hardlinks: BTreeMap<String, String>,
    #[serde(default)]
    symlinks: BTreeMap<String, String>,
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
fn run_interaction(
    key: &str,
    registration: &crate::shell::control::Registration,
    interaction: &InteractionFixture,
    events: std::sync::mpsc::Receiver<crate::cli_contract::Event>,
    observations: &mut Vec<Value>,
    work: impl FnOnce() -> terminal::CommandResult + Send,
) -> terminal::CommandResult {
    use crate::{cli_contract::Event, shell::control};
    assert_eq!(interaction.schema_version, 1);
    let raw = registration.observe_bytes();
    std::thread::scope(|scope| {
        let worker = scope.spawn(|| control::run(registration, work));
        let mut stdout = Vec::new();
        let mut waiting = false;
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            for step in &interaction.steps {
                match step {
                    InteractionStep::Write { hex } => {
                        waiting = false;
                        assert!(control::input(
                            key,
                            Some(String::from_utf8(unhex(hex)).unwrap())
                        ));
                    }
                    InteractionStep::Eof => {
                        waiting = false;
                        assert!(control::input(key, None));
                    }
                    InteractionStep::Expect { stdout_hex } => {
                        let expected = unhex(stdout_hex);
                        while stdout.len() < expected.len() {
                            match events
                                .recv_timeout(std::time::Duration::from_secs(5))
                                .expect("TTY stdout barrier")
                            {
                                Event::Stdout(text) => {
                                    control::acknowledge(key, text.len());
                                    let (fd, bytes) = raw
                                        .recv_timeout(std::time::Duration::from_secs(5))
                                        .expect("raw TTY stdout");
                                    assert_eq!(fd, 1);
                                    stdout.extend(bytes);
                                }
                                Event::WaitingForInput => waiting = true,
                                _ => {}
                            }
                        }
                        assert_eq!(stdout, expected, "TTY stdout barrier {key}");
                        observations
                            .push(json!({"stdoutHex":stdout_hex,"running":!worker.is_finished()}));
                    }
                    InteractionStep::Signal { signal, wait_for } => {
                        if wait_for.as_deref() == Some("output") {
                            let deadline =
                                std::time::Instant::now() + std::time::Duration::from_secs(5);
                            while !control::output_blocked(key) {
                                assert!(
                                    !worker.is_finished() && std::time::Instant::now() < deadline,
                                    "output barrier {key}"
                                );
                                std::thread::sleep(std::time::Duration::from_millis(1));
                            }
                            let pids = control::process_ids(key);
                            assert_eq!(pids.len(), 1);
                            assert!(control::signal(key, Some(pids[0]), *signal));
                            continue;
                        }
                        while !waiting || !control::stdin_drained_and_waiting(key) {
                            match events
                                .recv_timeout(std::time::Duration::from_secs(5))
                                .expect("TTY blocked-read barrier")
                            {
                                Event::WaitingForInput => waiting = true,
                                Event::Stdout(text) => {
                                    control::acknowledge(key, text.len());
                                    let (fd, bytes) = raw
                                        .recv_timeout(std::time::Duration::from_secs(5))
                                        .expect("raw TTY stdout");
                                    assert_eq!(fd, 1);
                                    stdout.extend(bytes);
                                }
                                _ => {}
                            }
                        }
                        let pids = control::process_ids(key);
                        assert_eq!(pids.len(), 1, "signal targets one virtual process");
                        assert!(control::signal(key, Some(pids[0]), *signal));
                    }
                    _ => panic!("Version 2 step in TTY protocol"),
                }
            }
        }));
        if outcome.is_err() {
            control::cancel(key);
        }
        let result = worker.join().unwrap();
        if let Err(error) = outcome {
            std::panic::resume_unwind(error);
        }
        result
    })
}
#[path = "cli_follow_bridge.rs"]
mod follow;
#[test]
#[ignore = "development case capture; invoked explicitly by cli:compat"]
fn cli_tooling_capture() {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../artifacts/cli-case-request.json");
    let request: Request = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(request.schema_version, 2);
    // Keep only one case snapshot resident. The matrix can contain hundreds of
    // complete VFS snapshots; collecting all Value trees multiplied memory use.
    use std::io::Write;
    let target = path.with_file_name("cli-case-actual.json");
    let cases_directory = target.with_extension("cases");
    std::fs::create_dir_all(&cases_directory).unwrap();
    let temporary = path.with_file_name("cli-case-actual.partial.json");
    let mut writer = std::io::BufWriter::new(std::fs::File::create(&temporary).unwrap());
    writer
        .write_all(b"{\"schemaVersion\":3,\"cases\":[")
        .unwrap();
    for (index, mut case) in request.cases.into_iter().enumerate() {
        if index > 0 {
            writer.write_all(b",").unwrap();
        }
        if index % 32 == 0 {
            eprintln!("capture case {index}: {}", case.id);
        }
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
        for (path, target) in case.fixture.hardlinks {
            w.vfs.link(&target, &path, "kali").unwrap();
        }
        for (path, target) in case.fixture.symlinks {
            w.vfs.symlink(&path, &target, "kali").unwrap();
        }
        for setup in case.fixture.setup {
            let result = terminal::execute(&mut w, &setup);
            assert_eq!(result.exit_code, 0, "{} setup: {}", case.id, result.stderr);
        }
        w.vfs.directory(&case.cwd, "kali").unwrap();
        w.terminal.cwd = case.cwd;
        w.terminal.env = case.env;
        // Structured executable requests model a child process environment.
        // Shell-script fixtures also use env for unexported initial variables.
        if case.script.is_none() {
            w.terminal.exported.extend(w.terminal.env.keys().cloned());
        }
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
        if let Some(interaction) = &case.interaction {
            for (index, name) in interaction.writers.iter().enumerate() {
                let pid = 100_000 + index as u32;
                w.processes.push(crate::world::VirtualProcess {
                    pid,
                    name: name.clone(),
                    user: "kali".into(),
                    running: true,
                });
                for arg in &mut case.argv {
                    *arg = arg.replace(&format!("{{{name}}}"), &pid.to_string());
                }
            }
        }
        let (sender, receiver) = std::sync::mpsc::channel();
        let output = case.interaction.as_ref().map(|_| {
            tauri::ipc::Channel::new(move |body| {
                sender
                    .send(body.deserialize::<crate::cli_contract::Event>().unwrap())
                    .unwrap();
                Ok(())
            })
        });
        let control = crate::shell::control::register_output(&case.id, output);
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
        let closed_consumer = case.io.as_ref().is_some_and(|io| io.closed_consumer);
        let work = |w: &mut WorldState| {
            if let Some(script) = case.script {
                terminal::execute(w, &script)
            } else if case.transport.as_deref().is_some_and(|t| t != "direct") || case.io.is_some()
            {
                let quote = |value: &str| format!("'{}'", value.replace('\'', "'\\''"));
                let mut command = std::iter::once(case.invocation.unwrap_or(case.command))
                    .chain(case.argv)
                    .map(|s| quote(&s))
                    .collect::<Vec<_>>()
                    .join(" ");
                if let Some(io) = case.io {
                    if let Some(path) = io.stdin_path {
                        command.push_str(&format!(" < {}", quote(&path)));
                    }
                    if let Some(path) = io.stdout_path {
                        command.push_str(&format!(
                            " {} {}",
                            if io.append { ">>" } else { ">" },
                            quote(&path)
                        ));
                    }
                    if let Some(path) = io.stderr_path {
                        command.push_str(&format!(" 2> {}", quote(&path)));
                    }
                }
                let script = if case.transport.as_deref() == Some("pipe") {
                    format!("{command} | /usr/bin/cat")
                } else if case.transport.as_deref() == Some("redirect") {
                    format!("{command} > /home/kali/reference-output")
                } else {
                    command
                };
                terminal::execute(w, &script)
            } else {
                let parts = std::iter::once(case.invocation.unwrap_or(case.command))
                    .chain(case.argv)
                    .collect();
                terminal::execute_parts(w, Ok(parts))
            }
        };
        let started = std::time::Instant::now();
        let mut observations = Vec::new();
        let result = if let Some(interaction) = case.interaction {
            if interaction.schema_version == 2 {
                follow::run(
                    &case.id,
                    &control,
                    &interaction,
                    receiver,
                    &mut observations,
                    &mut w,
                    work,
                )
            } else {
                run_interaction(
                    &case.id,
                    &control,
                    &interaction,
                    receiver,
                    &mut observations,
                    || work(&mut w),
                )
            }
        } else if closed_consumer {
            crate::shell_pipeline::streams::with_closed_consumer(|| work(&mut w))
        } else if case.input_events.is_empty() {
            work(&mut w)
        } else {
            crate::shell::control::run(&control, || work(&mut w))
        };
        assert_eq!(
            w.vfs.open_handle_count(),
            0,
            "leaked descriptor: {}",
            case.id
        );
        assert_eq!(w.vfs.watcher_count(), 0, "leaked watcher: {}", case.id);
        assert_eq!(w.scheduler.timer_count(), 0, "leaked timer: {}", case.id);
        assert!(w.scheduler.waiting.is_empty(), "leaked wait: {}", case.id);
        assert!(
            crate::shell::control::process_ids(&case.id).is_empty(),
            "leaked process registration: {}",
            case.id
        );
        drop(control);
        if case.roundtrip {
            let json = serde_json::to_string(&w.vfs).unwrap();
            w.vfs = serde_json::from_str(&json).unwrap();
        }
        w.vfs.collect();
        w.vfs.check_invariants().unwrap();
        let row = serde_json::to_vec(&json!({"id":case.id,"stdout":result.stdout,"stdoutHex": result.stdout_bytes.iter().map(|b| format!("{b:02x}")).collect::<String>(),"durationMs":started.elapsed().as_secs_f64()*1000.0,"stderr":result.stderr,
            "stderrHex":result.stderr_bytes.iter().map(|b| format!("{b:02x}")).collect::<String>(),"exitCode":result.exit_code,"termination":result.termination,"observations":observations,"ordered":result.ordered,"before":before,"after":snapshot(&w),"tty":tty,
            "files":w.vfs.nodes.iter().filter(|(p,n)| p.starts_with("/home/kali/") && n.kind == "file").map(|(p,n)| (p.clone(), n.blob.as_ref().map(|b| w.blobs[&b.hash].as_slice()).unwrap_or(n.content.as_bytes()).iter().map(|b|format!("{b:02x}")).collect::<String>())).collect::<BTreeMap<_,_>>()})).unwrap();
        let sha256 = format!("{:x}", Sha256::digest(&row));
        std::fs::write(cases_directory.join(format!("{sha256}.json")), row).unwrap();
        serde_json::to_writer(&mut writer, &json!({"id":case.id,"sha256":sha256})).unwrap();
    }
    writer.write_all(b"]}").unwrap();
    writer.flush().unwrap();
    drop(writer);
    std::fs::rename(temporary, target).unwrap();
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
