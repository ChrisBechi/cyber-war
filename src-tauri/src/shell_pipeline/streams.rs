//! A deterministic cooperative scheduler with bounded byte channels. There are
//! no host threads/processes per stage, and an empty buffer is distinct from EOF.
use super::*;
use crate::shell::control;
use crate::shell::signals::{ProcessSignalState, Termination, VirtualSignal};
use std::sync::Arc;
enum Read {
    Data(Vec<u8>),
    Eof,
    Pending,
    Interrupted,
}
use std::collections::VecDeque;

const CHUNK: usize = 4096;
const CAPACITY: usize = 64 * 1024;
const ADAPTER_LIMIT: usize = 4 * 1024 * 1024;
#[cfg(test)]
thread_local! { static READ_PATTERN: std::cell::RefCell<VecDeque<usize>> = const { std::cell::RefCell::new(VecDeque::new()) }; }
#[cfg(test)]
pub(crate) fn with_read_pattern<T>(pattern: &[usize], work: impl FnOnce() -> T) -> T {
    assert!(pattern.iter().all(|n| *n > 0));
    struct Restore(VecDeque<usize>);
    impl Drop for Restore {
        fn drop(&mut self) {
            READ_PATTERN.with(|p| *p.borrow_mut() = std::mem::take(&mut self.0));
        }
    }
    let _restore = Restore(READ_PATTERN.with(|p| p.replace(pattern.iter().copied().collect())));
    work()
}
#[cfg(test)]
thread_local! { static LAST_PEAK: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }
#[cfg(test)]
thread_local! { static CLOSED_CONSUMER: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }
#[cfg(test)]
pub(crate) fn with_closed_consumer<T>(work: impl FnOnce() -> T) -> T {
    struct Restore(bool);
    impl Drop for Restore {
        fn drop(&mut self) {
            CLOSED_CONSUMER.with(|c| c.set(self.0));
        }
    }
    let _restore = Restore(CLOSED_CONSUMER.with(|c| c.replace(true)));
    work()
}
#[cfg(test)]
pub(crate) fn last_peak() -> usize {
    LAST_PEAK.with(std::cell::Cell::get)
}
#[derive(Default)]
struct Pipe {
    chunks: VecDeque<Vec<u8>>,
    bytes: usize,
    writer_closed: bool,
    reader_closed: bool,
    peak: usize,
}
impl Pipe {
    fn push(&mut self, text: Vec<u8>) -> Result<bool, ()> {
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
    #[cfg(test)]
    fn read(&mut self) -> Read {
        self.read_limit(CHUNK)
    }
    fn read_limit(&mut self, limit: usize) -> Read {
        if let Some(mut text) = self.chunks.pop_front() {
            if text.len() > limit {
                self.chunks.push_front(text.split_off(limit));
            }
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
    Text(Vec<u8>, usize),
    File(u64),
    Pipe(usize),
    Terminal,
    Eof,
}
impl Input {
    fn read(&mut self, world: &mut WorldState, pipes: &mut [Pipe]) -> GameResult<Read> {
        self.read_limit(world, pipes, CHUNK)
    }
    fn read_limit(
        &mut self,
        world: &mut WorldState,
        pipes: &mut [Pipe],
        limit: usize,
    ) -> GameResult<Read> {
        #[cfg(test)]
        let limit = READ_PATTERN.with(|p| {
            let mut p = p.borrow_mut();
            let Some(n) = p.pop_front() else { return limit };
            p.push_back(n);
            limit.min(n)
        });
        Ok(match self {
            Self::Text(text, offset) => {
                if *offset == text.len() {
                    return Ok(Read::Eof);
                }
                let end = (*offset + limit).min(text.len());
                let value = text[*offset..end].into();
                *offset = end;
                Read::Data(value)
            }
            Self::Pipe(i) => pipes[*i].read_limit(limit),
            Self::File(handle) => {
                let blobs = world.blobs.clone();
                let bytes = world.fs_mut()?.read_handle_bytes(*handle, limit, &blobs)?;
                if bytes.is_empty() {
                    Read::Eof
                } else {
                    Read::Data(bytes)
                }
            }
            Self::Terminal => match control::read_limit(limit) {
                control::Read::Data(text) => Read::Data(text),
                control::Read::Eof => Read::Eof,
                control::Read::Pending => Read::Pending,
                control::Read::Interrupted => Read::Interrupted,
            },
            Self::Eof => Read::Eof,
        })
    }
    fn close(&self, pipes: &mut [Pipe]) {
        if let Self::Pipe(i) = self {
            pipes[*i].close_reader();
        }
    }
}
enum Engine {
    Yes(Vec<u8>),
    Cat(Box<crate::coreutils::cat::Cat>),
    Base64(Box<crate::coreutils::base64::Base64>),
    Tee(Box<crate::coreutils::tee::Tee>),
    Head(Box<crate::coreutils::head::Head>),
    Tail(Box<crate::coreutils::tail::Tail>),
    Legacy { text: Vec<u8>, read: bool },
    Finished,
}
struct Process {
    pid: u32,
    session: crate::world::TerminalSession,
    input: Input,
    engine: Engine,
    pending: VecDeque<(u8, Vec<u8>)>,
    destinations: [Destination; 3],
    files: Vec<File>,
    status: i32,
    done: bool,
    archive_job: Option<u32>,
    signals: Arc<ProcessSignalState>,
    termination: Option<Termination>,
    stdout_buffer: Option<crate::shell::stdio::OutputBuffer>,
}

fn chunks(pending: &mut VecDeque<(u8, Vec<u8>)>, fd: u8, text: Vec<u8>) {
    for chunk in text.chunks(CHUNK) {
        pending.push_back((fd, chunk.to_vec()));
    }
}

pub(super) fn interactive_or_producer(stage: &Stage) -> bool {
    let Some(name) = stage.arguments.first() else {
        return false;
    };
    let name = name.rsplit('/').next().unwrap_or(name);
    if name == "yes" {
        return true;
    }
    matches!(name, "cat" | "head" | "tail" | "base64" | "tee")
}
fn engine(stage: &Stage, available: bool) -> Engine {
    let name = stage
        .arguments
        .first()
        .map(String::as_str)
        .unwrap_or("")
        .rsplit('/')
        .next()
        .unwrap_or("");
    let args = stage.arguments.get(1..).unwrap_or_default();
    if available && name == "yes" && !args.iter().any(|a| a.starts_with('-') && a != "--") {
        let args = if args.first().is_some_and(|s| s == "--") {
            &args[1..]
        } else {
            args
        };
        return Engine::Yes(
            format!(
                "{}\n",
                if args.is_empty() {
                    "y".into()
                } else {
                    args.join(" ")
                }
            )
            .into_bytes(),
        );
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
        text: Vec::new(),
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
    } else if original.stdin_bytes.is_some() || original.stdin.is_some() {
        Input::Text(crate::coreutils::io::stdin(world), 0)
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
    let mut error = None;
    for redirect in &stage.redirects {
        let result = (|| -> GameResult<()> {
            match redirect {
                Redirect::Duplicate(to, from) => {
                    destinations[*to as usize] = destinations[*from as usize]
                }
                Redirect::Input(target) => {
                    let path = normalize(target, &original.cwd)?;
                    let handle = world.fs_mut()?.open(
                        &path,
                        crate::vfs::OpenFlags {
                            read: true,
                            ..Default::default()
                        },
                        0,
                        &actor,
                    )?;
                    input.close(pipes);
                    input = Input::File(handle);
                    files.push(File {
                        host: original.host.clone(),
                        handle,
                    });
                    input_tty = false;
                }
                Redirect::Output(fd, target, append) => {
                    let path = normalize(target, &original.cwd)?;
                    destinations[*fd as usize] = Destination::File(files.len());
                    files.push(open_output(world, path, &actor, *append)?);
                }
            }
            Ok(())
        })();
        if let Err(e) = result {
            error = Some(failure(format!("bash: {e}")));
            break;
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
            name.contains('/')
                || crate::shell::path_file(world, name, &actor).as_ref() == Some(&resolved)
        });
    let mut engine = engine(stage, available);
    let mut pending = VecDeque::new();
    let mut status = 0;
    if available
        && matches!(
            name.rsplit('/').next(),
            Some("cat" | "head" | "tail" | "base64" | "tee")
        )
        && error.is_none()
    {
        let posix = world.terminal.exported.contains("POSIXLY_CORRECT")
            && world.terminal.env.contains_key("POSIXLY_CORRECT");
        let parsed = if name.rsplit('/').next() == Some("tee") {
            crate::coreutils::tee::Tee::new(name, &stage.arguments[1..], posix)
                .map(|t| Engine::Tee(Box::new(t)))
        } else if name.rsplit('/').next() == Some("base64") {
            crate::coreutils::base64::Base64::new(name, &stage.arguments[1..], posix)
                .map(|b| Engine::Base64(Box::new(b)))
        } else if name.rsplit('/').next() == Some("head") {
            crate::coreutils::head::Head::new(name, &stage.arguments[1..], posix)
                .map(|h| Engine::Head(Box::new(h)))
        } else if name.rsplit('/').next() == Some("tail") {
            crate::coreutils::tail::Tail::new(name, &stage.arguments[1..], posix)
                .map(|t| Engine::Tail(Box::new(t)))
        } else {
            crate::coreutils::cat::Cat::new(name, &stage.arguments[1..], posix)
                .map(|c| Engine::Cat(Box::new(c)))
        };
        match parsed {
            Ok(parsed) => engine = parsed,
            Err(output) => {
                status = output.status;
                for (fd, bytes) in output_chunks(*output) {
                    chunks(&mut pending, fd, bytes);
                }
                engine = Engine::Finished;
                input.close(pipes);
            }
        }
    }
    if let Some(error) = error {
        chunks(&mut pending, 2, error.stderr.into_bytes());
        status = error.status;
        engine = Engine::Finished;
        input.close(pipes);
    }
    if matches!(engine, Engine::Legacy { read: false, .. }) {
        input.close(pipes);
    }
    world.processes.push(crate::world::VirtualProcess {
        pid,
        name: name.into(),
        user: actor,
        running: true,
    });
    let stdout_buffer = (matches!(engine, Engine::Tail(_))
        && matches!(destinations[1], Destination::File(_)))
    .then(crate::shell::stdio::OutputBuffer::default);
    let signals = control::register_process(pid);
    if let Engine::Tee(tee) = &engine {
        if tee.ignore_interrupts {
            signals.ignore(VirtualSignal::Int);
        }
        if tee.mode != crate::coreutils::tee::ErrorMode::Signal {
            signals.ignore(VirtualSignal::Pipe);
        }
    }
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
        signals,
        termination: None,
        stdout_buffer,
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
        Engine::Tee(tee) => {
            let (errors, finished) = if !tee.opened {
                tee.open(world)
            } else if tee.data.is_some() {
                tee.write_files(world)
            } else if !tee.active() {
                (Vec::new(), true)
            } else {
                match process
                    .input
                    .read_limit(world, pipes, crate::coreutils::tee::READ_SIZE)
                {
                    Ok(Read::Data(data)) => {
                        if tee.stdout {
                            chunks(&mut process.pending, 1, data.clone());
                        }
                        tee.data = Some(data);
                        (Vec::new(), false)
                    }
                    Ok(Read::Eof) => (Vec::new(), true),
                    Ok(Read::Pending | Read::Interrupted) => return Ok(false),
                    Err(error) => (crate::coreutils::tee::diagnostic("read error", error), true),
                }
            };
            if !errors.is_empty() {
                process.status = 1;
                chunks(&mut process.pending, 2, errors);
            }
            if finished {
                tee.close(world)?;
                process.engine = Engine::Finished;
                process.input.close(pipes);
            }
        }
        Engine::Base64(base64) => {
            if !base64.opened {
                base64.opened = true;
                if base64.file != "-" {
                    process.input.close(pipes);
                    let result = normalize(&base64.file, &process.session.cwd).and_then(|path| {
                        let fs = world.fs()?;
                        let node = fs.stat(&path, &process.session.user)?;
                        if node.kind == "directory" && !fs.allowed(node, &process.session.user, 4) {
                            return Err(crate::vfs::domain("Permission denied"));
                        }
                        world.fs_mut()?.open(
                            &path,
                            crate::vfs::OpenFlags {
                                read: true,
                                ..Default::default()
                            },
                            0,
                            &process.session.user,
                        )
                    });
                    match result {
                        Ok(handle) => base64.handle = Some(handle),
                        Err(error)
                            if crate::terminal_io::error_reason(&error) == "Is a directory" =>
                        {
                            base64.directory = true
                        }
                        Err(error) => {
                            chunks(
                                &mut process.pending,
                                2,
                                crate::coreutils::base64::diagnostic(&base64.file, error, true),
                            );
                            process.status = 1;
                            process.engine = Engine::Finished;
                            return Ok(true);
                        }
                    }
                }
            }
            let read = if base64.directory {
                Err(crate::vfs::domain("Is a directory"))
            } else if let Some(handle) = base64.handle {
                let blobs = world.blobs.clone();
                world
                    .fs_mut()?
                    .read_handle_bytes(handle, CHUNK, &blobs)
                    .map(|data| {
                        if data.is_empty() {
                            Read::Eof
                        } else {
                            Read::Data(data)
                        }
                    })
            } else {
                process.input.read(world, pipes)
            };
            let ended = match read {
                Ok(Read::Pending | Read::Interrupted) => return Ok(false),
                Ok(Read::Data(data)) => {
                    chunks(&mut process.pending, 1, base64.push(&data));
                    base64.transform.invalid()
                }
                Ok(Read::Eof) => {
                    chunks(&mut process.pending, 1, base64.finish());
                    true
                }
                Err(error) => {
                    chunks(
                        &mut process.pending,
                        2,
                        crate::coreutils::base64::diagnostic(&base64.file, error, false),
                    );
                    process.status = 1;
                    true
                }
            };
            if ended {
                if base64.transform.invalid() {
                    chunks(&mut process.pending, 2, b"base64: invalid input\n".to_vec());
                    process.status = 1;
                }
                if let Some(handle) = base64.handle.take() {
                    world.fs_mut()?.close(handle)?;
                }
                process.input.close(pipes);
                process.engine = Engine::Finished;
            }
        }
        Engine::Cat(cat) => {
            if cat.current.is_none() {
                let Some(file) = cat.files.pop_front() else {
                    chunks(&mut process.pending, 1, cat.transform.finish());
                    process.engine = Engine::Finished;
                    return Ok(true);
                };
                if file != "-" {
                    let result = normalize(&file, &process.session.cwd).and_then(|path| {
                        world.fs_mut()?.open(
                            &path,
                            crate::vfs::OpenFlags {
                                read: true,
                                ..Default::default()
                            },
                            0,
                            &process.session.user,
                        )
                    });
                    match result {
                        Ok(handle) => cat.handle = Some(handle),
                        Err(error) => {
                            process.status = 1;
                            chunks(
                                &mut process.pending,
                                2,
                                crate::coreutils::cat::diagnostic(&file, error),
                            );
                            return Ok(true);
                        }
                    }
                }
                let input_handle = cat.handle.or(match process.input {
                    Input::File(handle) => Some(handle),
                    _ => None,
                });
                if let (Some(input), Destination::File(index)) =
                    (input_handle, process.destinations[1])
                {
                    let (ino, offset, size, regular) = world.fs()?.handle_identity(input)?;
                    let (output_ino, _, _, _) =
                        world.fs()?.handle_identity(process.files[index].handle)?;
                    if regular && ino == output_ino && (offset as u64) < size {
                        if let Some(h) = cat.handle.take() {
                            world.fs_mut()?.close(h)?;
                        }
                        process.status = 1;
                        chunks(
                            &mut process.pending,
                            2,
                            crate::coreutils::cat::diagnostic(&file, "input file is output file"),
                        );
                        return Ok(true);
                    }
                }
                cat.current = Some(file);
            }
            let read = if let Some(handle) = cat.handle {
                let blobs = world.blobs.clone();
                world
                    .fs_mut()?
                    .read_handle_bytes(handle, CHUNK, &blobs)
                    .map(|bytes| {
                        if bytes.is_empty() {
                            Read::Eof
                        } else {
                            Read::Data(bytes)
                        }
                    })
            } else {
                process.input.read(world, pipes)
            };
            match read {
                Ok(Read::Pending | Read::Interrupted) => return Ok(false),
                Ok(Read::Data(bytes)) => {
                    chunks(&mut process.pending, 1, cat.transform.transform(&bytes))
                }
                ending => {
                    if let Err(error) = ending {
                        process.status = 1;
                        chunks(
                            &mut process.pending,
                            2,
                            crate::coreutils::cat::diagnostic(
                                cat.current.as_deref().unwrap_or("-"),
                                error,
                            ),
                        );
                    }
                    if let Some(handle) = cat.handle.take() {
                        world.fs_mut()?.close(handle)?;
                    }
                    cat.current = None;
                }
            }
        }
        Engine::Head(head) => {
            if head.current.is_none() {
                let Some(file) = head.files.pop_front() else {
                    process.input.close(pipes);
                    process.engine = Engine::Finished;
                    return Ok(true);
                };
                head.directory = false;
                if file != "-" {
                    let result = normalize(&file, &process.session.cwd).and_then(|path| {
                        let fs = world.fs()?;
                        let node = fs.stat(&path, &process.session.user)?;
                        if node.kind == "directory" && !fs.allowed(node, &process.session.user, 4) {
                            return Err(crate::vfs::domain("Permission denied"));
                        }
                        world.fs_mut()?.open(
                            &path,
                            crate::vfs::OpenFlags {
                                read: true,
                                ..Default::default()
                            },
                            0,
                            &process.session.user,
                        )
                    });
                    match result {
                        Ok(handle) => head.handle = Some(handle),
                        Err(error) => {
                            // POSIX open of a directory succeeds; the error occurs on read.
                            if crate::terminal_io::error_reason(&error) == "Is a directory" {
                                head.directory = true;
                            } else {
                                process.status = 1;
                                chunks(
                                    &mut process.pending,
                                    2,
                                    crate::coreutils::head::diagnostic(&file, error, true),
                                );
                                return Ok(true);
                            }
                        }
                    }
                }
                chunks(&mut process.pending, 1, head.start(file));
                if let Some(handle) = head.handle.or(match process.input {
                    Input::File(h) if head.current.as_deref() == Some("-") => Some(h),
                    _ => None,
                }) {
                    let (_, offset, size, regular) = world.fs()?.handle_identity(handle)?;
                    if regular {
                        head.regular_source(size.saturating_sub(offset as u64));
                    }
                }
            }
            let read = if head.transform.done() || head.source_remaining == Some(0) {
                Ok(Read::Eof)
            } else if head.directory {
                Err(crate::vfs::domain("Is a directory"))
            } else if let Some(handle) = head.handle {
                let blobs = world.blobs.clone();
                world
                    .fs_mut()?
                    .read_handle_bytes(handle, head.read_limit(), &blobs)
                    .map(|data| {
                        if data.is_empty() {
                            Read::Eof
                        } else {
                            Read::Data(data)
                        }
                    })
            } else {
                process.input.read_limit(world, pipes, head.read_limit())
            };
            match read {
                Ok(Read::Pending | Read::Interrupted) => return Ok(false),
                Ok(Read::Data(bytes)) => {
                    if let Some(left) = &mut head.source_remaining {
                        *left = left.saturating_sub(bytes.len() as u64);
                    }
                    let (output, consumed) = head.transform.push(&bytes);
                    chunks(&mut process.pending, 1, output);
                    if consumed < bytes.len() {
                        if let Some(handle) = head.handle.or(match process.input {
                            Input::File(h) if head.current.as_deref() == Some("-") => Some(h),
                            _ => None,
                        }) {
                            let (_, offset, _, regular) = world.fs()?.handle_identity(handle)?;
                            if regular {
                                world
                                    .fs_mut()?
                                    .seek(handle, offset - (bytes.len() - consumed))?;
                            }
                        }
                    }
                }
                ending => {
                    if let Err(error) = ending {
                        process.status = 1;
                        chunks(
                            &mut process.pending,
                            2,
                            crate::coreutils::head::diagnostic(
                                head.current.as_deref().unwrap_or("-"),
                                error,
                                false,
                            ),
                        );
                    } else {
                        chunks(&mut process.pending, 1, head.transform.finish());
                        if let Some(handle) = head.handle.or(match process.input {
                            Input::File(h) if head.current.as_deref() == Some("-") => Some(h),
                            _ => None,
                        }) {
                            let (_, offset, _, regular) = world.fs()?.handle_identity(handle)?;
                            if regular {
                                world.fs_mut()?.seek(
                                    handle,
                                    offset.saturating_sub(head.transform.retained()),
                                )?;
                            }
                        }
                    }
                    if let Some(handle) = head.handle.take() {
                        world.fs_mut()?.close(handle)?;
                    }
                    head.current = None;
                }
            }
        }
        Engine::Tail(tail) => {
            use crate::coreutils::tail::Step;
            let stdin_file = match process.input {
                Input::File(h) => Some(h),
                _ => None,
            };
            match tail.step(world, stdin_file, process.pid)? {
                Step::Output(fd, bytes) => {
                    let bytes = if fd == 1 {
                        process
                            .stdout_buffer
                            .as_mut()
                            .map_or_else(|| bytes.clone(), |buffer| buffer.push(&bytes))
                    } else {
                        bytes
                    };
                    chunks(&mut process.pending, fd, bytes);
                }
                Step::Progress => {}
                Step::Input => match process.input.read(world, pipes)? {
                    Read::Pending | Read::Interrupted => return Ok(false),
                    Read::Data(bytes) => tail.feed(Some(&bytes)),
                    Read::Eof => tail.feed(None),
                },
                Step::Wait => {
                    if let Some(buffer) = &mut process.stdout_buffer {
                        let bytes = buffer.flush();
                        if !bytes.is_empty() {
                            chunks(&mut process.pending, 1, bytes);
                            return Ok(true);
                        }
                    }
                    world
                        .scheduler
                        .waiting
                        .insert(process.pid, tail.wait_set(world));
                    return Ok(false);
                }
                Step::Done => {
                    if let Some(buffer) = &mut process.stdout_buffer {
                        chunks(&mut process.pending, 1, buffer.flush());
                    }
                    process.status = tail.status;
                    process.engine = Engine::Finished;
                    process.input.close(pipes);
                }
            }
            world.scheduler.waiting.remove(&process.pid);
        }
        Engine::Legacy { text, read } => {
            if *read {
                match process.input.read(world, pipes)? {
                    Read::Pending | Read::Interrupted => return Ok(false),
                    Read::Data(chunk) => {
                        if text.len() + chunk.len() > ADAPTER_LIMIT {
                            process.status = 1;
                            chunks(
                                &mut process.pending,
                                2,
                                b"shell: legacy stdin adapter limit: 4 MiB\n".to_vec(),
                            );
                            process.engine = Engine::Finished;
                            process.input.close(pipes);
                        } else {
                            text.extend_from_slice(&chunk);
                        }
                        return Ok(true);
                    }
                    Read::Eof => {}
                }
            }
            world.terminal = process.session.clone();
            let data = std::mem::take(text);
            world.terminal.stdin = String::from_utf8(data.clone()).ok();
            world.terminal.stdin_bytes = Some(data);
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
            let status = result.status;
            let job = result.archive_job;
            for (fd, data) in output_chunks(result) {
                chunks(&mut process.pending, fd, data);
            }
            process.session = world.terminal.clone();
            process.status = status;
            process.archive_job = job;
            process.engine = Engine::Finished;
        }
    }
    Ok(true)
}

fn close_operand(world: &mut WorldState, engine: &mut Engine) -> GameResult<()> {
    if let Engine::Tee(tee) = engine {
        tee.close(world)?;
    }
    if let Engine::Tail(tail) = engine {
        tail.close(world)?;
    }
    let handle = match engine {
        Engine::Cat(c) => c.handle.take(),
        Engine::Head(h) => h.handle.take(),
        Engine::Base64(b) => b.handle.take(),
        _ => None,
    };
    if let Some(handle) = handle {
        world.fs_mut()?.close(handle)?;
    }
    Ok(())
}

fn terminate(
    world: &mut WorldState,
    process: &mut Process,
    signal: VirtualSignal,
) -> GameResult<()> {
    close_operand(world, &mut process.engine)?;
    let termination = Termination::Signal { signal };
    process.termination = Some(termination);
    process.status = termination.status();
    process.pending.clear();
    if let Some(buffer) = &mut process.stdout_buffer {
        buffer.clear();
    }
    process.engine = Engine::Finished;
    Ok(())
}

// The shared delivery path reports stdout failures to fan-out commands before
// applying its default SIGPIPE/abort behavior. Other destinations can survive.
fn tee_output_error(
    world: &mut WorldState,
    process: &mut Process,
    pipe: bool,
    reason: impl std::fmt::Display,
) -> GameResult<bool> {
    let Engine::Tee(tee) = &mut process.engine else {
        return Ok(false);
    };
    if pipe && tee.mode == crate::coreutils::tee::ErrorMode::Signal {
        return Ok(false);
    }
    tee.stdout = false;
    process.pending.retain(|(fd, _)| *fd != 1);
    if tee.mode.diagnose(pipe) {
        process.status = 1;
        chunks(
            &mut process.pending,
            2,
            crate::coreutils::tee::diagnostic("standard output", reason),
        );
    }
    if tee.mode.fatal(pipe) {
        tee.close(world)?;
        process.engine = Engine::Finished;
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
        #[cfg(test)]
        if CLOSED_CONSUMER.with(std::cell::Cell::get) {
            let mut pipe = Pipe::default();
            pipe.close_reader();
            pipes.push(pipe);
            processes.last_mut().unwrap().destinations[1] = Destination::Pipe;
        }
        while processes.iter().any(|p| !p.done) {
            let wake_epoch = control::event_epoch();
            let mut progress = false;
            for index in (0..processes.len()).rev() {
                let process = &mut processes[index];
                if process.done {
                    continue;
                }
                world.terminal = process.session.clone();
                if matches!(&process.engine, Engine::Tee(t) if t.opened && t.stdout && matches!(t.mode, crate::coreutils::tee::ErrorMode::WarnNoPipe | crate::coreutils::tee::ErrorMode::ExitNoPipe))
                    && matches!(process.input, Input::Terminal | Input::Pipe(_))
                    && matches!(process.destinations[1], Destination::Pipe)
                    && pipes[index].reader_closed
                {
                    tee_output_error(world, process, true, "Broken pipe")?;
                    progress = true;
                }
                if matches!(&process.engine,Engine::Tail(t) if t.monitor_output())
                    && matches!(process.destinations[1], Destination::Pipe)
                    && pipes[index].reader_closed
                {
                    terminate(world, process, VirtualSignal::Pipe)?;
                    progress = true;
                }
                if let Some(signal) = world
                    .scheduler
                    .signals
                    .remove(&process.pid)
                    .filter(|signal| !process.signals.ignored(*signal))
                    .or_else(|| process.signals.take())
                    .or_else(control::termination_signal)
                {
                    terminate(world, process, signal)?;
                    progress = true;
                }
                if process.pending.is_empty() {
                    progress |= step(world, process, &stages[index], &mut pipes)?;
                }
                if let Some((fd, text)) = process.pending.front().cloned() {
                    let destination = process.destinations[fd as usize];
                    let mut remove_pending = true;
                    let sent = if matches!(destination, Destination::Pipe) {
                        match pipes[index].push(text.clone()) {
                            Ok(sent) => sent,
                            Err(()) => {
                                if fd == 1 && tee_output_error(world, process, true, "Broken pipe")?
                                {
                                    remove_pending = false;
                                } else {
                                    terminate(world, process, VirtualSignal::Pipe)?;
                                }
                                true
                            }
                        }
                    } else {
                        match control::with_process(&process.signals, || {
                            emit_bytes(
                                world,
                                destination,
                                &text,
                                &mut process.files,
                                &mut output,
                                &mut Vec::new(),
                            )
                        }) {
                            Ok(()) => true,
                            Err(error) => {
                                if fd == 1 && tee_output_error(world, process, false, &error)? {
                                    remove_pending = false;
                                } else {
                                    close_operand(world, &mut process.engine)?;
                                    process.pending.clear();
                                    process.engine = Engine::Finished;
                                    process.status = 1;
                                    let diagnostic = format!("bash: write error: {error}\n");
                                    if output.stdout.len() + output.stderr.len() < ADAPTER_LIMIT {
                                        output.stderr.push_str(&diagnostic);
                                        output
                                            .byte_ordered
                                            .push((2, diagnostic.as_bytes().to_vec()));
                                        output.ordered.push((2, diagnostic));
                                    }
                                }
                                true
                            }
                        }
                    };
                    if sent {
                        if remove_pending {
                            process.pending.pop_front();
                        }
                        progress = true;
                    }
                }
                // A signal may have interrupted delivery while waiting for UI capacity.
                if let Some(signal) = process.signals.take().or_else(control::termination_signal) {
                    terminate(world, process, signal)?;
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
                        output.termination =
                            Some(process.termination.unwrap_or(Termination::Exit {
                                code: process.status,
                            }));
                        output.archive_job = process.archive_job;
                    }
                    if let Some(p) = world.processes.iter_mut().find(|p| p.pid == process.pid) {
                        p.running = false;
                    }
                    progress = true;
                }
            }
            if !progress {
                let mut waiting = crate::shell::wait::WaitSet::default();
                for process in &processes {
                    if process.done {
                        continue;
                    }
                    waiting.processes.push((process.pid, true));
                    if let Some(condition) = world.scheduler.waiting.get(&process.pid) {
                        waiting.extend(condition);
                    } else {
                        waiting.input = true;
                        world.scheduler.waiting.insert(
                            process.pid,
                            crate::shell::wait::WaitSet {
                                input: true,
                                ..Default::default()
                            },
                        );
                    }
                }
                if !world.scheduler.advance_idle(&waiting) {
                    control::wait_world(world, waiting, wake_epoch);
                }
            }
        }
        Ok(())
    })();
    world
        .processes
        .retain(|p| !processes.iter().any(|running| running.pid == p.pid));
    for process in &mut processes {
        world.scheduler.finish(process.pid);
        control::unregister_process(process.pid);
        world.terminal.host = process.session.host.clone();
        close_operand(world, &mut process.engine)?;
        close_files(world, &process.files)?;
    }
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
    fn controlled_producer_observes_prefix_close_without_eof() {
        // Exercise the actual scheduler without relying on a certified producer
        // executable: prepare head with a controlled internal pipe as stdin.
        for argv in [vec!["head", "-n1"], vec!["head", "-c1"]] {
            let mut world = WorldState::new("kali", "lifeos").unwrap();
            let original = world.terminal.clone();
            let stage = Stage {
                arguments: argv.iter().map(|s| (*s).into()).collect(),
                ..Default::default()
            };
            let mut pipes = vec![Pipe::default()];
            let mut process = prepare(&mut world, &stage, 1, 2, &original, &mut pipes).unwrap();
            assert_eq!(pipes[0].push(b"first\nsecond\n".to_vec()), Ok(true));
            for _ in 0..5 {
                step(&mut world, &mut process, &stage, &mut pipes).unwrap();
                process.pending.clear();
                if matches!(process.engine, Engine::Finished) {
                    break;
                }
            }
            assert!(matches!(process.engine, Engine::Finished));
            assert!(!pipes[0].writer_closed, "producer never sent EOF");
            assert_eq!(pipes[0].push(b"late".to_vec()), Err(()));
            assert!(pipes[0].peak <= CAPACITY);
            close_operand(&mut world, &mut process.engine).unwrap();
            close_files(&mut world, &process.files).unwrap();
            control::unregister_process(process.pid);
            assert_eq!(world.vfs.open_handle_count(), 0);
        }
    }
    #[test]
    fn bounded_channel_distinguishes_backpressure_eof_and_broken_pipe() {
        let mut pipe = Pipe::default();
        assert!(matches!(pipe.read(), Read::Pending));
        for _ in 0..CAPACITY / CHUNK {
            assert_eq!(pipe.push(vec![b'x'; CHUNK]), Ok(true));
        }
        assert_eq!(pipe.push(b"x".to_vec()), Ok(false));
        assert_eq!(pipe.peak, CAPACITY);
        pipe.writer_closed = true;
        for _ in 0..CAPACITY / CHUNK {
            assert!(matches!(pipe.read(), Read::Data(_)));
        }
        assert!(matches!(pipe.read(), Read::Eof));
        pipe.close_reader();
        assert_eq!(pipe.push(b"x".to_vec()), Err(()));
    }
}
