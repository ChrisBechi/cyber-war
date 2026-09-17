//! Bash 5.2-compatible virtual subset; never invokes an operating-system shell.
pub mod control;
mod executor;
mod expansion;
#[cfg(test)]
mod foundation_tests;
#[cfg(test)]
mod performance_tests;
pub mod signals;
pub mod syntax;
pub mod tty;
use crate::{
    error::GameResult,
    terminal_io::Output,
    vfs::{domain, normalize},
    world::WorldState,
};
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicU32, Ordering};

pub const SHELL_ONLY: &[&str] = &[".", "return", "unset", "umask"];
pub const HELP: &str = "Cyber War virtual shell — Bash 5.2 subset\nbash|sh SCRIPT [ARGS...] or -c COMMANDS [NAME [ARGS...]]; source|. FILE [ARGS...]\nQuotes, escapes, parameters, assignments/export/unset, IFS splitting, ~, VFS globbing and $(commands).\nLists: ; newline && ||. Pipes and < > >> 2> 2>> 2>&1 1>&2, applied left to right.\nChild shells/pipelines isolate shell context; source shares it. Background jobs remain archive-only.\nLimits: 64 KiB source, 8 nesting levels, 16 pipeline stages, 4 MiB captured output.\nNo loops/functions, eval, exec, here-documents, arithmetic, backticks, advanced parameter operators or arbitrary descriptors. No host execution.\n";
static NEXT_PID: AtomicU32 = AtomicU32::new(10000);
pub fn next_pid() -> u32 {
    NEXT_PID.fetch_add(1, Ordering::Relaxed)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flow {
    Exit,
    Return,
}
#[derive(Clone, Debug)]
pub struct Runtime {
    pub pid: u32,
    pub umask: u16,
    pub last_background: Option<u32>,
    pub name: String,
    pub args: Vec<String>,
    pub source_depth: usize,
    pub flow: Option<Flow>,
    pub unset: BTreeSet<String>,
    /// Order of the virtual process environment, independent of key lookup.
    pub environment_order: Vec<String>,
    pub continuation: String,
}
impl Default for Runtime {
    fn default() -> Self {
        Self {
            pid: next_pid(),
            umask: 0o022,
            last_background: None,
            name: "bash".into(),
            args: Vec::new(),
            source_depth: 0,
            flow: None,
            unset: BTreeSet::new(),
            environment_order: Vec::new(),
            continuation: String::new(),
        }
    }
}
pub fn identifier(value: &str) -> bool {
    let mut chars = value.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}
pub fn is_builtin(name: &str) -> bool {
    [
        ".", "source", "return", "exit", "cd", "pwd", "echo", "export", "unset", "true", "false",
        "type", "command", "umask",
    ]
    .contains(&name)
}
pub fn path_file(world: &WorldState, name: &str, actor: &str) -> Option<String> {
    let env = crate::terminal::virtual_env(world, actor);
    let mut denied = None;
    for directory in env.get("PATH")?.split(':') {
        let path = normalize(
            &format!(
                "{}/{name}",
                if directory.is_empty() { "." } else { directory }
            ),
            &world.terminal.cwd,
        );
        let Ok(path) = path else {
            continue;
        };
        let fs = world.fs().ok()?;
        if let Ok(node) = fs.stat(&path, actor) {
            if node.kind == "file" && fs.allowed(node, actor, 1) {
                return Some(path);
            }
        }
        if fs.nodes.contains_key(&path) && denied.is_none() {
            denied = Some(path);
        }
    }
    denied
}
pub fn execute(world: &mut WorldState, source: &str) -> GameResult<Output> {
    match syntax::parse(source) {
        syntax::ParseResult::Complete(list) => executor::execute_list(world, &list),
        syntax::ParseResult::Incomplete(error) | syntax::ParseResult::Error(error) => {
            Err(domain(error.to_string()))
        }
    }
}
pub fn invoke(
    world: &mut WorldState,
    name: &str,
    args: &[String],
    actor: &str,
) -> GameResult<Output> {
    if args.first().is_some_and(|a| a == "--help" || a == "-h") {
        return Ok(Output::success(HELP.into()));
    }
    if args.first().is_some_and(|a| a == "--version") {
        return Ok(Output::success(
            "Cyber War virtual shell (not GNU Bash)\n".into(),
        ));
    }
    if world.terminal.shell_depth >= syntax::MAX_DEPTH {
        return Err(domain("shell: maximum nesting depth exceeded"));
    }
    let sourced = matches!(name, "source" | ".");
    let (script, content, positional) = if args.first().is_some_and(|a| a == "-c") && !sourced {
        (
            args.get(2).cloned().unwrap_or_else(|| name.into()),
            args.get(1)
                .ok_or_else(|| domain("shell: -c requires a command string"))?
                .clone(),
            args.get(3..).unwrap_or_default().to_vec(),
        )
    } else {
        let values = if args.first().is_some_and(|a| a == "--") {
            &args[1..]
        } else {
            args
        };
        let script = values
            .first()
            .ok_or_else(|| domain("shell: script path required"))?;
        if script.starts_with('-') {
            return Err(domain("shell: unsupported interpreter option"));
        }
        let path = normalize(script, &world.terminal.cwd)?;
        (
            script.clone(),
            world.fs()?.read(&path, actor)?,
            if sourced && values.len() == 1 {
                world.terminal.shell.args.clone()
            } else {
                values[1..].to_vec()
            },
        )
    };
    if let Some(shebang) = content.lines().next().and_then(|s| s.strip_prefix("#!")) {
        if ![
            "/bin/bash",
            "/bin/sh",
            "/usr/bin/bash",
            "/usr/bin/sh",
            "/usr/bin/env bash",
            "/usr/bin/env sh",
        ]
        .contains(&shebang.trim())
        {
            return Err(domain("shell: unsupported shebang interpreter"));
        }
    }
    let original = world.terminal.clone();
    if !sourced {
        world.terminal.last_status = 0;
        world.terminal.env.retain(|key, _| {
            original.exported.contains(key)
                || ["HOME", "PWD", "OLDPWD", "PATH", "USER", "LANG"].contains(&key.as_str())
        });
        world.terminal.shell.pid = next_pid();
        world.terminal.shell.name = script;
        world.terminal.shell.source_depth = 0;
    } else {
        world.terminal.shell.source_depth += 1;
    }
    world.terminal.shell.args = positional;
    world.terminal.shell.flow = None;
    world.terminal.shell_depth += 1;
    world.terminal.user = actor.into();
    let result = execute(world, &content);
    if sourced {
        let flow = world.terminal.shell.flow;
        world.terminal.shell.args = original.shell.args;
        world.terminal.shell.source_depth = original.shell.source_depth;
        world.terminal.shell.flow = if flow == Some(Flow::Exit) {
            flow
        } else {
            original.shell.flow
        };
        world.terminal.shell_depth = original.shell_depth;
        world.terminal.user = original.user;
    } else {
        world.terminal = original;
    }
    Ok(result.unwrap_or_else(|e| Output {
        stderr: format!("{e}\n"),
        status: 2,
        ..Output::default()
    }))
}
#[cfg(test)]
pub fn words(
    input: &str,
    env: &std::collections::BTreeMap<String, String>,
    script: &str,
    args: &[String],
    status: i32,
) -> GameResult<Vec<String>> {
    let syntax::ParseResult::Complete(list) = syntax::parse(input) else {
        return Err(domain("invalid syntax"));
    };
    let mut world = WorldState::new("test", "pc")?;
    world.terminal.env = env.clone();
    world.terminal.last_status = status;
    world.terminal.shell.name = script.into();
    world.terminal.shell.args = args.to_vec();
    let mut values = Vec::new();
    for item in list.items {
        for (_, pipeline) in item.pipelines {
            for command in pipeline.commands {
                for item in command.words {
                    values.extend(expansion::word(&mut world, &item, false)?.values);
                }
            }
        }
    }
    Ok(values)
}
