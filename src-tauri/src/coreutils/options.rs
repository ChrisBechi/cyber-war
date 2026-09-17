use crate::{error::GameResult, terminal_io::Options, vfs::domain};

pub(super) fn parse(
    name: &str,
    args: &[String],
    switches: &str,
    values: &str,
    aliases: &[(&str, char)],
) -> GameResult<Options> {
    let mut args = args.to_vec();
    if name == "printenv" {
        if let Some(index) = args
            .iter()
            .position(|arg| !arg.starts_with('-') || arg == "-")
        {
            args.insert(index, "--".into());
        }
    }
    let mut ended = false;
    for arg in &mut args {
        if ended {
            continue;
        }
        if arg == "--" {
            ended = true;
            continue;
        }
        if let Some(long) = arg.strip_prefix("--") {
            let (prefix, value) = long
                .split_once('=')
                .map_or((long, None), |(p, v)| (p, Some(v)));
            if matches!(prefix, "help" | "version") {
                continue;
            }
            let matches: Vec<_> = aliases
                .iter()
                .filter(|(s, _)| s.starts_with(prefix))
                .collect();
            if aliases.iter().any(|(s, _)| *s == prefix) {
                continue;
            }
            if matches.len() == 1 {
                *arg = format!(
                    "--{}{}",
                    matches[0].0,
                    value.map(|v| format!("={v}")).unwrap_or_default()
                );
            } else if matches.len() > 1 {
                return Err(domain(format!("{name}: option '{arg}' is ambiguous\nTry '{name} --help' for more information.")));
            }
        }
    }
    crate::terminal_io::options(name, &args, switches, values, aliases)
}

pub(super) fn early(
    name: &str,
    args: &[String],
    syntax: &str,
) -> Option<crate::terminal_io::Output> {
    let values = match name {
        "basename" => "s",
        "env" => "u",
        "head" | "tail" => "nc",
        "base64" => "w",
        "mkdir" => "m",
        "stat" => "c",
        "cp" | "mv" => "t",
        "sort" => "o",
        "uniq" => "fsw",
        _ => "",
    };
    let mut skip = false;
    for arg in args {
        if skip {
            skip = false;
            continue;
        }
        if arg == "--" {
            break;
        }
        if matches!(name, "env" | "printenv") && (!arg.starts_with('-') || arg == "-") {
            break;
        }
        if arg.len() == 2
            && arg.starts_with('-')
            && arg.chars().nth(1).is_some_and(|c| values.contains(c))
        {
            skip = true;
            continue;
        }
        if matches!(
            arg.as_str(),
            "--suffix"
                | "--unset"
                | "--lines"
                | "--bytes"
                | "--wrap"
                | "--mode"
                | "--format"
                | "--output"
                | "--target-directory"
        ) {
            skip = true;
            continue;
        }
        if arg == "--help" {
            return Some(crate::terminal_io::Output::success(
                super::help(name).unwrap_or_else(|| format!("Usage: {name} {syntax}\n")),
            ));
        }
        if arg == "--version" {
            return Some(crate::terminal_io::Output::success(super::version(name)));
        }
    }
    None
}

pub(super) fn operand_error(name: &str, message: &str) -> crate::error::GameError {
    domain(format!(
        "{name}: {message}\nTry '{name} --help' for more information."
    ))
}
