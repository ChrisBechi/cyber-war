use super::{
    backup::Strategy,
    foundation_options::{error, parse_checked, shell_quote, shell_quote_always},
    pathnames::reason,
};
use crate::{
    error::{GameError, GameResult},
    terminal_io::Output,
    vfs::{self, CanonicalMode, Errno, Follow},
    world::WorldState,
};
use std::collections::{BTreeMap, VecDeque};

struct Job {
    source: String,
    dest: String,
    first: Option<GameResult<()>>,
}
pub(crate) struct Links {
    invocation: String,
    jobs: VecDeque<Job>,
    pending: Option<Job>,
    symbolic: bool,
    logical: bool,
    relative: bool,
    force: bool,
    interactive: bool,
    verbose: bool,
    directories: bool,
    backup: Strategy,
    suffix: String,
    protect_created: bool,
    created: BTreeMap<String, u64>,
    pub waiting: bool,
    pub finished: bool,
    input: VecDeque<u8>,
    first_answer: Option<u8>,
    input_eof: bool,
    input_failed: bool,
}
fn failure(message: impl AsRef<str>) -> Output {
    Output {
        status: 1,
        stderr: format!("ln: {}\n", message.as_ref()),
        ..Default::default()
    }
}
fn access(path: &str, e: GameError) -> Output {
    failure(format!(
        "failed to access {}: {}",
        shell_quote_always(path),
        reason(e)
    ))
}
fn absolute(world: &WorldState, path: &str) -> GameResult<String> {
    vfs::absolute(path, &world.terminal.cwd)
}
fn attempt(
    world: &mut WorldState,
    source: &str,
    dest: &str,
    symbolic: bool,
    logical: bool,
    replace: bool,
) -> GameResult<()> {
    let actor = world.terminal.user.clone();
    let dest = absolute(world, dest)?;
    if symbolic {
        world
            .fs_mut()?
            .symlink_replace(&dest, source, &actor, replace)
    } else {
        let source = absolute(world, source)?;
        world
            .fs_mut()?
            .link_replace(&source, &dest, &actor, logical, replace)
    }
}
impl Links {
    pub fn new(
        invocation: &str,
        args: &[String],
        posix: bool,
        world: &mut WorldState,
    ) -> Result<Self, Box<Output>> {
        let actor = world.terminal.user.clone();
        let mut saw_target = false;
        let opts = parse_checked("ln", invocation, args, posix, |ch, value| {
            if ch == 't' {
                if saw_target {
                    return Err(Box::new(failure("multiple target directories specified")));
                }
                saw_target = true;
                let target = value.unwrap();
                let node =
                    absolute(world, target).and_then(|p| world.fs()?.stat(&p, &actor).cloned());
                match node {
                    Ok(n) if n.kind == "directory" => {}
                    Ok(_) => {
                        return Err(Box::new(failure(format!(
                            "target {} is not a directory",
                            shell_quote_always(target)
                        ))))
                    }
                    Err(e) => return Err(Box::new(access(target, e))),
                }
            }
            Ok(())
        })?;
        if let Some(kind) = opts.special {
            return Err(Box::new(Output::success(
                super::foundation_messages::message("ln", invocation, kind),
            )));
        }
        if opts.files.is_empty() {
            return Err(Box::new(error(
                "ln",
                invocation,
                "missing file operand",
                false,
            )));
        }
        let has = |ch| opts.flags.contains(&ch);
        let value = |ch| {
            opts.path_values
                .iter()
                .rev()
                .find(|(c, _)| *c == ch)
                .map(|(_, v)| v.clone())
        };
        let symbolic = has('s');
        let relative = has('r');
        if relative && !symbolic {
            return Err(Box::new(failure("cannot do --relative without --symbolic")));
        }
        let logical = opts.flags.iter().rev().find(|c| matches!(c, 'L' | 'P')) == Some(&'L');
        let force = opts.flags.iter().rev().find(|c| matches!(c, 'f' | 'i')) == Some(&'f');
        let interactive = opts.flags.iter().rev().find(|c| matches!(c, 'f' | 'i')) == Some(&'i');
        let mut directory = value('t');
        let mut files = opts.files.clone();
        let mut first = None;
        if has('T') {
            if directory.is_some() {
                return Err(Box::new(failure(
                    "cannot combine --target-directory and --no-target-directory",
                )));
            }
            if files.len() != 2 {
                let message = if files.len() < 2 {
                    format!(
                        "missing destination file operand after {}",
                        shell_quote_always(&files[0])
                    )
                } else {
                    format!("extra operand {}", shell_quote_always(&files[2]))
                };
                return Err(Box::new(error("ln", invocation, &message, false)));
            }
        } else if files.len() < 2 && directory.is_none() {
            directory = Some(".".into());
        } else {
            if files.len() == 2 && directory.is_none() && !relative {
                first = Some(attempt(
                    world, &files[0], &files[1], symbolic, logical, false,
                ));
            }
            let inspect = first.as_ref().is_none_or(|r| {
                matches!(
                    r,
                    Err(GameError::Vfs(
                        Errno::Exists | Errno::NotDirectory | Errno::Invalid
                    ))
                )
            });
            if inspect {
                let d = directory.as_ref().unwrap_or_else(|| files.last().unwrap());
                let lookup = absolute(world, d).and_then(|p| {
                    let fs = world.fs()?;
                    let node = if has('n') {
                        fs.lstat(&p, &actor)?
                    } else {
                        fs.stat(&p, &actor)?
                    };
                    if node.kind != "directory" {
                        Err(GameError::Vfs(Errno::NotDirectory))
                    } else {
                        Ok(())
                    }
                });
                match lookup {
                    Ok(()) => {
                        if directory.is_none() {
                            directory = files.pop();
                        }
                        first = None;
                    }
                    Err(e) if directory.is_some() || files.len() != 2 => {
                        return Err(Box::new(failure(format!(
                            "target {}: {}",
                            shell_quote_always(d),
                            reason(e)
                        ))))
                    }
                    Err(_) => {}
                }
            }
        }
        let env = super::foundation::exported(world, &actor);
        let backup = if has('b') || has('S') {
            let control = value('b');
            let source = if control.is_some() {
                "backup type"
            } else {
                "$VERSION_CONTROL"
            };
            Strategy::parse(
                control
                    .as_deref()
                    .or_else(|| env.get("VERSION_CONTROL").map(String::as_str))
                    .unwrap_or(""),
                source,
                invocation,
            )?
        } else {
            Strategy::None
        };
        let suffix = value('S')
            .or_else(|| env.get("SIMPLE_BACKUP_SUFFIX").cloned())
            .filter(|s| !s.is_empty() && !s.contains('/'))
            .unwrap_or("~".into());
        let protect_created = directory.is_some()
            && files.len() >= 2
            && force
            && !symbolic
            && backup != Strategy::Numbered;
        let jobs = if let Some(dir) = directory {
            files
                .into_iter()
                .map(|source| {
                    let stripped = source.trim_end_matches('/');
                    let base = stripped.rsplit('/').next().unwrap_or("");
                    let dest = format!(
                        "{}{base}",
                        if dir.ends_with('/') {
                            dir.clone()
                        } else {
                            format!("{dir}/")
                        }
                    );
                    Job {
                        source,
                        dest,
                        first: None,
                    }
                })
                .collect()
        } else {
            vec![Job {
                source: files[0].clone(),
                dest: files[1].clone(),
                first,
            }]
            .into()
        };
        Ok(Self {
            invocation: invocation.into(),
            jobs,
            pending: None,
            symbolic,
            logical,
            relative,
            force,
            interactive,
            verbose: has('v'),
            directories: has('d') || has('F'),
            backup,
            suffix,
            protect_created,
            created: BTreeMap::new(),
            waiting: false,
            finished: false,
            input: VecDeque::new(),
            first_answer: None,
            input_eof: false,
            input_failed: false,
        })
    }
    fn inode(
        &self,
        world: &WorldState,
        path: &str,
        follow: bool,
    ) -> GameResult<crate::vfs::VfsNode> {
        let p = absolute(world, path)?;
        let fs = world.fs()?;
        let actor = &world.terminal.user;
        if follow {
            fs.stat(&p, actor).cloned()
        } else {
            fs.lstat(&p, actor).cloned()
        }
    }
    fn relative_source(&self, world: &WorldState, source: &str, dest: &str) -> String {
        let result = (|| -> GameResult<String> {
            let source = absolute(world, source)?;
            let dest = absolute(world, dest)?;
            let fs = world.fs()?;
            let actor = &world.terminal.user;
            let from = fs.canonicalize(&source, actor, CanonicalMode::Missing, false)?;
            let dir = fs.canonicalize(vfs::parent(&dest), actor, CanonicalMode::Missing, false)?;
            Ok(vfs::relative_path(&from, &dir))
        })();
        result.unwrap_or_else(|_| source.into())
    }
    pub fn advance(&mut self, world: &mut WorldState) -> Output {
        let Some(mut job) = self.jobs.pop_front() else {
            self.finished = true;
            return if self.input_failed {
                failure("error closing file")
            } else {
                Output::default()
            };
        };
        if job.first.is_none() && !self.relative {
            job.first = Some(attempt(
                world,
                &job.source,
                &job.dest,
                self.symbolic,
                self.logical,
                false,
            ));
        }
        if job.first.as_ref().is_some_and(Result::is_ok) {
            return self.success(world, &job, None);
        }
        let source = if self.symbolic {
            None
        } else {
            match self.inode(world, &job.source, self.logical) {
                Ok(n) => Some(n),
                Err(e) => return access(&job.source, e),
            }
        };
        if source.as_ref().is_some_and(|n| n.kind == "directory") && !self.directories {
            return failure(format!(
                "{}: hard link not allowed for directory",
                shell_quote(&job.source)
            ));
        }
        if self.relative {
            job.source = self.relative_source(world, &job.source, &job.dest);
        }
        if self.force || self.interactive || self.backup != Strategy::None {
            match self.inode(world, &job.dest, false) {
                Ok(dest) => {
                    if dest.kind == "directory" {
                        return failure(format!(
                            "{}: cannot overwrite directory",
                            shell_quote(&job.dest)
                        ));
                    }
                    let path = absolute(world, &job.dest)
                        .and_then(|p| world.fs()?.resolve(&p, &world.terminal.user, Follow::No))
                        .unwrap_or_else(|_| job.dest.clone());
                    if self.created.get(&path) == Some(&dest.ino) {
                        return failure(format!(
                            "will not overwrite just-created {} with {}",
                            shell_quote_always(&job.dest),
                            shell_quote_always(&job.source)
                        ));
                    }
                    if if self.backup != Strategy::None {
                        !self.symbolic
                    } else {
                        self.force
                    } {
                        let source = source.or_else(|| self.inode(world, &job.source, true).ok());
                        if let Some(source) = source {
                            let same_name = absolute(world, &job.source)
                                .and_then(|p| {
                                    world.fs()?.resolve(&p, &world.terminal.user, Follow::No)
                                })
                                .is_ok_and(|p| p == path);
                            if source.ino == dest.ino && (source.nlink == 1 || same_name) {
                                return failure(format!(
                                    "{} and {} are the same file",
                                    shell_quote_always(&job.source),
                                    shell_quote_always(&job.dest)
                                ));
                            }
                        }
                    }
                    if self.interactive && Self::replaceable(&job) {
                        let out = Output {
                            stderr: format!(
                                "{}: replace {}? ",
                                self.invocation,
                                shell_quote_always(&job.dest)
                            ),
                            ..Default::default()
                        };
                        self.pending = Some(job);
                        self.waiting = true;
                        self.first_answer = None;
                        return out;
                    }
                }
                Err(GameError::Vfs(Errno::NotFound)) => {}
                Err(e) => return access(&job.dest, e),
            }
        }
        self.commit(world, job)
    }
    fn replaceable(job: &Job) -> bool {
        job.first
            .as_ref()
            .is_none_or(|r| matches!(r, Err(GameError::Vfs(Errno::Exists))))
    }
    fn success(&mut self, world: &WorldState, job: &Job, backup: Option<&str>) -> Output {
        if self.protect_created {
            if let Ok(path) = absolute(world, &job.dest)
                .and_then(|p| world.fs()?.resolve(&p, &world.terminal.user, Follow::No))
            {
                if let Ok(node) = world
                    .fs()
                    .and_then(|fs| fs.lstat(&path, &world.terminal.user))
                {
                    self.created.insert(path, node.ino);
                }
            }
        }
        Output {
            stdout: if self.verbose {
                format!(
                    "{}{} {}> {}\n",
                    backup
                        .map(|b| format!("{} ~ ", shell_quote_always(b)))
                        .unwrap_or_default(),
                    shell_quote_always(&job.dest),
                    if self.symbolic { '-' } else { '=' },
                    shell_quote_always(&job.source)
                )
            } else {
                String::new()
            },
            ..Default::default()
        }
    }
    fn commit(&mut self, world: &mut WorldState, mut job: Job) -> Output {
        let actor = world.terminal.user.clone();
        let replace = self.force || self.interactive || self.backup != Strategy::None;
        let mut backup = None;
        if self.backup != Strategy::None
            && Self::replaceable(&job)
            && self.inode(world, &job.dest, false).is_ok()
        {
            let dest = absolute(world, &job.dest).expect("previously resolved destination");
            let name = self
                .backup
                .name(world.fs().unwrap(), &dest, &actor, &self.suffix);
            match world
                .fs_mut()
                .and_then(|fs| fs.rename_entry(&dest, &name, &actor))
            {
                Ok(()) => {
                    backup = Some((
                        name.clone(),
                        job.dest.rsplit_once('/').map_or_else(
                            || name.rsplit('/').next().unwrap().to_string(),
                            |(dir, _)| format!("{dir}/{}", name.rsplit('/').next().unwrap()),
                        ),
                    ))
                }
                Err(GameError::Vfs(Errno::NotFound)) => {}
                Err(e) => {
                    return failure(format!(
                        "cannot backup {}: {}",
                        shell_quote_always(&job.dest),
                        reason(e)
                    ))
                }
            }
        }
        let result = if Self::replaceable(&job) && (replace || job.first.is_none()) {
            attempt(
                world,
                &job.source,
                &job.dest,
                self.symbolic,
                self.logical,
                replace,
            )
        } else {
            job.first.take().unwrap_or_else(|| {
                attempt(
                    world,
                    &job.source,
                    &job.dest,
                    self.symbolic,
                    self.logical,
                    false,
                )
            })
        };
        match result {
            Ok(()) => self.success(
                world,
                &job,
                backup.as_ref().map(|(_, shown)| shown.as_str()),
            ),
            Err(e) => {
                let arrow = if self.symbolic {
                    job.source.is_empty() || matches!(e, GameError::Vfs(Errno::NameTooLong))
                } else {
                    !matches!(
                        e,
                        GameError::Vfs(Errno::Exists | Errno::NoSpace | Errno::ReadOnly)
                    )
                };
                let mut out = failure(format!(
                    "failed to create {} link {}{}: {}",
                    if self.symbolic { "symbolic" } else { "hard" },
                    shell_quote_always(&job.dest),
                    if arrow {
                        format!(
                            " {}> {}",
                            if self.symbolic { '-' } else { '=' },
                            shell_quote_always(&job.source)
                        )
                    } else {
                        String::new()
                    },
                    reason(e)
                ));
                if let Some((backup, _)) = backup {
                    if let Err(e) = absolute(world, &job.dest)
                        .and_then(|dest| world.fs_mut()?.rename_entry(&backup, &dest, &actor))
                    {
                        out.stderr.push_str(&format!(
                            "ln: cannot un-backup {}: {}\n",
                            shell_quote_always(&job.dest),
                            reason(e)
                        ));
                    }
                }
                out
            }
        }
    }
    pub fn feed(&mut self, data: Vec<u8>) {
        self.input.extend(data);
    }
    pub fn read_error(&mut self) {
        self.input_failed = true;
        self.input_eof = true;
    }
    pub fn eof(&mut self) {
        self.input_eof = true;
    }
    pub fn answer(&mut self, world: &mut WorldState) -> Option<Output> {
        while let Some(byte) = self.input.pop_front() {
            if byte == b'\n' {
                return Some(self.finish_answer(world));
            }
            if self.first_answer.is_none() {
                self.first_answer = Some(byte);
            }
        }
        if self.input_eof {
            Some(self.finish_answer(world))
        } else {
            None
        }
    }
    fn finish_answer(&mut self, world: &mut WorldState) -> Output {
        self.waiting = false;
        let job = self.pending.take().expect("pending interactive link");
        if matches!(self.first_answer, Some(b'y' | b'Y')) {
            self.commit(world, job)
        } else {
            Output {
                status: 1,
                ..Default::default()
            }
        }
    }
}
