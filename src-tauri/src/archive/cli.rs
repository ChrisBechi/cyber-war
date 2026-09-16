use super::*;
use crate::{
    terminal_io::{options, Output},
    vfs::normalize,
    world::WorldState,
};
use std::collections::BTreeMap;

pub const COMMANDS: &[&str] = &[
    "zip", "unzip", "tar", "gzip", "gunzip", "zcat", "bzip2", "bunzip2", "xz", "unxz",
];
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Prompt {
    pub message: String,
    pub secret: bool,
}
#[derive(Clone)]
pub struct Pending {
    command: String,
    args: Vec<String>,
    actor: String,
    password: Option<String>,
    decisions: BTreeMap<String, bool>,
    overwrite: Overwrite,
    conflict: Option<String>,
    pub prompt: Prompt,
}
impl std::fmt::Debug for Pending {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArchivePending")
            .field("command", &self.command)
            .finish_non_exhaustive()
    }
}
impl Pending {
    fn new(command: &str, args: &[String], actor: &str) -> Self {
        Self {
            command: command.into(),
            args: args.into(),
            actor: actor.into(),
            password: None,
            decisions: BTreeMap::new(),
            overwrite: Overwrite::Ask,
            conflict: None,
            prompt: Prompt {
                message: String::new(),
                secret: false,
            },
        }
    }
}
pub fn manual(command: &str) -> Option<String> {
    let body = match command {
        "zip" => "zip [-re] ARCHIVE FILE...\n-r includes directories recursively, including dotfiles. -e asks for a password (AES-256). Existing ZIP entries are merged by name.",
        "unzip" => "unzip [-l|-t] [-o|-n] ARCHIVE [-d DIRECTORY]\n-l lists the index; -t verifies all entries. -o replaces, -n skips. Otherwise asks before replacement. Passwords are entered with hidden input.",
        "tar" => "tar -c|-t|-x [-v] [-z|-j|-J] -f ARCHIVE [-C DIRECTORY] [FILE...]\n-c creates, -t lists, -x extracts. -z gzip, -j bzip2, -J xz. Old-style czvf is accepted. Extraction detects compression by signature. -C must exist. TAR preserves virtual modes and safe relative symlinks.",
        "gzip" | "gunzip" | "bzip2" | "bunzip2" | "xz" | "unxz" => "COMMAND [-kdfc] FILE...\n-k keeps input; -d decompresses; -f replaces output; -c writes bytes to stdout and keeps input. Decompressor aliases imply -d. Directories are rejected. By default the input is removed after successful output creation.",
        "zcat" => "zcat FILE.gz...\nDecompresses gzip to stdout without creating files. Supports text pipelines and redirection.",
        _ => return None,
    };
    Some(format!("{command}(1) — CYBER WAR\n\n{body}\n\nOnly virtual files are accessed. Atomic extraction; 4096 entries, 32 MiB materialized data, 16 GiB logical data, 32 path levels, 8 nested archive levels. ZIP64, split archives, device nodes and hardlinks are unsupported.\n"))
}
pub fn execute(
    world: &mut WorldState,
    command: &str,
    args: &[String],
    actor: &str,
) -> Option<GameResult<Output>> {
    COMMANDS
        .contains(&command)
        .then(|| run(world, Pending::new(command, args, actor)))
}
pub fn respond(world: &mut WorldState, input: &str, cancel: bool) -> GameResult<Output> {
    let mut state = world
        .terminal
        .archive_pending
        .take()
        .ok_or_else(|| domain("archive: no pending input"))?;
    if cancel {
        return Ok(Output {
            status: 130,
            stderr: "^C\n".into(),
            ..Output::default()
        });
    }
    if state.prompt.secret {
        if input.is_empty() || input.len() > 1024 {
            return Err(domain("archive: invalid password length"));
        }
        state.password = Some(input.into());
    } else {
        match input.trim() {
            "y" | "yes" => {
                state.decisions.insert(state.conflict.take().unwrap(), true);
            }
            "n" | "no" => {
                state
                    .decisions
                    .insert(state.conflict.take().unwrap(), false);
            }
            "A" => state.overwrite = Overwrite::Replace,
            "N" => state.overwrite = Overwrite::Skip,
            _ => {
                world.terminal.archive_pending = Some(state);
                return Ok(Output::default());
            }
        }
    }
    let path = state
        .args
        .iter()
        .find(|a| !a.starts_with('-'))
        .cloned()
        .unwrap_or_default();
    let result = run(world, state);
    if let Err(e) = &result {
        record_failure(world, &path, &e.to_string(), None);
    }
    result
}
fn ask(
    world: &mut WorldState,
    mut state: Pending,
    message: String,
    secret: bool,
    conflict: Option<String>,
) -> Output {
    state.prompt = Prompt { message, secret };
    state.conflict = conflict;
    world.terminal.archive_pending = Some(state);
    Output::default()
}
fn defer(world: &mut WorldState, state: &Pending) -> GameResult<Option<Output>> {
    if !jobs::may_defer() || world.terminal.shell_depth > 0 {
        return Ok(None);
    }
    let mut parts = vec![state.command.clone()];
    parts.extend(state.args.clone());
    let size = jobs::estimate(world, &parts);
    if size < 8 * 1024 * 1024 {
        return Ok(None);
    }
    let job = jobs::enqueue(
        world,
        jobs::Work::Pending(state.clone()),
        parts.join(" "),
        size,
        false,
    )?;
    Ok(Some(Output {
        archive_job: Some(job.id),
        ..Output::default()
    }))
}
pub(crate) fn run(world: &mut WorldState, state: Pending) -> GameResult<Output> {
    let service = ArchiveService::default();
    let name = state.command.as_str();
    let args = &state.args;
    let actor = &state.actor;
    if args.iter().any(|s| s == "--help") {
        return Ok(Output::success(manual(name).unwrap()));
    }
    match name {
        "zip" => {
            let opts = options(
                name,
                args,
                "re",
                "",
                &[("recurse-paths", 'r'), ("encrypt", 'e')],
            )?;
            let path = opts
                .files
                .first()
                .ok_or_else(|| domain("usage: zip [-re] ARCHIVE FILE..."))?;
            if opts.files.len() < 2 {
                return Err(domain("zip: nothing to do"));
            }
            if opts.has('e') && state.password.is_none() {
                return Ok(ask(
                    world,
                    state.clone(),
                    format!("[{path}] password: "),
                    true,
                    None,
                ));
            }
            if let Some(output) = defer(world, &state)? {
                return Ok(output);
            }
            let result = service.create_archive(
                world,
                path,
                &opts.files[1..],
                ArchiveFormat::Zip,
                opts.has('r'),
                state.password.as_deref(),
                actor,
            )?;
            Ok(Output::success(
                result
                    .entries
                    .iter()
                    .map(|e| format!("  adding: {}\n", e.path))
                    .collect(),
            ))
        }
        "unzip" => {
            let opts = options(name, args, "lton", "d", &[])?;
            if opts.files.len() != 1
                || (opts.has('o') && opts.has('n'))
                || (opts.has('l') && opts.has('t'))
            {
                return Err(domain(
                    "usage: unzip [-l|-t] [-o|-n] ARCHIVE [-d DIRECTORY]",
                ));
            }
            let path = &opts.files[0];
            let info = service.inspect_archive(world, path, actor)?;
            if info.format != ArchiveFormat::Zip {
                return Err(domain("End-of-central-directory signature not found."));
            }
            if opts.has('l') {
                event(
                    world,
                    "ARCHIVE_OPENED",
                    &info.path,
                    None,
                    Vec::new(),
                    Some(info.format),
                );
                return Ok(Output::success(format!(
                    "Archive: {path}\nLength      Name\n{}",
                    info.entries
                        .iter()
                        .map(|e| format!(
                            "{:>10}  {}{}\n",
                            e.original_size,
                            e.path,
                            if e.kind == "directory" { "/" } else { "" }
                        ))
                        .collect::<String>()
                )));
            }
            if info.encrypted && state.password.is_none() {
                return Ok(ask(
                    world,
                    state.clone(),
                    format!("[{path}] password: "),
                    true,
                    None,
                ));
            }
            if opts.has('t') {
                if let Some(output) = defer(world, &state)? {
                    return Ok(output);
                }
                let entries =
                    service.test_archive(world, path, state.password.as_deref(), actor)?;
                return Ok(Output::success(format!(
                    "{}No errors detected.\n",
                    entries
                        .iter()
                        .map(|e| format!("testing: {} OK\n", e.path))
                        .collect::<String>()
                )));
            }
            let options = ExtractOptions {
                destination: opts
                    .counts
                    .iter()
                    .rev()
                    .find(|(c, _)| *c == 'd')
                    .map(|(_, v)| v.clone())
                    .unwrap_or_else(|| world.terminal.cwd.clone()),
                overwrite: if opts.has('o') {
                    Overwrite::Replace
                } else if opts.has('n') {
                    Overwrite::Skip
                } else {
                    state.overwrite.clone()
                },
                selected: Vec::new(),
                decisions: state.decisions.clone(),
            };
            if matches!(options.overwrite, Overwrite::Ask) {
                if let Some(conflict) = service.conflicts(world, path, &options, actor)?.first() {
                    return Ok(ask(
                        world,
                        state.clone(),
                        format!("replace {conflict}? [y]es, [n]o, [A]ll, [N]one: "),
                        false,
                        Some(conflict.clone()),
                    ));
                }
            }
            if let Some(output) = defer(world, &state)? {
                return Ok(output);
            }
            let entries =
                service.extract_archive(world, path, &options, state.password.as_deref(), actor)?;
            Ok(Output::success(format!(
                "Archive: {path}\n{}",
                entries
                    .iter()
                    .map(|p| format!("  inflating: {p}\n"))
                    .collect::<String>()
            )))
        }
        "tar" => {
            let mut args = args.to_vec();
            if let Some(first) = args.first_mut() {
                if !first.starts_with('-') {
                    first.insert(0, '-');
                }
            }
            let opts = options(
                name,
                &args,
                "cxtvzjJ",
                "fC",
                &[
                    ("create", 'c'),
                    ("extract", 'x'),
                    ("list", 't'),
                    ("verbose", 'v'),
                    ("file", 'f'),
                    ("directory", 'C'),
                    ("gzip", 'z'),
                    ("bzip2", 'j'),
                    ("xz", 'J'),
                ],
            )?;
            if ['c', 'x', 't'].iter().filter(|c| opts.has(**c)).count() != 1
                || ['z', 'j', 'J'].iter().filter(|c| opts.has(**c)).count() > 1
            {
                return Err(domain(
                    "tar: select exactly one of -c, -x, -t and at most one compression",
                ));
            }
            let path = opts
                .counts
                .iter()
                .rev()
                .find(|(c, _)| *c == 'f')
                .map(|(_, v)| v)
                .ok_or_else(|| domain("tar: archive file required (-f)"))?;
            let path = normalize(path, &world.terminal.cwd)?;
            let directory = opts
                .counts
                .iter()
                .rev()
                .find(|(c, _)| *c == 'C')
                .map(|(_, v)| normalize(v, &world.terminal.cwd))
                .transpose()?
                .unwrap_or_else(|| world.terminal.cwd.clone());
            world.fs()?.directory(&directory, actor)?;
            let format = if opts.has('z') {
                ArchiveFormat::TarGzip
            } else if opts.has('j') {
                ArchiveFormat::TarBzip2
            } else if opts.has('J') {
                ArchiveFormat::TarXz
            } else {
                ArchiveFormat::Tar
            };
            let entries = if opts.has('c') {
                let cwd = std::mem::replace(&mut world.terminal.cwd, directory);
                let result =
                    service.create_archive(world, &path, &opts.files, format, true, None, actor);
                world.terminal.cwd = cwd;
                result?
                    .entries
                    .into_iter()
                    .map(|e| e.path)
                    .collect::<Vec<_>>()
            } else {
                let inspected = service.inspect_archive(world, &path, actor)?;
                if inspected.format == ArchiveFormat::Zip
                    || inspected.format.is_stream()
                    || (format != ArchiveFormat::Tar && format != inspected.format)
                {
                    return Err(domain("tar: This does not look like a tar archive"));
                }
                if opts.has('t') {
                    event(
                        world,
                        "ARCHIVE_OPENED",
                        &inspected.path,
                        None,
                        Vec::new(),
                        Some(inspected.format),
                    );
                    inspected
                        .entries
                        .iter()
                        .map(|e| {
                            if opts.has('v') {
                                format!(
                                    "{:03o} {:>10} {}{}",
                                    e.permissions,
                                    e.original_size,
                                    e.path,
                                    if e.kind == "directory" { "/" } else { "" }
                                )
                            } else {
                                format!(
                                    "{}{}",
                                    e.path,
                                    if e.kind == "directory" { "/" } else { "" }
                                )
                            }
                        })
                        .collect()
                } else {
                    service.extract_archive(
                        world,
                        &path,
                        &ExtractOptions {
                            destination: directory,
                            overwrite: Overwrite::Replace,
                            selected: opts.files.clone(),
                            decisions: BTreeMap::new(),
                        },
                        None,
                        actor,
                    )?
                }
            };
            Ok(Output::success(if opts.has('v') || opts.has('t') {
                format!("{}\n", entries.join("\n"))
            } else {
                String::new()
            }))
        }
        _ => {
            let opts = options(
                name,
                args,
                "kdfc",
                "",
                &[
                    ("keep", 'k'),
                    ("decompress", 'd'),
                    ("force", 'f'),
                    ("stdout", 'c'),
                ],
            )?;
            if opts.files.is_empty() {
                return Err(domain(format!("{name}: file required")));
            }
            let format = match name {
                "bzip2" | "bunzip2" => ArchiveFormat::Bzip2,
                "xz" | "unxz" => ArchiveFormat::Xz,
                _ => ArchiveFormat::Gzip,
            };
            let decompress =
                opts.has('d') || matches!(name, "gunzip" | "bunzip2" | "unxz" | "zcat");
            let stdout = opts.has('c') || name == "zcat";
            let mut output = Vec::new();
            for path in &opts.files {
                let bytes = if decompress {
                    service.decompress_file(
                        world,
                        path,
                        format,
                        opts.has('k'),
                        opts.has('f'),
                        stdout,
                        actor,
                    )?
                } else {
                    service.compress_file(
                        world,
                        path,
                        format,
                        opts.has('k'),
                        opts.has('f'),
                        stdout,
                        actor,
                    )?
                };
                if output.len() + bytes.len() > 32 * 1024 * 1024 {
                    return Err(domain("archive: stdout size limit exceeded"));
                }
                output.extend(bytes);
            }
            if stdout {
                if decompress {
                    match String::from_utf8(output) {
                        Ok(text) => Ok(Output::success(text)),
                        Err(e) => Ok(Output {
                            binary: Some(e.into_bytes()),
                            ..Output::default()
                        }),
                    }
                } else {
                    Ok(Output {
                        binary: Some(output),
                        ..Output::default()
                    })
                }
            } else {
                Ok(Output::default())
            }
        }
    }
}
