//! Incremental suffix/from-start selection and event-driven virtual file follow.
use super::{foundation_options as options, head::Selection};
use crate::vfs::{OpenFlags, WatchId, WatchTarget};
use crate::{error::GameResult, terminal_io::Output, world::WorldState};
use std::collections::VecDeque;

// Named tail operands reject directories after checking read access. Shell
// redirections may still supply a directory descriptor and fail at read time.
fn open_named(world: &mut WorldState, path: &str, actor: &str) -> GameResult<u64> {
    let fs = world.fs_mut()?;
    let handle = fs.open(
        path,
        OpenFlags {
            read: true,
            ..Default::default()
        },
        0,
        actor,
    )?;
    if fs.stat(path, actor)?.kind == "directory" {
        fs.close(handle)?;
        return Err(crate::error::GameError::Vfs(crate::vfs::Errno::IsDirectory));
    }
    Ok(handle)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Follow {
    Descriptor,
    Name,
}
#[derive(Default)]
pub(super) struct Options {
    pub from_start: bool,
    pub follow: Option<Follow>,
    pub retry: bool,
    pub pids: Vec<u32>,
    pub interval: Option<f64>,
    pub unchanged: Option<u64>,
}
pub(super) fn failure(message: String) -> Output {
    Output {
        stderr: format!("tail: {message}\n"),
        status: 1,
        ..Default::default()
    }
}
impl Options {
    pub fn parse(
        &mut self,
        ch: char,
        value: Option<&str>,
        invocation: &str,
    ) -> Result<(), Box<Output>> {
        match ch {
            'f' => {
                let value = value.unwrap_or("descriptor");
                self.follow = Some(if !value.is_empty() && "descriptor".starts_with(value) {
                    Follow::Descriptor
                } else if !value.is_empty() && "name".starts_with(value) {
                    Follow::Name
                } else {
                    return Err(Box::new(failure(format!("{} argument {} for '--follow'\nValid arguments are:\n  - 'descriptor'\n  - 'name'\nTry '{invocation} --help' for more information.",
                        if value.is_empty() {"ambiguous"} else {"invalid"}, options::quote(value)))));
                });
            }
            'F' => {
                self.follow = Some(Follow::Name);
                self.retry = true;
            }
            'r' => self.retry = true,
            's' => {
                let value = value.unwrap();
                self.interval = Some(
                    value
                        .trim_start_matches(|c: char| c.is_ascii_whitespace())
                        .parse::<f64>()
                        .ok()
                        .filter(|n| !n.is_nan() && *n >= 0.0)
                        .ok_or_else(|| {
                            Box::new(failure(format!(
                                "invalid number of seconds: {}",
                                options::quote(value)
                            )))
                        })?,
                );
            }
            'm' | 'p' => {
                let value = value.unwrap();
                let number = value
                    .trim_start_matches(|c: char| c.is_ascii_whitespace())
                    .strip_prefix('+')
                    .unwrap_or(value.trim_start_matches(|c: char| c.is_ascii_whitespace()));
                let reason = if ch == 'p' {
                    "PID"
                } else {
                    "maximum number of unchanged stats between opens"
                };
                if number.is_empty() || !number.bytes().all(|b| b.is_ascii_digit()) {
                    return Err(Box::new(failure(format!(
                        "invalid {reason}: {}",
                        options::quote(value)
                    ))));
                }
                let n = number.bytes().fold(0u64, |n, b| {
                    n.saturating_mul(10).saturating_add((b - b'0') as u64)
                });
                if ch == 'p' {
                    if n > i32::MAX as u64 {
                        return Err(Box::new(failure(format!(
                            "invalid PID: {}: Value too large for data type",
                            options::quote(value)
                        ))));
                    }
                    self.pids.push(n as u32);
                } else {
                    self.unchanged = Some(n);
                }
            }
            _ => unreachable!(),
        }
        Ok(())
    }
}

fn historical(args: &[String]) -> Vec<String> {
    let Some(first) = args.first().filter(|s| {
        s.starts_with(['-', '+']) && s.as_bytes().get(1).is_some_and(u8::is_ascii_digit)
    }) else {
        return args.to_vec();
    };
    if args.len() > 2 || args.get(1).is_some_and(|s| s.starts_with('-') && s != "-") {
        return args.to_vec();
    }
    let digits = 1 + first[1..].bytes().take_while(u8::is_ascii_digit).count();
    let suffix = &first[digits..];
    let (suffix, follow) = suffix
        .strip_suffix('f')
        .map_or((suffix, false), |s| (s, true));
    let (kind, multiplier) = match suffix {
        "" | "l" => ('n', ""),
        "c" => ('c', ""),
        "b" => ('c', "b"),
        _ => return args.to_vec(),
    };
    let mut out = vec![
        format!("-{kind}"),
        format!("{}{multiplier}", &first[..digits]),
    ];
    if follow {
        out.push("-f".into());
    }
    out.extend_from_slice(&args[1..]);
    out
}

pub(crate) struct Transform {
    selection: Selection,
    start: bool,
    delimiter: u8,
    skip: u64,
    bytes: VecDeque<u8>,
    records: u64,
}
impl Transform {
    pub fn new(selection: Selection, start: bool, delimiter: u8) -> Self {
        Self {
            selection,
            start,
            delimiter,
            skip: selection.count.saturating_sub(1),
            bytes: VecDeque::new(),
            records: 0,
        }
    }
    pub fn push(&mut self, input: &[u8]) -> Vec<u8> {
        if self.start {
            let mut begin = 0;
            while self.skip > 0 && begin < input.len() {
                if self.selection.bytes || input[begin] == self.delimiter {
                    self.skip -= 1;
                }
                begin += 1;
            }
            return input[begin..].to_vec();
        }
        if self.selection.count == 0 {
            return Vec::new();
        }
        for &byte in input {
            self.bytes.push_back(byte);
            if self.selection.bytes {
                if self.bytes.len() as u64 > self.selection.count {
                    self.bytes.pop_front();
                }
            } else {
                self.records += u64::from(byte == self.delimiter);
                // Retain N complete records and a possible incomplete record.
                while self.records > self.selection.count {
                    if self.bytes.pop_front() == Some(self.delimiter) {
                        self.records -= 1;
                    }
                }
            }
        }
        Vec::new()
    }
    pub fn finish(&mut self) -> Vec<u8> {
        if !self.start
            && !self.selection.bytes
            && self.records == self.selection.count
            && self.bytes.back().is_some_and(|b| *b != self.delimiter)
        {
            while let Some(b) = self.bytes.pop_front() {
                if b == self.delimiter {
                    break;
                }
            }
        }
        self.bytes.drain(..).collect()
    }
    pub fn retained(&self) -> usize {
        self.bytes.len()
    }
}

enum Position {
    New,
    Reverse {
        scan: usize,
        floor: usize,
        left: u64,
        last: bool,
    },
    Read,
    Done,
}
struct Source {
    name: String,
    path: String,
    handle: Option<u64>,
    owned: bool,
    regular: bool,
    end: Option<usize>,
    position: Position,
    transform: Transform,
    watches: Vec<(WatchId, u64)>,
    unavailable: bool,
    missing: bool,
    polling: bool,
    timer: Option<crate::shell::wait::TimerId>,
    recheck: bool,
    tty: bool,
}
pub(crate) enum Step {
    Output(u8, Vec<u8>),
    Progress,
    Input,
    Wait,
    Done,
}
pub(crate) struct Tail {
    sources: Vec<Source>,
    current: usize,
    initial: bool,
    selection: Selection,
    options: Options,
    delimiter: u8,
    headers: bool,
    last_header: Option<usize>,
    pending: VecDeque<(u8, Vec<u8>)>,
    pub status: i32,
    closed: bool,
}
impl Tail {
    pub fn monitor_output(&self) -> bool {
        !self.initial && self.options.follow.is_some()
    }
    pub fn new(invocation: &str, args: &[String], posix: bool) -> Result<Self, Box<Output>> {
        let parsed = options::parse("tail", invocation, &historical(args), posix)?;
        if let Some(kind) = parsed.special {
            return Err(Box::new(Output::success(
                super::foundation_messages::message("tail", invocation, kind),
            )));
        }
        let selection = parsed.selection.unwrap_or_default();
        let delimiter = if parsed.flags.contains(&'z') {
            0
        } else {
            b'\n'
        };
        let headers = parsed
            .flags
            .iter()
            .rev()
            .find(|c| matches!(c, 'q' | 'v'))
            .map_or(parsed.files.len() > 1, |c| *c == 'v');
        let files = if parsed.files.is_empty() {
            vec!["-".into()]
        } else {
            parsed.files
        };
        if parsed.tail.follow == Some(Follow::Name) && files.iter().any(|f| f == "-") {
            return Err(Box::new(failure("cannot follow '-' by name".into())));
        }
        let sources = files
            .into_iter()
            .map(|name| Source {
                name,
                path: String::new(),
                handle: None,
                owned: false,
                regular: false,
                end: None,
                position: Position::New,
                transform: Transform::new(selection, parsed.tail.from_start, delimiter),
                watches: Vec::new(),
                unavailable: false,
                missing: false,
                polling: false,
                timer: None,
                recheck: false,
                tty: false,
            })
            .collect();
        let mut pending = VecDeque::new();
        if parsed.tail.retry && parsed.tail.follow.is_none() {
            pending.push_back((
                2,
                b"tail: warning: --retry ignored; --retry is useful only when following\n".to_vec(),
            ));
        }
        if parsed.tail.retry && parsed.tail.follow == Some(Follow::Descriptor) {
            pending.push_back((
                2,
                b"tail: warning: --retry only effective for the initial open\n".to_vec(),
            ));
        }
        if !parsed.tail.pids.is_empty() && parsed.tail.follow.is_none() {
            pending.push_back((
                2,
                b"tail: warning: PID ignored; --pid=PID is useful only when following\n".to_vec(),
            ));
        }
        Ok(Self {
            sources,
            current: 0,
            initial: true,
            selection,
            options: parsed.tail,
            delimiter,
            headers,
            last_header: None,
            pending,
            status: 0,
            closed: false,
        })
    }
    fn header(&mut self, index: usize) {
        if self.headers && self.last_header != Some(index) {
            let name = &self.sources[index].name;
            let title = if name == "-" { "standard input" } else { name };
            self.pending.push_back((
                1,
                format!(
                    "{}==> {title} <==\n",
                    if self.last_header.is_some() { "\n" } else { "" }
                )
                .into_bytes(),
            ));
            self.last_header = Some(index);
        }
    }
    pub fn feed(&mut self, bytes: Option<&[u8]>) {
        if !self.initial {
            if let Some(bytes) = bytes {
                self.pending.push_back((1, bytes.to_vec()));
            }
            return;
        }
        let source = &mut self.sources[self.current];
        let output = if let Some(bytes) = bytes {
            source.transform.push(bytes)
        } else {
            source.position = Position::Done;
            source.transform.finish()
        };
        if source.transform.retained() > 4 * 1024 * 1024 {
            source.position = Position::Done;
            self.status = 1;
            self.pending
                .push_back((2, b"tail: virtual suffix buffer limit: 4 MiB\n".to_vec()));
        }
        if !output.is_empty() {
            self.pending.push_back((1, output));
        }
    }
    fn release_source(world: &mut WorldState, source: &mut Source) -> GameResult<()> {
        if let Some(timer) = source.timer.take() {
            world.scheduler.cancel_timer(timer);
        }
        for (watch, _) in source.watches.drain(..) {
            world.fs_mut()?.unsubscribe(watch);
        }
        if source.owned {
            if let Some(handle) = source.handle.take() {
                world.fs_mut()?.close(handle)?;
            }
        }
        Ok(())
    }
    pub fn close(&mut self, world: &mut WorldState) -> GameResult<()> {
        for source in &mut self.sources {
            Self::release_source(world, source)?;
        }
        self.closed = true;
        Ok(())
    }
    fn subscribe(world: &mut WorldState, source: &mut Source, by_name: bool) -> GameResult<()> {
        for (watch, _) in source.watches.drain(..) {
            world.fs_mut()?.unsubscribe(watch);
        }
        let mut targets = Vec::new();
        if by_name {
            targets.extend(
                world
                    .fs()?
                    .lookup_dependencies(&source.path, &world.terminal.user)
                    .into_iter()
                    .map(WatchTarget::Path),
            );
        }
        if let Some(handle) = source.handle {
            targets.push(WatchTarget::Inode(world.fs()?.handle_identity(handle)?.0));
        }
        for target in targets {
            let watch = world.fs_mut()?.subscribe(target)?;
            source
                .watches
                .push((watch, world.fs()?.watch_revision(watch).unwrap()));
        }
        Ok(())
    }
    pub fn wait_set(&self, world: &WorldState) -> crate::shell::wait::WaitSet {
        crate::shell::wait::WaitSet {
            vfs: self
                .sources
                .iter()
                .flat_map(|s| s.watches.iter())
                .map(|(watch, revision)| crate::shell::wait::VfsWait {
                    host: world.terminal.host.clone(),
                    watch: *watch,
                    revision: *revision,
                })
                .collect(),
            processes: self
                .options
                .pids
                .iter()
                .map(|pid| {
                    (
                        *pid,
                        world.processes.iter().any(|p| p.pid == *pid && p.running),
                    )
                })
                .collect(),
            timers: self.sources.iter().filter_map(|s| s.timer).collect(),
            ..Default::default()
        }
    }
    pub fn step(
        &mut self,
        world: &mut WorldState,
        stdin_file: Option<u64>,
        pid: u32,
    ) -> GameResult<Step> {
        if let Some((fd, bytes)) = self.pending.pop_front() {
            return Ok(Step::Output(fd, bytes));
        }
        if self.closed {
            return Ok(Step::Done);
        }
        if self.options.follow.is_none() && !self.options.from_start && self.selection.count == 0 {
            self.close(world)?;
            return Ok(Step::Done);
        }
        if self.initial {
            if self.current == self.sources.len() {
                self.initial = false;
                self.current = 0;
                return Ok(Step::Progress);
            }
            if matches!(self.sources[self.current].position, Position::Done) {
                if self.options.follow.is_none() {
                    Self::release_source(world, &mut self.sources[self.current])?;
                }
                self.current += 1;
                return Ok(Step::Progress);
            }
            if matches!(self.sources[self.current].position, Position::New) {
                self.open_initial(world, stdin_file)?;
                return Ok(Step::Progress);
            }
            return match self.initial_read(world) {
                Ok(step) => Ok(step),
                Err(error) => {
                    let source = &mut self.sources[self.current];
                    self.pending
                        .push_back((2, diagnostic(&source.name, error, false)));
                    self.status = 1;
                    source.position = Position::Done;
                    source.unavailable = true;
                    Self::release_source(world, source)?;
                    source.handle = None;
                    Ok(Step::Progress)
                }
            };
        }
        if self.options.follow.is_none() {
            self.close(world)?;
            return Ok(Step::Done);
        }
        self.follow_step(world, pid)
    }
    fn open_initial(&mut self, world: &mut WorldState, stdin_file: Option<u64>) -> GameResult<()> {
        let index = self.current;
        let source = &mut self.sources[index];
        let actor = world.terminal.user.clone();
        source.path = crate::vfs::normalize(&source.name, &world.terminal.cwd)?;
        if source.name == "-" && self.options.follow == Some(Follow::Name) {
            self.status = 1;
            self.pending
                .push_back((2, b"tail: cannot follow '-' by name\n".to_vec()));
            self.close(world)?;
            return Ok(());
        }
        source.polling = world
            .fs()?
            .lookup_dependencies(&source.path, &actor)
            .iter()
            .any(|p| {
                world
                    .fs()
                    .is_ok_and(|fs| fs.nodes.get(p).is_some_and(|n| n.kind == "symlink"))
            });
        if source.name == "-" {
            source.handle = stdin_file;
            source.tty = stdin_file.is_none() && world.terminal.io.stdin_tty;
        } else {
            match open_named(world, &source.path, &actor) {
                Ok(handle) => {
                    source.handle = Some(handle);
                    source.owned = true;
                }
                Err(error) => {
                    source.missing =
                        crate::terminal_io::error_reason(&error) == "No such file or directory";
                    let directory = world
                        .fs()?
                        .stat(&source.path, &actor)
                        .is_ok_and(|n| n.kind == "directory" && n.mode & 0o444 != 0);
                    source.unavailable = true;
                    source.position = Position::Done;
                    self.status = 1;
                    source.polling = true;
                    let diagnostic = diagnostic(&source.name, error, !directory);
                    if self.options.follow.is_some() && self.options.retry {
                        Self::subscribe(world, source, true)?;
                    }
                    if directory {
                        self.header(index);
                    }
                    self.pending.push_back((2, diagnostic));
                    return Ok(());
                }
            }
        }
        source.position = Position::Read;
        if let Some(handle) = source.handle {
            let (_, offset, size, regular) = world.fs()?.handle_identity(handle)?;
            source.regular = regular;
            if regular {
                source.end = (!self.options.from_start).then_some(size as usize);
                if self.selection.bytes {
                    let begin = if self.options.from_start {
                        offset
                            .saturating_add(self.selection.count.saturating_sub(1) as usize)
                            .min(if self.options.follow.is_some() {
                                crate::binary::MAX_BLOB
                            } else {
                                size as usize
                            })
                    } else {
                        (size as usize)
                            .saturating_sub(self.selection.count as usize)
                            .max(offset)
                    };
                    world.fs_mut()?.seek(handle, begin)?;
                } else if !self.options.from_start {
                    source.position = Position::Reverse {
                        scan: size as usize,
                        floor: offset,
                        left: self.selection.count,
                        last: true,
                    };
                }
            }
            if self.options.follow.is_some() {
                Self::subscribe(world, source, self.options.follow == Some(Follow::Name))?;
            }
        }
        self.header(index);
        Ok(())
    }
    fn initial_read(&mut self, world: &mut WorldState) -> GameResult<Step> {
        let source = &mut self.sources[self.current];
        let Some(handle) = source.handle else {
            return Ok(Step::Input);
        };
        let blobs = world.blobs.clone();
        if let Position::Reverse {
            scan,
            floor,
            left,
            last,
        } = &mut source.position
        {
            if *scan <= *floor || *left == 0 {
                world.fs_mut()?.seek(handle, *scan)?;
                source.position = Position::Read;
                return Ok(Step::Progress);
            }
            let begin = scan.saturating_sub(4096).max(*floor);
            world.fs_mut()?.seek(handle, begin)?;
            let bytes = world
                .fs_mut()?
                .read_handle_bytes(handle, *scan - begin, &blobs)?;
            let mut selected = None;
            for (i, b) in bytes.iter().enumerate().rev() {
                if *b == self.delimiter {
                    if *last {
                        *last = false;
                        continue;
                    }
                    *left -= 1;
                    if *left == 0 {
                        selected = Some(begin + i + 1);
                        break;
                    }
                }
                *last = false;
            }
            *scan = selected.unwrap_or(begin);
            return Ok(Step::Progress);
        }
        let offset = world.fs()?.handle_identity(handle)?.1;
        let limit = source
            .end
            .map_or(1024, |end| end.saturating_sub(offset).min(1024));
        let bytes = if limit == 0 {
            Vec::new()
        } else {
            world.fs_mut()?.read_handle_bytes(handle, limit, &blobs)?
        };
        if bytes.is_empty() {
            source.position = Position::Done;
            if !source.regular {
                let out = source.transform.finish();
                if !out.is_empty() {
                    self.pending.push_back((1, out));
                }
            }
        } else {
            let out = if source.regular && (self.selection.bytes || !self.options.from_start) {
                bytes
            } else {
                source.transform.push(&bytes)
            };
            if !out.is_empty() {
                self.pending.push_back((1, out));
            }
        }
        Ok(Step::Progress)
    }
    fn follow_step(&mut self, world: &mut WorldState, pid: u32) -> GameResult<Step> {
        let writers_dead = !self.options.pids.is_empty()
            && self.options.pids.iter().all(|pid| {
                *pid != 0 && !world.processes.iter().any(|p| p.pid == *pid && p.running)
            });
        let active = self
            .sources
            .iter()
            .any(|s| s.handle.is_some() || s.tty || (self.options.retry && !s.watches.is_empty()));
        if !active {
            // Pipe stdin is finite even with -f; failed named operands diagnose.
            if self.sources.iter().any(|s| s.name != "-") {
                self.status = 1;
                self.pending
                    .push_back((2, b"tail: no files remaining\n".to_vec()));
            }
            self.close(world)?;
            return Ok(Step::Progress);
        }
        for turn in 0..self.sources.len() {
            let index = (self.current + turn) % self.sources.len();
            let source = &mut self.sources[index];
            if source.tty {
                self.current = index;
                return Ok(Step::Input);
            }
            let mut changed = source.watches.iter().any(|(id, revision)| {
                world.fs().ok().and_then(|fs| fs.watch_revision(*id)) != Some(*revision)
            });
            if source.polling && changed && source.timer.is_none() {
                let namespace = source.watches.iter().any(|(watch, _)| {
                    world
                        .fs()
                        .ok()
                        .and_then(|fs| fs.watch_event(*watch))
                        .is_some_and(|e| e.change.namespace)
                });
                let iterations = if namespace && !source.unavailable {
                    self.options.unchanged.unwrap_or(5).saturating_add(1)
                } else {
                    1
                };
                let seconds = self.options.interval.unwrap_or(1.0);
                let delay =
                    (seconds * crate::shell::wait::TICKS_PER_SECOND as f64 * iterations as f64)
                        .min((u64::MAX - world.scheduler.now) as f64) as u64;
                source.timer = Some(world.scheduler.timer(pid, delay)?);
                source.recheck = true;
                for (id, revision) in &mut source.watches {
                    *revision = world.fs()?.watch_revision(*id).unwrap();
                }
            }
            if let Some(timer) = source.timer {
                if !world.scheduler.fired(timer) {
                    continue;
                }
                world.scheduler.cancel_timer(timer);
                source.timer = None;
                changed |= source.recheck;
                source.recheck = false;
            }
            if changed
                && (self.options.follow == Some(Follow::Name)
                    || (self.options.retry && source.unavailable))
                && source.name != "-"
            {
                let actor = world.terminal.user.clone();
                let existing = source
                    .handle
                    .and_then(|h| world.fs().ok()?.handle_identity(h).ok().map(|i| i.0));
                let candidate = open_named(world, &source.path, &actor);
                match candidate {
                    Ok(handle) => {
                        let ino = world.fs()?.handle_identity(handle)?.0;
                        if existing == Some(ino) {
                            world.fs_mut()?.close(handle)?;
                        } else {
                            let message = if source.unavailable && !source.missing {
                                "has become accessible"
                            } else if source.unavailable {
                                "has appeared;  following new file"
                            } else {
                                "has been replaced;  following new file"
                            };
                            if let Some(old) = source.handle.replace(handle) {
                                world.fs_mut()?.close(old)?;
                            }
                            source.owned = true;
                            source.unavailable = false;
                            self.pending.push_back((
                                2,
                                format!(
                                    "tail: {} {message}\n",
                                    options::shell_quote_always(&source.name)
                                )
                                .into_bytes(),
                            ));
                        }
                    }
                    Err(error) => {
                        source.missing =
                            crate::terminal_io::error_reason(&error) == "No such file or directory";
                        if !source.unavailable {
                            let reason = crate::terminal_io::error_reason(error);
                            let message = if self.options.retry {
                                format!(
                                    "tail: {} has become inaccessible: {reason}\n",
                                    options::shell_quote_always(&source.name)
                                )
                            } else {
                                format!("tail: {}: {reason}\n", options::shell_quote(&source.name))
                            };
                            self.pending.push_back((2, message.into_bytes()));
                        }
                        source.unavailable = true;
                        if let Some(old) = source.handle.take() {
                            world.fs_mut()?.close(old)?;
                        }
                    }
                }
                Self::subscribe(
                    world,
                    source,
                    self.options.follow == Some(Follow::Name) || source.unavailable,
                )?;
                if source.unavailable && !self.options.retry {
                    Self::release_source(world, source)?;
                    self.status = 1;
                    return Ok(Step::Progress);
                }
            }
            if let Some(handle) = source.handle {
                let (_, mut offset, size, _) = world.fs()?.handle_identity(handle)?;
                // GNU observes the current stat size, not intermediate kernel
                // mutations. A truncate followed by growth past this offset
                // before the next read must not replay the already-read range.
                if size < (offset as u64) {
                    world.fs_mut()?.seek(handle, 0)?;
                    offset = 0;
                    self.pending.push_back((
                        2,
                        format!(
                            "tail: {}: file truncated\n",
                            options::shell_quote(&source.name)
                        )
                        .into_bytes(),
                    ));
                }
                for (id, revision) in &mut source.watches {
                    *revision = world.fs()?.watch_revision(*id).unwrap();
                    world.fs_mut()?.acknowledge_watch(*id, *revision);
                }
                if (offset as u64) < size {
                    let blobs = world.blobs.clone();
                    let bytes = world.fs_mut()?.read_handle_bytes(handle, 4096, &blobs)?;
                    self.header(index);
                    self.pending.push_back((1, bytes));
                    self.current = (index + 1) % self.sources.len();
                    return Ok(Step::Progress);
                }
            } else {
                for (id, revision) in &mut source.watches {
                    *revision = world.fs()?.watch_revision(*id).unwrap();
                    world.fs_mut()?.acknowledge_watch(*id, *revision);
                }
            }
        }
        if self.pending.is_empty() && writers_dead {
            self.close(world)?;
            return Ok(Step::Done);
        }
        if self.pending.is_empty() {
            Ok(Step::Wait)
        } else {
            Ok(Step::Progress)
        }
    }
}
pub(crate) fn diagnostic(file: &str, reason: impl std::fmt::Display, opening: bool) -> Vec<u8> {
    let reason = crate::terminal_io::error_reason(reason);
    let reason = if reason == "Too many levels of symbolic links" {
        "Symbolic link loop"
    } else {
        &reason
    };
    format!(
        "tail: {} {}{}: {reason}\n",
        if opening {
            "cannot open"
        } else {
            "error reading"
        },
        options::shell_quote_always(file),
        if opening { " for reading" } else { "" }
    )
    .into_bytes()
}
