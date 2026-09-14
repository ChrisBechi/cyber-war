//! Audited text/file command subset. All reads and writes use the selected VFS.
use crate::{
    error::GameResult,
    vfs::{domain, normalize, HOME},
    world::WorldState,
};

const MAX_OUTPUT: usize = 4 * 1024 * 1024;

fn output_limit(size: usize) -> GameResult<()> {
    if size > MAX_OUTPUT {
        Err(domain("virtual terminal output limit: 4 MiB"))
    } else {
        Ok(())
    }
}

#[derive(Default, Debug)]
pub struct Output {
    pub stdout: String,
    pub stderr: String,
    pub status: i32,
}
impl Output {
    pub fn success(stdout: String) -> Self {
        Self {
            stdout,
            ..Self::default()
        }
    }
    pub(crate) fn error(&mut self, command: &str, file: &str, error: impl std::fmt::Display) {
        self.stderr
            .push_str(&format!("{command}: {file}: {error}\n"));
        self.status = 1;
    }
}

#[derive(Default)]
pub(crate) struct Options {
    pub flags: Vec<char>,
    pub files: Vec<String>,
    pub counts: Vec<(char, String)>,
    pub help: bool,
}
impl Options {
    pub(crate) fn has(&self, flag: char) -> bool {
        self.flags.contains(&flag)
    }
}

pub(crate) fn options(
    command: &str,
    args: &[String],
    switches: &str,
    values: &str,
    aliases: &[(&str, char)],
) -> GameResult<Options> {
    let mut parsed = Options::default();
    let mut end = false;
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if end || !arg.starts_with('-') || arg == "-" {
            parsed.files.push(arg.clone());
        } else if arg == "--" {
            end = true;
        } else if arg == "--help" {
            parsed.help = true;
        } else if let Some(long) = arg.strip_prefix("--") {
            let (name, attached) = long
                .split_once('=')
                .map_or((long, None), |(n, v)| (n, Some(v)));
            let flag = aliases
                .iter()
                .find(|(alias, _)| *alias == name)
                .map(|(_, flag)| *flag)
                .ok_or_else(|| {
                    domain(format!(
                        "{command}: option '{arg}' is not supported by the virtual subset"
                    ))
                })?;
            if values.contains(flag) {
                let value = if let Some(value) = attached {
                    value.to_string()
                } else {
                    index += 1;
                    args.get(index).cloned().ok_or_else(|| {
                        domain(format!("{command}: option '--{name}' requires an argument"))
                    })?
                };
                parsed.counts.push((flag, value));
            } else if attached.is_none() {
                parsed.flags.push(flag);
            } else {
                return Err(domain(format!(
                    "{command}: option '--{name}' does not allow an argument"
                )));
            }
        } else if index == 0
            && matches!(command, "head" | "tail")
            && arg[1..].chars().all(|c| c.is_ascii_digit())
        {
            parsed.counts.push(('n', arg[1..].to_string()));
        } else {
            for (offset, flag) in arg[1..].char_indices() {
                if values.contains(flag) {
                    let rest = &arg[1 + offset + flag.len_utf8()..];
                    let value = if rest.is_empty() {
                        index += 1;
                        args.get(index).cloned().ok_or_else(|| {
                            domain(format!("{command}: option '-{flag}' requires an argument"))
                        })?
                    } else {
                        rest.into()
                    };
                    parsed.counts.push((flag, value));
                    break;
                } else if switches.contains(flag) {
                    parsed.flags.push(flag);
                } else {
                    return Err(domain(format!(
                        "{command}: option '-{flag}' is not supported by the virtual subset"
                    )));
                }
            }
        }
        index += 1;
    }
    Ok(parsed)
}

pub fn manual(command: &str) -> Option<String> {
    let syntax = match command {
        "pwd" => "pwd [-L|-P] [--]",
        "cd" => "cd [-L|-P] [--] [DIRECTORY|-]",
        "cat" => "cat [-nbEsTu] [--number] [--number-nonblank] [--show-ends] [--show-tabs] [--squeeze-blank] [--] FILE...",
        "head" => "head [-qv] [-n N|-n -N|-c N|-c -N] [--lines=N|--bytes=N] [--] FILE...",
        "tail" => "tail [-qv] [-n N|-n +N|-c N|-c +N] [--lines=N|--bytes=N] [--] FILE...",
        "cp" => "cp [-rRnv] [--recursive] [--no-clobber] [--verbose] [--] SOURCE DESTINATION",
        "mv" => "mv [-nv] [--no-clobber] [--verbose] [--] SOURCE DESTINATION",
        "rm" => "rm [-rRfv] [--recursive] [--force] [--verbose] [--] PATH...",
        _ => return None,
    };
    let detail = match command {
        "pwd" | "cd" => "-L/-P are equivalent: this VFS has no symbolic links. cd updates PWD/OLDPWD; cd - prints the previous directory. CDPATH is not implemented.",
        "cat" => "-b overrides -n. Numbering continues across files. Newlines and tabs are preserved unless explicitly displayed/transformed. -u is accepted with no buffering effect. -v/-e/-t/-A are not implemented.",
        "head" | "tail" => "Default: 10 lines. -q/--quiet/--silent suppress headers; -v/--verbose always shows headers. Decimal counts only. -n and -c accept attached or separate counts; the last one wins. Legacy -N is accepted only first. No follow mode, suffix multipliers or zero delimiters. Byte slices splitting UTF-8 cannot be represented by this text VFS and return an explicit error.",
        "cp" | "mv" => "Two operands only. -n skips existing targets; -v reports changes. Existing regular files may be replaced; existing directory merging and interactive prompts are not implemented. Operations are atomic in the game: errors roll back this command. Metadata is virtual, not a complete POSIX inode implementation.",
        "rm" => "Permanent VFS deletion, never GUI trash. -f ignores missing files but not permissions. -r/-R is required for directories. Virtual root and trash infrastructure are protected. Atomic command rollback on errors.",
        _ => "",
    };
    Some(format!("{command}(1) — CYBER WAR virtual subset\n\nSYNOPSIS\n  {syntax}\n\n{detail}\n\nOnly the selected local/SSH virtual filesystem is used.\nInteractive stdin and pipelines are not implemented. Unsupported options return an error.\n"))
}

pub fn execute(
    world: &mut WorldState,
    command: &str,
    args: &[String],
    actor: &str,
) -> Option<GameResult<Output>> {
    match command {
        "pwd" | "cd" => Some(navigation(world, command, args, actor)),
        "cat" => Some(cat(world, args, actor)),
        "head" | "tail" => Some(slice(world, command, args, actor)),
        "cp" | "mv" => Some(transfer(world, command, args, actor)),
        "rm" => Some(remove(world, args, actor)),
        _ => None,
    }
}

fn navigation(
    world: &mut WorldState,
    command: &str,
    args: &[String],
    actor: &str,
) -> GameResult<Output> {
    let opts = options(command, args, "LP", "", &[])?;
    if opts.help {
        return Ok(Output::success(manual(command).unwrap_or_default()));
    }
    if command == "pwd" {
        if !opts.files.is_empty() {
            return Err(domain("pwd: operands are not supported by this subset"));
        }
        return Ok(Output::success(format!("{}\n", world.terminal.cwd)));
    }
    if opts.files.len() > 1 {
        return Err(domain("cd: too many arguments"));
    }
    let home = world.terminal.env.get("HOME").cloned().unwrap_or_else(|| {
        if actor == "root" {
            "/root".into()
        } else {
            HOME.into()
        }
    });
    let requested = opts.files.first().map(String::as_str).unwrap_or(&home);
    let previous = if requested == "-" {
        Some(
            world
                .terminal
                .env
                .get("OLDPWD")
                .cloned()
                .ok_or_else(|| domain("cd: OLDPWD not set"))?,
        )
    } else {
        None
    };
    if requested.is_empty() {
        return Ok(Output::default());
    }
    let next = normalize(
        previous.as_deref().unwrap_or(requested),
        &world.terminal.cwd,
    )?;
    world.fs()?.directory(&next, actor)?;
    world
        .terminal
        .env
        .insert("OLDPWD".into(), world.terminal.cwd.clone());
    world.terminal.env.insert("PWD".into(), next.clone());
    world.terminal.cwd = next.clone();
    Ok(Output::success(if previous.is_some() {
        format!("{next}\n")
    } else {
        String::new()
    }))
}

fn read(world: &WorldState, file: &str, actor: &str) -> GameResult<String> {
    if file == "-" {
        return Err(domain(
            "interactive stdin is not implemented by this virtual terminal",
        ));
    }
    world
        .fs()?
        .read(&normalize(file, &world.terminal.cwd)?, actor)
}

fn cat(world: &WorldState, args: &[String], actor: &str) -> GameResult<Output> {
    let opts = options(
        "cat",
        args,
        "nbEsTu",
        "",
        &[
            ("number", 'n'),
            ("number-nonblank", 'b'),
            ("show-ends", 'E'),
            ("show-tabs", 'T'),
            ("squeeze-blank", 's'),
        ],
    )?;
    if opts.help {
        return Ok(Output::success(manual("cat").unwrap_or_default()));
    }
    if opts.files.is_empty() {
        return Err(domain(
            "cat: interactive stdin is not implemented; provide FILE operands",
        ));
    }
    let mut output = Output::default();
    let mut text = String::new();
    for file in &opts.files {
        match read(world, file, actor) {
            Ok(content) => {
                output_limit(text.len() + content.len())?;
                text.push_str(&content);
            }
            Err(error) => output.error("cat", file, error),
        }
    }
    let mut number = 1;
    let mut blank_before = false;
    for line in text.split_inclusive('\n') {
        let terminated = line.ends_with('\n');
        let body = if terminated {
            &line[..line.len() - 1]
        } else {
            line
        };
        let blank = body.is_empty();
        if opts.has('s') && blank && blank_before {
            continue;
        }
        blank_before = blank;
        if (opts.has('b') && !blank) || (opts.has('n') && !opts.has('b')) {
            output.stdout.push_str(&format!("{number:>6}\t"));
            number += 1;
        }
        let mut body = if opts.has('T') {
            body.replace('\t', "^I")
        } else {
            body.into()
        };
        if opts.has('E') && terminated && body.ends_with('\r') {
            body.pop();
            body.push_str("^M");
        }
        output.stdout.push_str(&body);
        if terminated {
            if opts.has('E') {
                output.stdout.push('$');
            }
            output.stdout.push('\n');
        }
        output_limit(output.stdout.len())?;
    }
    Ok(output)
}

fn slice(world: &WorldState, command: &str, args: &[String], actor: &str) -> GameResult<Output> {
    let opts = options(
        command,
        args,
        "qv",
        "nc",
        &[
            ("lines", 'n'),
            ("bytes", 'c'),
            ("quiet", 'q'),
            ("silent", 'q'),
            ("verbose", 'v'),
        ],
    )?;
    if opts.help {
        return Ok(Output::success(manual(command).unwrap_or_default()));
    }
    if opts.files.is_empty() {
        return Err(domain(format!(
            "{command}: interactive stdin is not implemented; provide FILE operands"
        )));
    }
    let (unit, count) = opts
        .counts
        .last()
        .map(|(unit, count)| (*unit, count.as_str()))
        .unwrap_or(('n', "10"));
    let digits = count.strip_prefix(['+', '-']).unwrap_or(count);
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return Err(domain(format!(
            "{command}: invalid count '{count}'; decimal counts only"
        )));
    }
    let amount = digits
        .parse::<usize>()
        .map_err(|_| domain(format!("{command}: count is too large")))?;
    let headers = opts
        .flags
        .last()
        .map_or(opts.files.len() > 1, |flag| *flag == 'v');
    let mut output = Output::default();
    let mut emitted_header = false;
    for file in &opts.files {
        let result = read(world, file, actor).and_then(|content| {
            let lines: Vec<&str> = content.split_inclusive('\n').collect();
            let length = if unit == 'n' {
                lines.len()
            } else {
                content.len()
            };
            let (start, end) = if command == "head" {
                (
                    0,
                    if count.starts_with('-') {
                        length.saturating_sub(amount)
                    } else {
                        amount.min(length)
                    },
                )
            } else if count.starts_with('+') {
                (amount.saturating_sub(1).min(length), length)
            } else {
                (length.saturating_sub(amount), length)
            };
            if unit == 'n' {
                Ok(lines[start..end].concat())
            } else {
                content.get(start..end).map(str::to_string).ok_or_else(|| {
                    domain(
                        "byte range splits UTF-8; raw byte output is not supported by the text VFS",
                    )
                })
            }
        });
        match result {
            Ok(text) => {
                output_limit(output.stdout.len() + text.len() + file.len() + 12)?;
                if headers {
                    if emitted_header {
                        output.stdout.push('\n');
                    }
                    output.stdout.push_str(&format!("==> {file} <==\n"));
                    emitted_header = true;
                }
                output.stdout.push_str(&text);
            }
            Err(error) => output.error(command, file, error),
        }
    }
    Ok(output)
}

fn transfer(
    world: &mut WorldState,
    command: &str,
    args: &[String],
    actor: &str,
) -> GameResult<Output> {
    let mut aliases = vec![("no-clobber", 'n'), ("verbose", 'v')];
    if command == "cp" {
        aliases.push(("recursive", 'r'));
    }
    let opts = options(
        command,
        args,
        if command == "cp" { "rRnv" } else { "nv" },
        "",
        &aliases,
    )?;
    if opts.help {
        return Ok(Output::success(manual(command).unwrap_or_default()));
    }
    if opts.files.len() != 2 {
        return Err(domain(format!(
            "{command}: this subset requires SOURCE and DESTINATION"
        )));
    }
    let source = normalize(&opts.files[0], &world.terminal.cwd)?;
    let destination = normalize(&opts.files[1], &world.terminal.cwd)?;
    let original = world.fs()?.stat(&source, actor)?.clone();
    if command == "cp" && original.kind == "directory" && !opts.has('r') && !opts.has('R') {
        return Err(domain("cp: omitting directory; use -r"));
    }
    let target = if world
        .fs()?
        .stat(&destination, actor)
        .is_ok_and(|node| node.kind == "directory")
    {
        format!("{}/{}", destination.trim_end_matches('/'), original.name)
    } else {
        destination.clone()
    };
    if source == target || target.starts_with(&format!("{source}/")) {
        return Err(domain(format!(
            "{command}: source and destination must be different and not nested"
        )));
    }
    let existing = world.fs()?.nodes.get(&target).cloned();
    if existing.is_some() {
        world.fs()?.stat(&target, actor)?;
    }
    if existing.is_some() && opts.has('n') {
        return Ok(Output::default());
    }
    if let Some(existing) = existing {
        if existing.kind != "file" || original.kind != "file" {
            return Err(domain(format!(
                "{command}: replacing or merging existing directories is not implemented"
            )));
        }
        world.fs()?.readable(&source, actor)?;
        if command == "cp" {
            if let Some(blob) = &original.blob {
                world.fs_mut()?.write_blob(&target, blob.clone(), actor)?;
            } else {
                world.fs_mut()?.write(&target, &original.content, actor)?;
            }
            if let Some(node) = world.fs_mut()?.nodes.get_mut(&target) {
                node.metadata = original.metadata;
            }
        } else {
            world.fs_mut()?.remove(&target, actor, false)?;
            world
                .fs_mut()?
                .transfer(&source, &target, actor, true, false)?;
        }
    } else {
        world.fs_mut()?.transfer(
            &source,
            &target,
            actor,
            command == "mv",
            opts.has('r') || opts.has('R'),
        )?;
    }
    Ok(Output::success(if opts.has('v') {
        let display_target = if target != destination {
            format!("{}/{}", opts.files[1].trim_end_matches('/'), original.name)
        } else {
            opts.files[1].clone()
        };
        format!(
            "{}'{}' -> '{}'\n",
            if command == "mv" { "renamed " } else { "" },
            opts.files[0],
            display_target
        )
    } else {
        String::new()
    }))
}

fn remove(world: &mut WorldState, args: &[String], actor: &str) -> GameResult<Output> {
    let opts = options(
        "rm",
        args,
        "rRfv",
        "",
        &[("recursive", 'r'), ("force", 'f'), ("verbose", 'v')],
    )?;
    if opts.help {
        return Ok(Output::success(manual("rm").unwrap_or_default()));
    }
    if opts.files.is_empty() && !opts.has('f') {
        return Err(domain("rm: missing operand"));
    }
    let mut output = String::new();
    for file in &opts.files {
        let path = normalize(file, &world.terminal.cwd)?;
        if crate::vfs::VirtualFileSystem::is_trash_container(&path) {
            return Err(domain("rm: virtual trash infrastructure is protected"));
        }
        // Check traversal even under -f, which must not turn permission errors into success.
        if opts.has('f') && !world.fs()?.path_exists(&path, actor)? {
            continue;
        }
        let node = world.fs()?.stat(&path, actor)?.clone();
        let recursive = opts.has('r') || opts.has('R');
        if node.kind == "directory" && !recursive {
            return Err(domain(format!(
                "rm: cannot remove '{file}': Is a directory"
            )));
        }
        let mut removed = if opts.has('v') {
            world
                .fs()?
                .nodes
                .values()
                .filter(|node| node.id == path || node.id.starts_with(&format!("{path}/")))
                .map(|node| (node.id.clone(), node.kind == "directory"))
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        world.fs_mut()?.remove(&path, actor, recursive)?;
        removed.sort_by(|a, b| {
            b.0.matches('/')
                .count()
                .cmp(&a.0.matches('/').count())
                .then_with(|| a.0.cmp(&b.0))
        });
        for (item, directory) in removed {
            output.push_str(&format!(
                "removed {}'{}{}'\n",
                if directory { "directory " } else { "" },
                file,
                &item[path.len()..]
            ));
        }
    }
    if world.fs()?.directory(&world.terminal.cwd, actor).is_err() {
        world.terminal.cwd = HOME.into();
        world.terminal.env.insert("PWD".into(), HOME.into());
    }
    Ok(Output::success(output))
}
