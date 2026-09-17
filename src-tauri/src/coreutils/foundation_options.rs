//! Foundation GNU 9.7 option behavior captured in the isolated Linux reference.
//! basename/printenv stop at the first operand; dirname/whoami use GNU
//! permutation unless POSIXLY_CORRECT is present.
use crate::terminal_io::Output;

#[derive(Default)]
pub(super) struct Options {
    pub files: Vec<String>,
    pub multiple: bool,
    pub suffix: Option<String>,
    pub zero: bool,
    pub special: Option<&'static str>,
    pub flags: Vec<char>,
    pub selection: Option<super::head::Selection>,
}

pub(super) fn error(name: &str, invocation: &str, message: &str, option: bool) -> Output {
    Output {
        stderr: format!(
            "{}: {message}\nTry '{invocation} --help' for more information.\n",
            if option { invocation } else { name }
        ),
        status: if name == "printenv" { 2 } else { 1 },
        ..Default::default()
    }
}

// GNU C-locale diagnostics quote bytes; embedded quotes/control/non-ASCII bytes
// are escaped rather than interpreted as terminal control sequences.
pub(super) fn quote(value: &str) -> String {
    let mut out = String::from("'");
    for b in value.bytes() {
        match b {
            b'\n' => out.push_str("\\n"),
            b'\t' => out.push_str("\\t"),
            b'\r' => out.push_str("\\r"),
            7 => out.push_str("\\a"),
            8 => out.push_str("\\b"),
            11 => out.push_str("\\v"),
            12 => out.push_str("\\f"),
            b'\\' => out.push_str("\\\\"),
            b'\'' => out.push_str("\\'"),
            32..=126 => out.push(b as char),
            _ => out.push_str(&format!("\\{b:03o}")),
        }
    }
    out.push('\'');
    out
}

/// GNU shell-escape quoting for path diagnostics; Foundation operand errors use
/// the C quoting style above. The two styles deliberately have separate contracts.
pub(super) fn shell_quote(value: &str) -> String {
    if !value.is_empty()
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_+-./:=,@%".contains(&b))
    {
        return value.into();
    }
    shell_quote_always(value)
}

pub(super) fn shell_quote_always(value: &str) -> String {
    if value.contains('\'')
        && value
            .bytes()
            .all(|b| (32..127).contains(&b) && !b"\"$`\\".contains(&b))
    {
        return format!("\"{value}\"");
    }
    let mut out = String::from("'");
    let mut escaped = false;
    for b in value.bytes() {
        let needs_escape = !(32..127).contains(&b);
        if needs_escape != escaped {
            out.push_str(if needs_escape { "'$'" } else { "''" });
            escaped = needs_escape;
        }
        if needs_escape {
            // Reuse C-locale byte escapes from the Foundation quoter.
            match b {
                7 => out.push_str("\\a"),
                8 => out.push_str("\\b"),
                9 => out.push_str("\\t"),
                10 => out.push_str("\\n"),
                11 => out.push_str("\\v"),
                12 => out.push_str("\\f"),
                13 => out.push_str("\\r"),
                _ => out.push_str(&format!("\\{b:03o}")),
            }
        } else if b == b'\'' {
            out.push_str("'\\''");
        } else {
            out.push(b as char);
        }
    }
    out.push('\'');
    out
}

pub(super) fn parse(
    name: &str,
    invocation: &str,
    args: &[String],
    posix: bool,
) -> Result<Options, Box<Output>> {
    let mut out = Options::default();
    let mut long = Vec::new();
    match name {
        "basename" => long.extend([("multiple", 'a'), ("suffix", 's'), ("zero", 'z')]),
        "dirname" => long.push(("zero", 'z')),
        "printenv" => long.push(("null", '0')),
        "head" => long.extend([
            ("bytes", 'c'),
            ("lines", 'n'),
            ("quiet", 'q'),
            ("silent", 'q'),
            ("verbose", 'v'),
            ("zero-terminated", 'z'),
        ]),
        "cat" => long.extend([
            ("number-nonblank", 'b'),
            ("number", 'n'),
            ("squeeze-blank", 's'),
            ("show-nonprinting", 'v'),
            ("show-ends", 'E'),
            ("show-tabs", 'T'),
            ("show-all", 'A'),
        ]),
        _ => {}
    }
    long.extend([("help", 'h'), ("version", 'v')]);
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if arg == "--" {
            index += 1;
            break;
        }
        if arg == "-" || !arg.starts_with('-') {
            if posix || matches!(name, "basename" | "printenv") {
                break;
            }
            out.files.push(arg.clone());
            index += 1;
            continue;
        }
        let mut parsed = Vec::new();
        if let Some(value) = arg.strip_prefix("--") {
            let (prefix, attached) = value
                .split_once('=')
                .map_or((value, None), |(k, v)| (k, Some(v)));
            let matches: Vec<_> = long.iter().filter(|(n, _)| n.starts_with(prefix)).collect();
            let found = long
                .iter()
                .find(|(n, _)| *n == prefix)
                .or_else(|| (matches.len() == 1).then(|| matches[0]));
            let Some((_, ch)) = found else {
                if matches.len() > 1 {
                    let alternatives = matches
                        .iter()
                        .map(|(name, _)| format!(" '--{name}'"))
                        .collect::<String>();
                    return Err(Box::new(error(
                        name,
                        invocation,
                        &format!("option '{arg}' is ambiguous; possibilities:{alternatives}"),
                        true,
                    )));
                }
                return Err(Box::new(error(
                    name,
                    invocation,
                    &format!("unrecognized option '{}'", arg),
                    true,
                )));
            };
            let takes_value =
                (name == "basename" && *ch == 's') || (name == "head" && matches!(*ch, 'n' | 'c'));
            if !takes_value && attached.is_some() {
                return Err(Box::new(error(
                    name,
                    invocation,
                    &format!("option '--{prefix}' doesn't allow an argument"),
                    true,
                )));
            }
            if matches!(name, "cat" | "head")
                && matches!(*ch, 'h' | 'v')
                && matches!(found.unwrap().0, "help" | "version")
            {
                out.special = Some(if *ch == 'h' { "help" } else { "version" });
                return Ok(out);
            }
            let val = if takes_value {
                Some(if let Some(value) = attached {
                    value.to_string()
                } else {
                    index += 1;
                    args.get(index).cloned().ok_or_else(|| {
                        error(
                            name,
                            invocation,
                            &format!(
                                "option '--{}' requires an argument",
                                if name == "head" {
                                    found.unwrap().0
                                } else {
                                    prefix
                                }
                            ),
                            true,
                        )
                    })?
                })
            } else {
                None
            };
            parsed.push((*ch, val));
        } else {
            for (offset, ch) in arg[1..].char_indices() {
                let allowed = match name {
                    "basename" => "asz",
                    "dirname" => "z",
                    "printenv" => "0iu",
                    "cat" => "AbEnestTuv",
                    "head" => "cnqvz0123456789",
                    _ => "",
                };
                if !allowed.contains(ch) {
                    // getopt diagnoses the first invalid argv byte even when it
                    // belongs to a UTF-8 sequence. Keep bytes separate from display.
                    let mut bytes = format!("{invocation}: invalid option -- '").into_bytes();
                    bytes.push(arg.as_bytes()[1 + offset]);
                    bytes.extend_from_slice(
                        format!("'\nTry '{invocation} --help' for more information.\n").as_bytes(),
                    );
                    return Err(Box::new(Output {
                        stderr: String::from_utf8_lossy(&bytes).into_owned(),
                        byte_ordered: vec![(2, bytes)],
                        status: if name == "printenv" { 2 } else { 1 },
                        ..Default::default()
                    }));
                }
                if name == "head" && ch.is_ascii_digit() {
                    return Err(Box::new(error(
                        name,
                        invocation,
                        &format!("invalid trailing option -- {ch}"),
                        false,
                    )));
                }
                if (name == "basename" && ch == 's')
                    || (name == "printenv" && ch == 'u')
                    || (name == "head" && matches!(ch, 'n' | 'c'))
                {
                    let rest = &arg[1 + offset + ch.len_utf8()..];
                    let value = if rest.is_empty() {
                        index += 1;
                        args.get(index).cloned().ok_or_else(|| {
                            error(
                                name,
                                invocation,
                                &format!("option requires an argument -- '{ch}'"),
                                true,
                            )
                        })?
                    } else {
                        rest.into()
                    };
                    parsed.push((ch, Some(value)));
                    break;
                }
                if ch == 'i' {
                    return Err(Box::new(Output {
                        stderr: format!("Try '{invocation} --help' for more information.\n"),
                        status: 2,
                        ..Default::default()
                    }));
                }
                parsed.push((ch, None));
            }
        }
        for (ch, value) in parsed {
            if name == "head" {
                if let Some(value) = value {
                    out.selection = Some(super::head::Selection::parse(ch, &value)?);
                } else {
                    out.flags.push(ch);
                }
                continue;
            }
            if name == "cat" {
                out.flags.push(ch);
                continue;
            }
            match ch {
                'a' => out.multiple = true,
                's' => {
                    out.multiple = true;
                    out.suffix = value;
                }
                'z' | '0' => out.zero = true,
                'h' | 'v' => {
                    out.special = Some(if ch == 'h' { "help" } else { "version" });
                    return Ok(out);
                }
                _ => {
                    return Err(Box::new(Output {
                        stderr: format!("Try '{invocation} --help' for more information.\n"),
                        status: 2,
                        ..Default::default()
                    }))
                }
            }
        }
        index += 1;
    }
    out.files.extend_from_slice(&args[index..]);
    Ok(out)
}
