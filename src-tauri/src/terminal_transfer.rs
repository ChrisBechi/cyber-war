//! Coreutils copy/rename subset. Operands operate only on the selected VFS.
use crate::{
    error::GameResult,
    terminal_io::{error_reason, options, Options, Output},
    vfs::{normalize, VirtualFileSystem},
    world::WorldState,
};
use std::collections::BTreeSet;

struct Paths {
    source: String,
    target: String,
    source_label: String,
    target_label: String,
}

struct Transfer<'a> {
    command: &'a str,
    actor: &'a str,
    opts: Options,
    output: Output,
    written: BTreeSet<String>,
}
impl Transfer<'_> {
    fn write(&mut self, fd: u8, text: String) {
        if fd == 1 {
            self.output.stdout.push_str(&text);
        } else {
            self.output.stderr.push_str(&text);
            self.output.status = 1;
        }
        self.output.ordered.push((fd, text));
    }
    fn error(&mut self, message: String) {
        self.write(2, format!("{}: {message}\n", self.command));
    }
    fn verbose(&mut self, paths: &Paths) {
        if self.opts.has('v') {
            let prefix = if self.command == "mv" { "renamed " } else { "" };
            self.write(
                1,
                format!(
                    "{prefix}'{}' -> '{}'\n",
                    paths.source_label, paths.target_label
                ),
            );
        }
    }
    fn no_clobber(&self) -> bool {
        if self.command == "cp" {
            self.opts.has('n')
        } else {
            self.opts
                .flags
                .iter()
                .rev()
                .find(|f| **f == 'n' || **f == 'f')
                == Some(&'n')
        }
    }
    fn entry(&mut self, fs: &mut VirtualFileSystem, mut paths: Paths, depth: usize) {
        if let Ok(path) = fs.resolve(&paths.source, self.actor, crate::vfs::Follow::No) {
            paths.source = path;
        }
        if let Ok(path) =
            fs.resolve_missing(&paths.target, self.actor, crate::vfs::Follow::No, true)
        {
            paths.target = path;
        }
        let original = match fs.lstat(&paths.source, self.actor) {
            Ok(node) => node.clone(),
            Err(error) => {
                self.error(format!(
                    "cannot stat '{}': {}",
                    paths.source_label,
                    error_reason(error)
                ));
                return;
            }
        };
        if paths.source_label.ends_with('/') && original.kind != "directory" {
            self.error(format!(
                "cannot stat '{}': Not a directory",
                paths.source_label
            ));
            return;
        }
        if paths.source == paths.target {
            self.error(format!(
                "'{}' and '{}' are the same file",
                paths.source_label, paths.target_label
            ));
            return;
        }
        if original.kind == "directory"
            && paths
                .target
                .starts_with(&format!("{}/", paths.source.trim_end_matches('/')))
        {
            self.error(format!(
                "cannot {} a directory, '{}', into itself, '{}'",
                if self.command == "cp" { "copy" } else { "move" },
                paths.source_label,
                paths.target_label
            ));
            return;
        }
        if self.command == "cp"
            && original.kind == "directory"
            && !self.opts.has('r')
            && !self.opts.has('R')
        {
            self.error(format!(
                "-r not specified; omitting directory '{}'",
                paths.source_label
            ));
            return;
        }
        let existing = match fs.path_exists(&paths.target, self.actor) {
            Ok(true) => fs.lstat(&paths.target, self.actor).ok().cloned(),
            Ok(false) => None,
            Err(error) => {
                self.error(format!(
                    "cannot access '{}': {}",
                    paths.target_label,
                    error_reason(error)
                ));
                return;
            }
        };
        let merging = self.command == "cp"
            && original.kind == "directory"
            && existing.as_ref().is_some_and(|n| n.kind == "directory");
        if !merging && existing.is_some() && self.no_clobber() {
            return;
        }
        if let Some(target) = &existing {
            if (original.kind == "directory") != (target.kind == "directory") {
                self.error(format!(
                    "cannot overwrite {} '{}' with {}",
                    if target.kind == "directory" {
                        "directory"
                    } else {
                        "non-directory"
                    },
                    paths.target_label,
                    if original.kind == "directory" {
                        "directory"
                    } else {
                        "non-directory"
                    }
                ));
                return;
            }
            if !merging && self.opts.has('u') && original.modified_at <= target.modified_at {
                return;
            }
            if !merging && self.written.contains(&paths.target) {
                self.error(format!(
                    "will not overwrite just-created '{}' with '{}'",
                    paths.target_label, paths.source_label
                ));
                return;
            }
        }
        if self.command == "mv" {
            match fs.rename_entry(&paths.source, &paths.target, self.actor) {
                Ok(()) => {
                    self.written.insert(paths.target.clone());
                    self.verbose(&paths);
                }
                Err(error) => self.error(format!(
                    "cannot move '{}' to '{}': {}",
                    paths.source_label,
                    paths.target_label,
                    error_reason(error)
                )),
            }
            return;
        }
        if original.kind == "directory" {
            if depth >= 256 {
                self.error(format!(
                    "virtual traversal depth limit reached at '{}'",
                    paths.source_label
                ));
                return;
            }
            if existing.is_none() {
                if let Err(error) = fs.mkdir(&paths.target, self.actor) {
                    self.error(format!(
                        "cannot create directory '{}': {}",
                        paths.target_label,
                        error_reason(error)
                    ));
                    return;
                }
                self.verbose(&paths);
            }
            match fs.list(&paths.source, self.actor) {
                Ok(children) => {
                    for child in children {
                        self.entry(
                            fs,
                            Paths {
                                source: child.id,
                                target: join(&paths.target, &child.name),
                                source_label: join(&paths.source_label, &child.name),
                                target_label: join(&paths.target_label, &child.name),
                            },
                            depth + 1,
                        );
                    }
                }
                Err(error) => self.error(format!(
                    "cannot access '{}': {}",
                    paths.source_label,
                    error_reason(error)
                )),
            }
            if existing.is_none() {
                // Apply the source mode after children have been written.
                if let Err(error) = fs.chmod(&paths.target, self.actor, original.mode & !fs.umask) {
                    self.error(error_reason(error));
                }
            }
            return;
        }
        // Default non-recursive cp dereferences a final source symlink. Recursive
        // copies preserve links; ancestors remain subject to the VFS boundary.
        let source = if original.kind == "symlink"
            && !self.opts.has('P')
            && !self.opts.has('r')
            && !self.opts.has('R')
        {
            match follow_link(fs, &paths.source, self.actor) {
                Ok(path) => path,
                Err(error) => {
                    self.error(format!(
                        "cannot stat '{}': {}",
                        paths.source_label,
                        error_reason(error)
                    ));
                    return;
                }
            }
        } else {
            paths.source.clone()
        };
        if source == paths.target {
            self.error(format!(
                "'{}' and '{}' are the same file",
                paths.source_label, paths.target_label
            ));
            return;
        }
        let mut result = fs.copy_entry(&source, &paths.target, self.actor);
        if self.opts.has('f')
            && existing.as_ref().is_some_and(|n| n.kind == "file")
            && result
                .as_ref()
                .is_err_and(|e| error_reason(e) == "Permission denied")
            && fs.readable(&source, self.actor).is_ok()
        {
            // Force retries only an unwritable destination. Keep a failed retry
            // atomic per file; earlier successful operands remain committed.
            let before = fs.clone();
            result = fs
                .remove(&paths.target, self.actor, false)
                .and_then(|()| fs.copy_entry(&source, &paths.target, self.actor));
            if result.is_err() {
                *fs = before;
            }
        }
        match result {
            Ok(()) => {
                self.written.insert(paths.target.clone());
                self.verbose(&paths);
            }
            Err(error) => self.error(format!(
                "cannot copy '{}' to '{}': {}",
                paths.source_label,
                paths.target_label,
                error_reason(error)
            )),
        }
    }
}

fn follow_link(fs: &VirtualFileSystem, path: &str, actor: &str) -> GameResult<String> {
    fs.resolve(path, actor, crate::vfs::Follow::Yes)
}

fn join(directory: &str, name: &str) -> String {
    format!("{}/{name}", directory.trim_end_matches('/'))
}

pub(crate) fn execute(
    world: &mut WorldState,
    command: &str,
    args: &[String],
    actor: &str,
) -> GameResult<Output> {
    let mut aliases = vec![
        ("no-clobber", 'n'),
        ("verbose", 'v'),
        ("force", 'f'),
        ("target-directory", 't'),
        ("no-target-directory", 'T'),
        ("version", 'V'),
    ];
    if command == "cp" {
        aliases.extend([("recursive", 'r'), ("no-dereference", 'P')]);
    }
    let opts = options(
        command,
        args,
        if command == "cp" { "rRnvfuTP" } else { "nvfuT" },
        "t",
        &aliases,
    )?;
    if opts.help {
        return Ok(Output::success(
            crate::terminal_io::manual(command).unwrap_or_default(),
        ));
    }
    if opts.has('V') {
        return Ok(Output::success(format!("{command} (GNU coreutils) 9.7\n")));
    }
    let mut transfer = Transfer {
        command,
        actor,
        opts,
        output: Output::default(),
        written: BTreeSet::new(),
    };
    let targets: Vec<_> = transfer
        .opts
        .counts
        .iter()
        .filter(|(flag, _)| *flag == 't')
        .map(|(_, value)| value.clone())
        .collect();
    if targets.len() > 1 {
        transfer.error("multiple target directories specified".into());
        return Ok(transfer.output);
    }
    if !targets.is_empty() && transfer.opts.has('T') {
        transfer
            .error("cannot combine --target-directory (-t) and --no-target-directory (-T)".into());
        return Ok(transfer.output);
    }
    let mut sources = transfer.opts.files.clone();
    let explicit_target = targets.first().cloned();
    let destination_label = if let Some(target) = &explicit_target {
        target.clone()
    } else {
        if sources.len() < 2 {
            let detail = sources.first().map_or("missing file operand".into(), |s| {
                format!("missing destination file operand after '{s}'")
            });
            transfer.error(format!(
                "{detail}\nTry '{command} --help' for more information."
            ));
            return Ok(transfer.output);
        }
        sources.pop().unwrap()
    };
    if sources.is_empty() {
        transfer.error(format!(
            "missing file operand\nTry '{command} --help' for more information."
        ));
        return Ok(transfer.output);
    }
    if transfer.opts.has('T') && sources.len() > 1 {
        transfer.error(format!(
            "extra operand '{}'\nTry '{command} --help' for more information.",
            sources[1]
        ));
        return Ok(transfer.output);
    }
    let destination = normalize(&destination_label, &world.terminal.cwd)?;
    let directory = world
        .fs()?
        .stat(&destination, actor)
        .is_ok_and(|n| n.kind == "directory");
    if (explicit_target.is_some() || sources.len() > 1) && !directory {
        let reason = match world.fs()?.stat(&destination, actor) {
            Ok(_) => "Not a directory".into(),
            Err(error) => error_reason(error),
        };
        transfer.error(format!("target '{destination_label}': {reason}"));
        return Ok(transfer.output);
    }
    if destination_label.ends_with('/') && !directory {
        transfer.error(format!(
            "cannot access '{destination_label}': Not a directory"
        ));
        return Ok(transfer.output);
    }
    let into_directory = directory && !transfer.opts.has('T');
    for label in sources {
        let source = match normalize(&label, &world.terminal.cwd) {
            Ok(path) => path,
            Err(error) => {
                transfer.error(format!("cannot stat '{label}': {}", error_reason(error)));
                continue;
            }
        };
        let name = if label.trim_end_matches('/').rsplit('/').next() == Some(".") {
            "."
        } else {
            source.rsplit('/').next().unwrap_or("")
        };
        let target = if into_directory {
            normalize(&join(&destination, name), "/")?
        } else {
            destination.clone()
        };
        let target_label = if into_directory {
            join(&destination_label, name)
        } else {
            destination_label.clone()
        };
        transfer.entry(
            world.fs_mut()?,
            Paths {
                source,
                target,
                source_label: label,
                target_label,
            },
            0,
        );
    }
    Ok(transfer.output)
}
