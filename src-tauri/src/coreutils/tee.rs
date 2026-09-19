//! Incremental byte fan-out. Scheduling, descriptors and signals are shared.
use super::foundation_options;
use crate::{
    error::GameResult,
    terminal_io::Output,
    vfs::{normalize, OpenFlags},
    world::WorldState,
};

// Observed read boundary of the locked musl oracle; stdout/files are unbuffered.
pub(crate) const READ_SIZE: usize = 1024;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum ErrorMode {
    #[default]
    Signal,
    Warn,
    WarnNoPipe,
    Exit,
    ExitNoPipe,
}
impl ErrorMode {
    pub fn parse(value: Option<&str>, invocation: &str) -> Result<Self, Box<Output>> {
        let Some(value) = value else {
            return Ok(Self::WarnNoPipe);
        };
        let modes = [
            ("warn", Self::Warn),
            ("warn-nopipe", Self::WarnNoPipe),
            ("exit", Self::Exit),
            ("exit-nopipe", Self::ExitNoPipe),
        ];
        if let Some((_, mode)) = modes.iter().find(|(name, _)| *name == value) {
            return Ok(*mode);
        }
        let found: Vec<_> = modes
            .iter()
            .filter(|(name, _)| name.starts_with(value))
            .collect();
        if found.len() == 1 {
            return Ok(found[0].1);
        }
        Err(Box::new(Output { status: 1, stderr: format!("tee: {} argument {} for '--output-error'\nValid arguments are:\n  - 'warn'\n  - 'warn-nopipe'\n  - 'exit'\n  - 'exit-nopipe'\nTry '{invocation} --help' for more information.\n", if found.is_empty() { "invalid" } else { "ambiguous" }, foundation_options::quote(value)), ..Default::default() }))
    }
    pub fn fatal(self, pipe: bool) -> bool {
        self == Self::Exit || (self == Self::ExitNoPipe && !pipe)
    }
    pub fn diagnose(self, pipe: bool) -> bool {
        !pipe || matches!(self, Self::Warn | Self::Exit)
    }
}

pub(crate) struct Tee {
    operands: Vec<String>,
    pub handles: Vec<(String, Option<u64>)>,
    pub opened: bool,
    pub data: Option<Vec<u8>>,
    pub stdout: bool,
    pub mode: ErrorMode,
    pub ignore_interrupts: bool,
    append: bool,
}
impl Tee {
    pub fn new(invocation: &str, args: &[String], posix: bool) -> Result<Self, Box<Output>> {
        let opts = foundation_options::parse("tee", invocation, args, posix)?;
        if let Some(kind) = opts.special {
            return Err(Box::new(Output::success(
                super::foundation_messages::message("tee", invocation, kind),
            )));
        }
        Ok(Self {
            operands: opts.files,
            handles: Vec::new(),
            opened: false,
            data: None,
            stdout: true,
            mode: opts.tee_mode,
            ignore_interrupts: opts.flags.contains(&'i'),
            append: opts.flags.contains(&'a'),
        })
    }
    pub fn open(&mut self, world: &mut WorldState) -> (Vec<u8>, bool) {
        self.opened = true;
        let mut errors = Vec::new();
        for name in &self.operands {
            let actor = world.terminal.user.clone();
            let opened = normalize(name, &world.terminal.cwd).and_then(|path| {
                world.fs_mut()?.open(
                    &path,
                    OpenFlags {
                        write: true,
                        create: true,
                        truncate: !self.append,
                        append: self.append,
                        ..Default::default()
                    },
                    0o666,
                    &actor,
                )
            });
            match opened {
                Ok(handle) => self.handles.push((name.clone(), Some(handle))),
                Err(error) => {
                    errors.extend(diagnostic(name, error));
                    if self.mode.fatal(false) {
                        return (errors, true);
                    }
                }
            }
        }
        (errors, false)
    }
    pub fn write_files(&mut self, world: &mut WorldState) -> (Vec<u8>, bool) {
        let data = self.data.take().expect("one bounded fan-out chunk");
        let mut errors = Vec::new();
        for (name, handle) in &mut self.handles {
            let Some(id) = *handle else { continue };
            if let Err(error) = super::io::write_handle(world, id, &data) {
                errors.extend(diagnostic(name, error));
                let _ = world.fs_mut().and_then(|fs| fs.close(id));
                *handle = None;
                if self.mode.fatal(false) {
                    return (errors, true);
                }
            }
        }
        (errors, false)
    }
    pub fn active(&self) -> bool {
        self.stdout || self.handles.iter().any(|(_, h)| h.is_some())
    }
    pub fn close(&mut self, world: &mut WorldState) -> GameResult<()> {
        for (_, handle) in &mut self.handles {
            if let Some(id) = handle.take() {
                world.fs_mut()?.close(id)?;
            }
        }
        self.data = None;
        Ok(())
    }
}
pub(crate) fn diagnostic(name: &str, error: impl std::fmt::Display) -> Vec<u8> {
    let reason = crate::terminal_io::error_reason(error);
    let reason = match reason.as_str() {
        "Too many levels of symbolic links" => "Symbolic link loop",
        "No space left on virtual device" => "No space left on device",
        _ => &reason,
    };
    format!("tee: {}: {reason}\n", foundation_options::shell_quote(name)).into_bytes()
}
