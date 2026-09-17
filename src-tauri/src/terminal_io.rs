//! Audited text/file command subset. All reads and writes use the selected VFS.
use crate::{
    error::GameResult,
    vfs::{domain, normalize, HOME},
    world::WorldState,
};

#[derive(Default, Debug)]
pub struct Output {
    pub byte_ordered: Vec<(u8, Vec<u8>)>,
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
    let next = world.fs()?.resolve(&next, actor, crate::vfs::Follow::Yes)?;
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
