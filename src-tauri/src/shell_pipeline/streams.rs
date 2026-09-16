//! A deterministic cooperative scheduler with bounded byte channels. There are
//! no host threads/processes per stage, and an empty buffer is distinct from EOF.
use super::*;
use crate::shell::control::{self, Read};
use std::collections::VecDeque;

const CHUNK: usize = 4096;
const CAPACITY: usize = 64 * 1024;
const ADAPTER_LIMIT: usize = 4 * 1024 * 1024;
#[cfg(test)]
thread_local! { static LAST_PEAK: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }
#[cfg(test)]
pub(crate) fn last_peak() -> usize {
    LAST_PEAK.with(std::cell::Cell::get)
}
#[derive(Default)]
struct Pipe {
    chunks: VecDeque<String>,
    bytes: usize,
    writer_closed: bool,
    reader_closed: bool,
    peak: usize,
}
impl Pipe {
    fn push(&mut self, text: String) -> Result<bool, ()> {
        if self.reader_closed {
            return Err(());
        }
        if self.bytes + text.len() > CAPACITY {
            return Ok(false);
        }
        self.bytes += text.len();
        self.peak = self.peak.max(self.bytes);
        self.chunks.push_back(text);
        Ok(true)
    }
    fn read(&mut self) -> Read {
        if let Some(text) = self.chunks.pop_front() {
            self.bytes -= text.len();
            Read::Data(text)
        } else if self.writer_closed {
            Read::Eof
        } else {
            Read::Pending
        }
    }
    fn close_reader(&mut self) {
        self.reader_closed = true;
        self.chunks.clear();
        self.bytes = 0;
    }
}
enum Input {
    Text(String, usize),
    Pipe(usize),
    Terminal,
    Eof,
}
impl Input {
    fn read(&mut self, pipes: &mut [Pipe]) -> Read {
        match self {
            Self::Text(text, offset) => {
                if *offset == text.len() {
                    return Read::Eof;
                }
                let end = chunk_end(text, *offset);
                let value = text[*offset..end].into();
                *offset = end;
                Read::Data(value)
            }
            Self::Pipe(i) => pipes[*i].read(),
            Self::Terminal => control::read(),
            Self::Eof => Read::Eof,
        }
    }
    fn close(&self, pipes: &mut [Pipe]) {
        if let Self::Pipe(i) = self {
            pipes[*i].close_reader();
        }
    }
}
enum Engine {
    Yes(String),
    Cat,
    Head(usize),
    Legacy { text: String, read: bool },
    Finished,
}
struct Process {
    pid: u32,
    session: crate::world::TerminalSession,
    input: Input,
    engine: Engine,
    pending: VecDeque<(u8, String)>,
    destinations: [Destination; 3],
    files: Vec<File>,
    status: i32,
    done: bool,
    archive_job: Option<u32>,
}
fn chunk_end(text: &str, start: usize) -> usize {
    let mut end = (start + CHUNK).min(text.len());
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    end
}
fn chunks(pending: &mut VecDeque<(u8, String)>, fd: u8, text: String) {
    let mut offset = 0;
    while offset < text.len() {
        let end = chunk_end(&text, offset);
        pending.push_back((fd, text[offset..end].into()));
        offset = end;
    }
}
pub(super) fn interactive_or_producer(stage: &Stage) -> bool {
    let Some(name) = stage.arguments.first() else {
        return false;
    };
    let args = &stage.arguments[1..];
    if name == "yes" {
        return true;
    }
    match name.as_str() {
        "cat" => crate::terminal_io::options("cat", args, "nbEsTu", "", &[])
            .is_ok_and(|o| !o.help && (o.files.is_empty() || o.files == ["-"])),
        "head" => matches!(engine(stage, true), Engine::Head(_)),
        _ => false,
    }
}
fn engine(stage: &Stage, available: bool) -> Engine {
    let name = stage.arguments.first().map(String::as_str).unwrap_or("");
    let args = stage.arguments.get(1..).unwrap_or_default();
    if available {
        if name == "yes" && !args.iter().any(|a| a.starts_with('-') && a != "--") {
            let args = if args.first().is_some_and(|s| s == "--") {
                &args[1..]
            } else {
                args
            };
            return Engine::Yes(format!(
                "{}\n",
                if args.is_empty() {
                    "y".into()
                } else {
                    args.join(" ")
                }
            ));
        }
        if name == "cat" && (args.is_empty() || args == ["-"]) {
            return Engine::Cat;
        }
        if name == "head" {
            let count = if args.is_empty() {
                Some(10)
            } else if args.len() == 2 && args[0] == "-n" {
                args[1].parse().ok()
            } else if args.len() == 1 {
                args[0]
                    .strip_prefix("-n")
                    .or_else(|| args[0].strip_prefix("--lines="))
                    .and_then(|s| s.parse().ok())
            } else {
                None
            };
            if let Some(count) = count {
                return Engine::Head(count);
            }
        }
    }
    // Only commands with an stdin contract need to wait for upstream EOF.
    let input_command = if name == "sudo" {
        args.first().map(String::as_str).unwrap_or("")
    } else {
        name
    };
    let read = [
        "cat",
        "head",
        "tail",
        "grep",
        "sort",
        "uniq",
        "wc",
        "cut",
        "tr",
        "tee",
        "base64",
        "sha256sum",
        "strings",
        "bash",
        "sh",
        "source",
        ".",
    ]
    .contains(&input_command);
    Engine::Legacy {
        text: String::new(),
        read,
    }
}
fn prepare(
    world: &mut WorldState,
    stage: &Stage,
    index: usize,
    count: usize,
    original: &crate::world::TerminalSession,
    pipes: &mut [Pipe],
) -> GameResult<Process> {
    world.terminal = original.clone();
    let actor = original.user.clone();
    let pid = crate::shell::next_pid();
    let mut input = if index > 0 {
        Input::Pipe(index - 1)
    } else if let Some(text) = &original.stdin {
        Input::Text(text.clone(), 0)
    } else if original.io.stdin_tty {
        Input::Terminal
    } else {
        Input::Eof
    };
    let mut input_tty = index == 0 && original.io.stdin_tty;
    let mut destinations = [
        Destination::Null,
        if index + 1 < count {
            Destination::Pipe
        } else {
            Destination::Stdout
        },
        Destination::Stderr,
    ];
    let mut files = Vec::new();
    let mut input_path = None;
    let mut error = None;
    for redirect in &stage.redirects {
        let result = (|| -> GameResult<()> {
            match redirect {
                Redirect::Duplicate(to, from) => {
                    destinations[*to as usize] = destinations[*from as usize]
                }
                Redirect::Input(target) => {
                    let path = normalize(target, &original.cwd)?;
                    if path != "/dev/null" {
                        world.fs()?.readable(&path, &actor)?;
                    }
                    input_path = Some(path);
                    input_tty = false;
                }
                Redirect::Output(fd, target, append) => {
                    let path = normalize(target, &original.cwd)?;
                    if path == "/dev/null" {
                        destinations[*fd as usize] = Destination::Null;
                    } else {
                        world.fs_mut()?.prepare_output(&path, &actor, *append)?;
                        destinations[*fd as usize] = Destination::File(files.len());
                        files.push(File {
                            path,
                            host: original.host.clone(),
                            append: *append,
                            offset: 0,
                        });
                    }
                }
            }
            Ok(())
        })();
        if let Err(e) = result {
            error = Some(failure(format!("bash: {e}")));
            break;
        }
    }
    if let Some(path) = input_path {
        input.close(pipes);
        match if path == "/dev/null" {
            Ok(String::new())
        } else {
            world.fs()?.read(&path, &actor)
        } {
            Ok(text) => input = Input::Text(text, 0),
            Err(e) => error = Some(failure(format!("bash: {e}"))),
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
    for (key, value) in &stage.assignments {
        world.terminal.env.insert(key.clone(), value.clone());
        world.terminal.exported.insert(key.clone());
        world.terminal.shell.unset.remove(key);
    }
    let name = stage.arguments.first().map(String::as_str).unwrap_or("");
    let available = original.host.is_some()
        || crate::packages::executables::resolve(world, name, &actor).is_some_and(|resolved| {
            crate::shell::path_file(world, name, &actor).as_ref() == Some(&resolved)
        });
    let mut engine = engine(stage, available);
    let mut pending = VecDeque::new();
    let mut status = 0;
    if let Some(error) = error {
        chunks(&mut pending, 2, error.stderr);
        status = error.status;
        engine = Engine::Finished;
        input.close(pipes);
    }
    if matches!(engine, Engine::Head(0) | Engine::Legacy { read: false, .. }) {
        input.close(pipes);
    }
    world.processes.push(crate::world::VirtualProcess {
        pid,
        name: name.into(),
        user: actor,
        running: true,
    });
    Ok(Process {
        pid,
        session: world.terminal.clone(),
        input,
        engine,
        pending,
        destinations,
        files,
        status,
        done: false,
        archive_job: None,
    })
}
fn step(
    world: &mut WorldState,
    process: &mut Process,
    stage: &Stage,
    pipes: &mut [Pipe],
) -> GameResult<bool> {
    match &mut process.engine {
        Engine::Finished => return Ok(false),
        Engine::Yes(line) => {
            if line.len() > CHUNK {
                chunks(&mut process.pending, 1, line.clone());
            } else {
                chunks(
                    &mut process.pending,
                    1,
                    line.repeat((CHUNK / line.len()).max(1)),
                );
            }
        }
        Engine::Head(0) => {
            process.input.close(pipes);
            process.engine = Engine::Finished;
        }
        Engine::Cat | Engine::Head(_) => match process.input.read(pipes) {
            Read::Pending => return Ok(false),
            Read::Eof => process.engine = Engine::Finished,
            Read::Data(mut text) => {
                if let Engine::Head(remaining) = &mut process.engine {
                    let mut end = text.len();
                    for (i, byte) in text.bytes().enumerate() {
                        if byte == b'\n' {
                            *remaining -= 1;
                            if *remaining == 0 {
                                end = i + 1;
                                break;
                            }
                        }
                    }
                    text.truncate(end);
                    if *remaining == 0 {
                        process.input.close(pipes);
                        process.engine = Engine::Finished;
                    }
                }
                chunks(&mut process.pending, 1, text);
            }
        },
        Engine::Legacy { text, read } => {
            if *read {
                match process.input.read(pipes) {
                    Read::Pending => return Ok(false),
                    Read::Data(chunk) => {
                        if text.len() + chunk.len() > ADAPTER_LIMIT {
                            process.status = 1;
                            chunks(
                                &mut process.pending,
                                2,
                                "shell: legacy stdin adapter limit: 4 MiB\n".into(),
                            );
                            process.engine = Engine::Finished;
                            process.input.close(pipes);
                        } else {
                            text.push_str(&chunk);
                        }
                        return Ok(true);
                    }
                    Read::Eof => {}
                }
            }
            world.terminal = process.session.clone();
            world.terminal.stdin = Some(std::mem::take(text));
            let before = world.clone();
            let actor = world.terminal.user.clone();
            let mut result = if stage.arguments.is_empty() {
                Output {
                    status: stage.assignment_status.unwrap_or(0),
                    ..Output::default()
                }
            } else {
                match crate::archive::jobs::with_deferral(false, || {
                    control::capture(|| crate::terminal::dispatch(world, &stage.arguments, &actor))
                }) {
                    Ok(output) => output,
                    Err(error) => {
                        *world = before.clone();
                        failure(error)
                    }
                }
            };
            if world.terminal.nano.is_some()
                || world.terminal.archive_pending.is_some()
                || world.terminal.package_pending.is_some()
            {
                *world = before;
                result = failure(
                    "bash: interactive input requires a foreground command without redirection",
                );
            }
            if let Some(bytes) = result.binary.take() {
                match process.destinations[1] {
                    Destination::File(i) if !process.files[i].append => {
                        let size = bytes.len() as u64;
                        if let Err(e) = crate::archive::write_bytes(world, &process.files[i].path, bytes, &actor, size) { result = failure(e); }
                    }
                    Destination::Null => {}, _ => result = failure("bash: binary stdout requires > VIRTUAL_FILE; binary pipes/append are unsupported"),
                }
            }
            if result.stdout.len() + result.stderr.len() > ADAPTER_LIMIT {
                result = failure("shell: legacy output adapter limit: 4 MiB");
            }
            for (fd, text) in [(1, &result.stdout), (2, &result.stderr)] {
                let emitted: usize = result
                    .ordered
                    .iter()
                    .filter(|(f, _)| *f == fd)
                    .map(|(_, s)| s.len())
                    .sum();
                if emitted < text.len() {
                    result.ordered.push((fd, text[emitted..].into()));
                }
            }
            for (fd, text) in result.ordered {
                chunks(&mut process.pending, fd, text);
            }
            process.session = world.terminal.clone();
            process.status = result.status;
            process.archive_job = result.archive_job;
            process.engine = Engine::Finished;
        }
    }
    Ok(true)
}

pub(super) fn run(world: &mut WorldState, stages: &[Stage]) -> GameResult<Output> {
    let original = world.terminal.clone();
    let mut pipes: Vec<Pipe> = (1..stages.len()).map(|_| Pipe::default()).collect();
    let mut processes = Vec::new();
    let mut output = Output::default();
    let result = (|| -> GameResult<()> {
        for (index, stage) in stages.iter().enumerate() {
            processes.push(prepare(
                world,
                stage,
                index,
                stages.len(),
                &original,
                &mut pipes,
            )?);
        }
        while processes.iter().any(|p| !p.done) {
            if control::cancelled() {
                output.status = 130;
                break;
            }
            let mut progress = false;
            for index in (0..processes.len()).rev() {
                let process = &mut processes[index];
                if process.done {
                    continue;
                }
                world.terminal = process.session.clone();
                if process.pending.is_empty() {
                    progress |= step(world, process, &stages[index], &mut pipes)?;
                }
                if let Some((fd, text)) = process.pending.front() {
                    let destination = process.destinations[*fd as usize];
                    let sent = if matches!(destination, Destination::Pipe) {
                        match pipes[index].push(text.clone()) {
                            Ok(sent) => sent,
                            Err(()) => {
                                process.status = 141;
                                process.pending.clear();
                                process.engine = Engine::Finished;
                                true
                            }
                        }
                    } else {
                        match emit(
                            world,
                            destination,
                            text,
                            &mut process.files,
                            &original.user,
                            &mut output,
                            &mut String::new(),
                        ) {
                            Ok(()) => true,
                            Err(error) => {
                                process.pending.clear();
                                process.engine = Engine::Finished;
                                process.status = 1;
                                let diagnostic = format!("bash: write error: {error}\n");
                                if output.stdout.len() + output.stderr.len() < ADAPTER_LIMIT {
                                    output.stderr.push_str(&diagnostic);
                                    output.ordered.push((2, diagnostic));
                                }
                                true
                            }
                        }
                    };
                    if sent {
                        process.pending.pop_front();
                        progress = true;
                    }
                }
                if process.pending.is_empty() && matches!(process.engine, Engine::Finished) {
                    process.done = true;
                    if process.status == 0
                        && stages[index].arguments.first().is_some_and(|name| {
                            !["help", "clear", "echo", "lab"].contains(&name.as_str())
                        })
                    {
                        world.techniques.insert(stages[index].arguments[0].clone());
                    }
                    process.input.close(&mut pipes);
                    if index < pipes.len() {
                        pipes[index].writer_closed = true;
                    }
                    if index + 1 == stages.len() {
                        output.status = process.status;
                        output.archive_job = process.archive_job;
                    }
                    if let Some(p) = world.processes.iter_mut().find(|p| p.pid == process.pid) {
                        p.running = false;
                    }
                    progress = true;
                }
            }
            if !progress {
                control::wait();
            }
        }
        Ok(())
    })();
    world
        .processes
        .retain(|p| !processes.iter().any(|running| running.pid == p.pid));
    world.terminal = original;
    #[cfg(test)]
    LAST_PEAK.with(|peak| peak.set(pipes.iter().map(|pipe| pipe.peak).max().unwrap_or(0)));
    result?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_channel_distinguishes_backpressure_eof_and_broken_pipe() {
        let mut pipe = Pipe::default();
        assert!(matches!(pipe.read(), Read::Pending));
        for _ in 0..CAPACITY / CHUNK {
            assert_eq!(pipe.push("x".repeat(CHUNK)), Ok(true));
        }
        assert_eq!(pipe.push("x".into()), Ok(false));
        assert_eq!(pipe.peak, CAPACITY);
        pipe.writer_closed = true;
        for _ in 0..CAPACITY / CHUNK {
            assert!(matches!(pipe.read(), Read::Data(_)));
        }
        assert!(matches!(pipe.read(), Read::Eof));
        pipe.close_reader();
        assert_eq!(pipe.push("x".into()), Err(()));
    }
}
