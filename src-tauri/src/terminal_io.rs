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
    /// Ordered writes used when a shell duplicates stdout/stderr or runs scripts.
    pub ordered: Vec<(u8, String)>,
    pub archive_job: Option<u32>,
    pub binary: Option<Vec<u8>>,
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
            .push_str(&format!("{command}: {file}: {}\n", error_reason(error)));
        self.status = 1;
    }
}

pub(crate) fn error_reason(error: impl std::fmt::Display) -> String {
    let error = error.to_string();
    for (ending, message) in [
        ("no such file or directory", "No such file or directory"),
        ("parent does not exist", "No such file or directory"),
        ("permission denied", "Permission denied"),
        ("is a directory", "Is a directory"),
        ("not a directory", "Not a directory"),
        (
            "not a directory (symlink traversal is not allowed)",
            "Not a directory",
        ),
    ] {
        if error == ending || error.ends_with(&format!(": {ending}")) {
            return message.into();
        }
    }
    error
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
            if let Some((_, flag)) = aliases
                .iter()
                .find(|(alias, _)| alias.contains('=') && *alias == long)
            {
                parsed.flags.push(*flag);
                index += 1;
                continue;
            }
            let (name, attached) = long
                .split_once('=')
                .map_or((long, None), |(n, v)| (n, Some(v)));
            let flag = aliases
                .iter()
                .find(|(alias, _)| *alias == name)
                .map(|(_, flag)| *flag)
                .ok_or_else(|| {
                    domain(format!(
                        "{command}: unrecognized option '{arg}'\nTry '{command} --help' for more information."
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
                        "{command}: invalid option -- '{flag}'\nTry '{command} --help' for more information."
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
        "cp" => "cp [-rRnvfuTP] [-t DIRECTORY] [--recursive] [--no-clobber] [--verbose] [--force] [--no-dereference] [--target-directory=DIRECTORY] [--no-target-directory] [--version] [--] SOURCE... DESTINATION",
        "mv" => "mv [-nvfuT] [-t DIRECTORY] [--no-clobber] [--verbose] [--force] [--target-directory=DIRECTORY] [--no-target-directory] [--version] [--] SOURCE... DESTINATION",
        "rm" => "rm [-rRfvd] [--recursive] [--force] [--verbose] [--dir] [--version] [--] PATH...",
        _ => return None,
    };
    let detail = match command {
        "pwd" | "cd" => "-L/-P are equivalent: this VFS has no symbolic links. cd updates PWD/OLDPWD; cd - prints the previous directory. CDPATH is not implemented.",
        "cat" => "-b overrides -n. Numbering continues across files. Newlines and tabs are preserved unless explicitly displayed/transformed. -u is accepted with no buffering effect. -v/-e/-t/-A are not implemented.",
        "head" | "tail" => "Default: 10 lines. -q/--quiet/--silent suppress headers; -v/--verbose always shows headers. Decimal counts only. -n and -c accept attached or separate counts; the last one wins. Legacy -N is accepted only first. No follow mode, suffix multipliers or zero delimiters. Byte slices splitting UTF-8 cannot be represented by this text VFS and return an explicit error.",
        "cp" | "mv" => "Multiple sources require a destination directory. -t selects it explicitly; -T treats the destination as an exact path. cp -r/-R merges directories; mv replaces only empty destination directories. -n skips existing targets; -u skips targets at least as new; -v reports changes. cp -f retries unwritable destinations; mv uses the last -f/-n. Successful operands survive other operand failures. mv preserves virtual metadata and needs parent permissions, not file read access. cp preserves binary/media content, applies virtual umask 022, and creates fresh timestamps. cp -P and recursive copies preserve links; non-recursive cp follows final source links only. Interactive input, metadata preservation flags, backup flags, symlinked ancestors/destinations and full version banners remain unsupported. Recursion is bounded to 256 levels. A failed force retry is atomic per file.",
        "rm" => "Permanent VFS deletion, never GUI trash. -f ignores missing files but not permissions. -r/-R traverses directories without following symbolic links; -d removes empty directories. Each entry checks its parent permissions. Successful removals survive errors on other entries. Virtual root, dot/dot-dot and trash infrastructure are protected. Recursion is bounded to 256 levels. Interactive prompts, sticky bits and full version banners remain unsupported. Removing the current directory resets the game session to its virtual home.",
        _ => "",
    };
    Some(format!("{command}(1) â€” CYBER WAR virtual subset\n\nSYNOPSIS\n  {syntax}\n\n{detail}\n\nOnly the selected local/SSH virtual filesystem is used.\nText stdin from pipes and input redirections is supported; interactive stdin is not implemented. Unsupported options return an error.\n"))
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
        "cp" | "mv" => Some(crate::terminal_transfer::execute(
            world, command, args, actor,
        )),
        "rm" => Some(crate::terminal_remove::execute(world, args, actor)),
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
        return world.terminal.stdin.clone().ok_or_else(|| {
            domain("interactive stdin is not implemented by this virtual terminal")
        });
    }
    world
        .fs()?
        .read(&normalize(file, &world.terminal.cwd)?, actor)
}

fn cat(world: &WorldState, args: &[String], actor: &str) -> GameResult<Output> {
    let mut opts = options(
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
    if opts.files.is_empty() && world.terminal.stdin.is_some() {
        opts.files.push("-".into());
    }
    if opts.files.is_empty() {
        return Err(domain(
            "cat: interactive stdin is not implemented; provide FILE operands",
        ));
    }
    let mut output = Output::default();
    let mut stdin_consumed = false;
    let mut number = 1;
    let mut blank_before = false;
    let mut line_start = true;
    for file in &opts.files {
        if file == "-" {
            if stdin_consumed {
                continue;
            }
            stdin_consumed = true;
        }
        let text = match read(world, file, actor) {
            Ok(text) => text,
            Err(error) => {
                let offset = output.stderr.len();
                output.error("cat", file, error);
                output.ordered.push((2, output.stderr[offset..].to_owned()));
                continue;
            }
        };
        let offset = output.stdout.len();
        for line in text.split_inclusive('\n') {
            let terminated = line.ends_with('\n');
            let body = if terminated {
                &line[..line.len() - 1]
            } else {
                line
            };
            let blank = line_start && body.is_empty();
            if opts.has('s') && blank && blank_before {
                continue;
            }
            blank_before = blank;
            if line_start && ((opts.has('b') && !blank) || (opts.has('n') && !opts.has('b'))) {
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
            line_start = terminated;
            output_limit(output.stdout.len())?;
        }
        if output.stdout.len() > offset {
            output.ordered.push((1, output.stdout[offset..].to_owned()));
        }
    }
    Ok(output)
}

fn slice(world: &WorldState, command: &str, args: &[String], actor: &str) -> GameResult<Output> {
    let mut opts = options(
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
    if opts.files.is_empty() && world.terminal.stdin.is_some() {
        opts.files.push("-".into());
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
    let mut stdin_consumed = false;
    for file in &opts.files {
        if file == "-" {
            if stdin_consumed {
                continue;
            }
            stdin_consumed = true;
        }
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
                let offset = output.stdout.len();
                output_limit(output.stdout.len() + text.len() + file.len() + 12)?;
                if headers {
                    if emitted_header {
                        output.stdout.push('\n');
                    }
                    output.stdout.push_str(&format!(
                        "==> {} <==\n",
                        if file == "-" { "standard input" } else { file }
                    ));
                    emitted_header = true;
                }
                output.stdout.push_str(&text);
                output.ordered.push((1, output.stdout[offset..].to_owned()));
            }
            Err(error) => {
                let offset = output.stderr.len();
                output.error(command, file, error);
                output.ordered.push((2, output.stderr[offset..].to_owned()));
            }
        }
    }
    Ok(output)
}
