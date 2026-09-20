use crate::vfs::domain;

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
