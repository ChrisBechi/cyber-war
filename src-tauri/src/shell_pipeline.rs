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
    pub assignment_order: Vec<String>,
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
    host: Option<String>,
    handle: u64,
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

fn emit_bytes(
    world: &mut WorldState,
    destination: Destination,
    data: &[u8],
    files: &mut [File],
    output: &mut Output,
    pipe: &mut Vec<u8>,
) -> GameResult<()> {
    if data.is_empty() {
        return Ok(());
    }
    if output
        .byte_ordered
        .iter()
        .map(|(_, b)| b.len())
        .sum::<usize>()
        + pipe.len()
        + data.len()
        > 4 * 1024 * 1024 - 1024
        && !matches!(destination, Destination::File(_) | Destination::Null)
    {
        return Err(domain("shell: output limit: 4 MiB"));
    }
    match destination {
        Destination::Stdout | Destination::Stderr => {
            for data in data.chunks(4096) {
                let fd = if matches!(destination, Destination::Stdout) {
                    1
                } else {
                    2
                };
                // Text is presentation only. byte_ordered/binary retain the exact command bytes.
                let display = String::from_utf8_lossy(data);
                if !crate::shell::control::emit_chunk(fd, &display) {
                    break;
                }
                if fd == 1 {
                    output.stdout.push_str(&display);
                    output
                        .binary
                        .get_or_insert_with(Vec::new)
                        .extend_from_slice(data);
                } else {
                    output.stderr.push_str(&display);
                }
                output.ordered.push((fd, display.to_string()));
                output.byte_ordered.push((fd, data.to_vec()));
            }
        }
        Destination::Pipe => pipe.extend_from_slice(data),
        Destination::Null => {}
        Destination::File(index) => {
            let file = &files[index];
            let previous = world.terminal.host.clone();
            world.terminal.host = file.host.clone();
            let result = crate::coreutils::io::write_handle(world, file.handle, data);
            world.terminal.host = previous;
            result?;
        }
    }
    Ok(())
}
fn output_chunks(mut result: Output) -> Vec<(u8, Vec<u8>)> {
    if !result.byte_ordered.is_empty() {
        return result.byte_ordered;
    }
    let mut chunks = Vec::new();
    if let Some(bytes) = result.binary.take() {
        chunks.push((1, bytes));
    }
    for (fd, text) in [(1, &result.stdout), (2, &result.stderr)] {
        let emitted: usize = result
            .ordered
            .iter()
            .filter(|(f, _)| *f == fd)
            .map(|(_, s)| s.len())
            .sum();
        if emitted < text.len() {
            result.ordered.push((fd, text[emitted..].to_owned()));
        }
    }
    chunks.extend(
        result
            .ordered
            .into_iter()
            .map(|(fd, text)| (fd, text.into_bytes())),
    );
    chunks
}

fn open_output(
    world: &mut WorldState,
    path: String,
    actor: &str,
    append: bool,
) -> GameResult<File> {
    let host = world.terminal.host.clone();
    let handle = world.fs_mut()?.open(
        &path,
        crate::vfs::OpenFlags {
            write: true,
            create: true,
            truncate: !append,
            append,
            ..Default::default()
        },
        0o666,
        actor,
    )?;
    Ok(File { host, handle })
}
fn read_input(world: &mut WorldState, path: &str, actor: &str) -> GameResult<Vec<u8>> {
    crate::coreutils::io::read(world, path, actor, &mut None)
}

fn close_files(world: &mut WorldState, files: &[File]) -> GameResult<()> {
    for file in files {
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
        fs.close(file.handle)?;
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
    let mut pipe = Vec::new();
    let mut output = Output::default();
    for (index, stage) in stages.iter().enumerate() {
        if pipeline {
            world.terminal = original.clone();
        }
        let mut input = if index == 0 {
            original
                .stdin_bytes
                .clone()
                .or_else(|| original.stdin.as_ref().map(|s| s.as_bytes().to_vec()))
        } else {
            Some(std::mem::take(&mut pipe))
        };
        if input.is_none() && !original.io.stdin_tty {
            input = Some(Vec::new());
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
                        world.fs()?.readable(&path, &actor)?;
                        input_path = Some(path);
                        input_tty = false;
                    }
                    Redirect::Output(fd, target, append) => {
                        let path = normalize(target, &world.terminal.cwd)?;
                        destinations[*fd as usize] = Destination::File(files.len());
                        files.push(open_output(world, path, &actor, *append)?);
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
                match read_input(world, &path, &actor) {
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
        world.terminal.stdin = input
            .as_ref()
            .and_then(|b| String::from_utf8(b.clone()).ok());
        world.terminal.stdin_bytes = input;
        for key in &stage.assignment_order {
            let value = &stage.assignments[key];
            world.terminal.env.insert(key.clone(), value.clone());
            world.terminal.shell.unset.remove(key);
            if !stage.arguments.is_empty() {
                world.terminal.exported.insert(key.clone());
                if !world.terminal.shell.environment_order.contains(key) {
                    world.terminal.shell.environment_order.push(key.clone());
                }
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
        let status = result.status;
        let archive_job = result.archive_job;
        let chunks = output_chunks(result);
        let mut result = Output {
            status,
            archive_job,
            ..Default::default()
        };
        for (fd, text) in chunks {
            if let Err(error) = emit_bytes(
                world,
                destinations[fd as usize],
                &text,
                &mut files,
                &mut output,
                &mut pipe,
            ) {
                result.status = 1;
                // A failed write is diagnosed on stderr, including its redirection.
                let error = format!("bash: write error: {error}\n");
                if emit_bytes(
                    world,
                    destinations[2],
                    error.as_bytes(),
                    &mut files,
                    &mut output,
                    &mut pipe,
                )
                .is_err()
                {
                    break;
                }
            }
        }
        close_files(world, &files)?;
        output.status = result.status;
        output.archive_job = result.archive_job;
        if !stage.arguments.is_empty() {
            for key in stage.assignments.keys() {
                if !original.shell.environment_order.contains(key) {
                    world.terminal.shell.environment_order.retain(|k| k != key);
                }
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
        world.terminal.stdin_bytes = original.stdin_bytes;
    }
    Ok(output)
}
