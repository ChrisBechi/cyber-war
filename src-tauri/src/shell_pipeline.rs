//! Virtual file descriptors. No OS handles, subprocesses or sockets are used.
use crate::{
    error::GameResult,
    terminal_io::Output,
    vfs::{domain, normalize},
    world::WorldState,
};
pub(crate) mod streams;

#[derive(Debug, Clone)]
pub struct IoState {
    pub stdin_tty: bool,
    pub stdout_tty: bool,
    pub stderr_tty: bool,
}
impl Default for IoState {
    fn default() -> Self {
        Self {
            stdin_tty: true,
            stdout_tty: true,
            stderr_tty: true,
        }
    }
}

#[derive(Debug)]
pub(crate) enum Redirect {
    Input(String),
    Output(u8, String, bool),
    Duplicate(u8, u8),
}
#[derive(Default)]
pub(crate) struct Stage {
    pub arguments: Vec<String>,
    pub redirects: Vec<Redirect>,
    pub assignments: std::collections::BTreeMap<String, String>,
    pub assignment_status: Option<i32>,
}

fn parse(parts: &[String]) -> GameResult<Vec<Stage>> {
    let mut stages = vec![Stage::default()];
    let mut i = 0;
    while i < parts.len() {
        let token = &parts[i];
        if token == "\0|" {
            if stages.last().unwrap().arguments.is_empty() {
                return Err(domain("bash: syntax error near unexpected token `|'"));
            }
            stages.push(Stage::default());
        } else if let Some(operator) = token.strip_prefix('\0') {
            let stage = stages.last_mut().unwrap();
            if operator == "&" {
                return Err(domain(
                    "bash: background execution is supported only for archive jobs",
                ));
            }
            i += 1;
            let target = parts
                .get(i)
                .filter(|s| !s.starts_with('\0'))
                .ok_or_else(|| domain("bash: syntax error near unexpected token `newline'"))?;
            let redirect = match operator {
                "<" | "0<" => Redirect::Input(target.clone()),
                ">" | "1>" => Redirect::Output(1, target.clone(), false),
                ">>" | "1>>" => Redirect::Output(1, target.clone(), true),
                "2>" => Redirect::Output(2, target.clone(), false),
                "2>>" => Redirect::Output(2, target.clone(), true),
                ">&" | "1>&" | "2>&" if target == "1" || target == "2" => Redirect::Duplicate(
                    if operator.starts_with('2') { 2 } else { 1 },
                    target.parse().unwrap(),
                ),
                _ => {
                    return Err(domain(format!(
                        "bash: unsupported file descriptor redirection: {operator}{target}"
                    )))
                }
            };
            stage.redirects.push(redirect);
        } else {
            stages.last_mut().unwrap().arguments.push(token.clone());
        }
        i += 1;
    }
    if stages.len() > 16 || stages.len() > 1 && stages.last().unwrap().arguments.is_empty() {
        return Err(domain("bash: invalid pipeline"));
    }
    Ok(stages)
}

#[derive(Clone, Copy)]
enum Destination {
    Stdout,
    Stderr,
    Pipe,
    File(usize),
    Null,
}
struct File {
    path: String,
    host: Option<String>,
    append: bool,
    offset: usize,
}

fn failure(error: impl std::fmt::Display) -> Output {
    let text = error.to_string();
    Output {
        status: if text.contains("command not found") {
            127
        } else {
            1
        },
        stderr: format!("{text}\n"),
        ..Output::default()
    }
}

fn emit(
    world: &mut WorldState,
    destination: Destination,
    text: &str,
    files: &mut [File],
    actor: &str,
    output: &mut Output,
    pipe: &mut String,
) -> GameResult<()> {
    if text.is_empty() {
        return Ok(());
    }
    if matches!(
        destination,
        Destination::Stdout | Destination::Stderr | Destination::Pipe
    ) && output.stdout.len() + output.stderr.len() + pipe.len() + text.len()
        > 4 * 1024 * 1024 - 1024
    {
        return Err(domain("shell: output limit: 4 MiB"));
    }
    match destination {
        Destination::Stdout => {
            output.stdout.push_str(text);
            output.ordered.push((1, text.into()));
            crate::shell::control::emit(1, text);
        }
        Destination::Stderr => {
            output.stderr.push_str(text);
            output.ordered.push((2, text.into()));
            crate::shell::control::emit(2, text);
        }
        Destination::Pipe => pipe.push_str(text),
        Destination::Null => {}
        Destination::File(index) => {
            let file = &mut files[index];
            let fs = match &file.host {
                Some(host) => {
                    &mut world
                        .network
                        .hosts
                        .get_mut(host)
                        .ok_or_else(|| domain("redirect host missing"))?
                        .files
                }
                None => &mut world.vfs,
            };
            let mut content = fs
                .nodes
                .get(&file.path)
                .map(|n| n.content.clone())
                .unwrap_or_default();
            let start = if file.append {
                content.len()
            } else {
                file.offset
            };
            let end = (start + text.len()).min(content.len());
            if !content.is_char_boundary(start) || !content.is_char_boundary(end) {
                return Err(domain(
                    "bash: redirected write splits UTF-8; byte streams are outside this subset",
                ));
            }
            content.replace_range(start..end, text);
            fs.write(&file.path, &content, actor)?;
            file.offset = start + text.len();
        }
    }
    Ok(())
}

pub fn run(world: &mut WorldState, parts: &[String]) -> GameResult<Output> {
    let stages = parse(parts)?; // Reject syntax before opening any files.
    run_stages(world, &stages)
}

pub(crate) fn run_stages(world: &mut WorldState, stages: &[Stage]) -> GameResult<Output> {
    if stages.len() > 1 || stages.first().is_some_and(streams::interactive_or_producer) {
        return streams::run(world, stages);
    }
    let pipeline = stages.len() > 1;
    let original = world.terminal.clone();
    let actor = original.user.clone();
    let mut pipe = String::new();
    let mut output = Output::default();
    for (index, stage) in stages.iter().enumerate() {
        if pipeline {
            world.terminal = original.clone();
        }
        let mut input = if index == 0 {
            original.stdin.clone()
        } else {
            Some(std::mem::take(&mut pipe))
        };
        if input.is_none() && !original.io.stdin_tty {
            input = Some(String::new());
        }
        let mut input_tty = index == 0 && original.io.stdin_tty;
        let mut input_path = None;
        let mut destinations = [
            Destination::Null,
            if index + 1 < stages.len() {
                Destination::Pipe
            } else {
                Destination::Stdout
            },
            Destination::Stderr,
        ];
        let mut files = Vec::new();
        let mut opening_error = None;
        for redirect in &stage.redirects {
            let result = (|| -> GameResult<()> {
                match redirect {
                    Redirect::Duplicate(to, from) => {
                        destinations[*to as usize] = destinations[*from as usize]
                    }
                    Redirect::Input(target) => {
                        let path = normalize(target, &world.terminal.cwd)?;
                        if path != "/dev/null" {
                            world.fs()?.readable(&path, &actor)?;
                        }
                        input_path = Some(path);
                        input_tty = false;
                    }
                    Redirect::Output(fd, target, append) => {
                        let path = normalize(target, &world.terminal.cwd)?;
                        if path == "/dev/null" {
                            destinations[*fd as usize] = Destination::Null;
                        } else {
                            world.fs_mut()?.prepare_output(&path, &actor, *append)?;
                            destinations[*fd as usize] = Destination::File(files.len());
                            files.push(File {
                                path,
                                host: world.terminal.host.clone(),
                                append: *append,
                                offset: 0,
                            });
                        }
                    }
                }
                Ok(())
            })();
            if let Err(error) = result {
                let target = match redirect {
                    Redirect::Input(p) | Redirect::Output(_, p, _) => p.as_str(),
                    _ => "",
                };
                opening_error = Some(format!("bash: {target}: {error}"));
                break;
            }
        }
        if opening_error.is_none() {
            if let Some(path) = input_path {
                match if path == "/dev/null" {
                    Ok(String::new())
                } else {
                    world.fs()?.read(&path, &actor)
                } {
                    Ok(text) => input = Some(text),
                    Err(error) => opening_error = Some(format!("bash: {path}: {error}")),
                }
            }
        }
        let is_tty = |destination| match destination {
            Destination::Stdout => original.io.stdout_tty,
            Destination::Stderr => original.io.stderr_tty,
            _ => false,
        };
        world.terminal.io = IoState {
            stdin_tty: input_tty,
            stdout_tty: is_tty(destinations[1]),
            stderr_tty: is_tty(destinations[2]),
        };
        world.terminal.stdin = input;
        for (key, value) in &stage.assignments {
            world.terminal.env.insert(key.clone(), value.clone());
            world.terminal.shell.unset.remove(key);
            if !stage.arguments.is_empty() {
                world.terminal.exported.insert(key.clone());
            }
        }
        let before_command = world.clone();
        let mut result = if let Some(error) = opening_error {
            failure(error)
        } else if stage.arguments.is_empty() {
            Output {
                status: stage.assignment_status.unwrap_or(0),
                ..Output::default()
            }
        } else {
            match crate::archive::jobs::with_deferral(
                !pipeline && stage.redirects.is_empty(),
                || {
                    crate::shell::control::capture(|| {
                        crate::terminal::dispatch(world, &stage.arguments, &actor)
                    })
                },
            ) {
                Ok(result) => result,
                Err(error) => {
                    *world = before_command.clone();
                    failure(error)
                }
            }
        };
        if (pipeline || !stage.redirects.is_empty())
            && (world.terminal.archive_pending.is_some()
                || world.terminal.package_pending.is_some()
                || world.terminal.nano.is_some())
        {
            *world = before_command;
            result = failure(
                "bash: interactive input requires a foreground command without redirection",
            );
        }
        if let Some(bytes) = result.binary.take() {
            match destinations[1] {
                Destination::File(i) if !files[i].append => {
                    let host = world.terminal.host.clone();
                    world.terminal.host = files[i].host.clone();
                    let size = bytes.len() as u64;
                    let saved = crate::archive::write_bytes(world, &files[i].path, bytes, &actor, size);
                    world.terminal.host = host;
                    if let Err(error) = saved { result = failure(error); }
                }
                Destination::Null => {},
                _ => result = failure("bash: binary stdout requires > VIRTUAL_FILE; binary pipes/append are unsupported"),
            }
        }
        let mut chunks = std::mem::take(&mut result.ordered);
        // Legacy commands still return aggregate streams. Script diagnostics may
        // also be appended after its final ordered command (e.g. a syntax error).
        for (fd, text) in [(1, &result.stdout), (2, &result.stderr)] {
            let emitted: usize = chunks
                .iter()
                .filter(|(f, _)| *f == fd)
                .map(|(_, s)| s.len())
                .sum();
            if emitted < text.len() {
                chunks.push((fd, text[emitted..].to_string()));
            }
        }
        for (fd, text) in chunks {
            if let Err(error) = emit(
                world,
                destinations[fd as usize],
                &text,
                &mut files,
                &actor,
                &mut output,
                &mut pipe,
            ) {
                result.status = 1;
                // A failed write is diagnosed on stderr, including its redirection.
                let error = format!("bash: write error: {error}\n");
                if emit(
                    world,
                    destinations[2],
                    &error,
                    &mut files,
                    &actor,
                    &mut output,
                    &mut pipe,
                )
                .is_err()
                {
                    break;
                }
            }
        }
        output.status = result.status;
        output.archive_job = result.archive_job;
        if !stage.arguments.is_empty() {
            for key in stage.assignments.keys() {
                match original.env.get(key) {
                    Some(value) => {
                        world.terminal.env.insert(key.clone(), value.clone());
                    }
                    None => {
                        world.terminal.env.remove(key);
                    }
                }
                if !original.exported.contains(key) {
                    world.terminal.exported.remove(key);
                }
                if original.shell.unset.contains(key) {
                    world.terminal.shell.unset.insert(key.clone());
                }
            }
        }
        if result.status == 0
            && stage
                .arguments
                .first()
                .is_some_and(|n| !["help", "clear", "echo", "lab"].contains(&n.as_str()))
        {
            world.techniques.insert(stage.arguments[0].clone());
        }
    }
    if pipeline {
        world.terminal = original;
    } else {
        world.terminal.io = original.io;
        world.terminal.stdin = original.stdin;
    }
    Ok(output)
}
