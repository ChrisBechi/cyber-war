use super::{
    io,
    options::{early, operand_error, parse},
};
use crate::{
    error::GameResult,
    terminal_io::{Options, Output},
    vfs::domain,
    world::WorldState,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use sha2::{Digest, Sha256};

pub(super) fn execute(
    world: &mut WorldState,
    name: &str,
    args: &[String],
    actor: &str,
) -> GameResult<Output> {
    let syntax = match name {
        "cat" => "[-AbEnestTuv] [FILE]...",
        "head" | "tail" => "[-qv] [-n COUNT | -c COUNT] [FILE]... (decimal counts; no follow mode)",
        "tee" => "[-a] [FILE]...",
        "base64" => "[-di] [-w COLS] [FILE]",
        _ => "[-bt] [FILE]... | [-c] [--status | --quiet] [CHECKSUM_FILE]...",
    };
    if let Some(out) = early(name, args, syntax) {
        return Ok(out);
    }
    let mut opts = match name {
        "cat" => parse(
            name,
            args,
            "AbEnestTuv",
            "",
            &[
                ("show-all", 'A'),
                ("number-nonblank", 'b'),
                ("show-ends", 'E'),
                ("number", 'n'),
                ("squeeze-blank", 's'),
                ("show-tabs", 'T'),
                ("show-nonprinting", 'v'),
            ],
        )?,
        "head" | "tail" => parse(
            name,
            args,
            "qv",
            "nc",
            &[
                ("quiet", 'q'),
                ("silent", 'q'),
                ("verbose", 'v'),
                ("lines", 'n'),
                ("bytes", 'c'),
            ],
        )?,
        "tee" => parse(name, args, "a", "", &[("append", 'a')])?,
        "base64" => parse(
            name,
            args,
            "di",
            "w",
            &[("decode", 'd'), ("ignore-garbage", 'i'), ("wrap", 'w')],
        )?,
        _ => parse(
            name,
            args,
            "bct",
            "",
            &[
                ("binary", 'b'),
                ("text", 't'),
                ("check", 'c'),
                ("quiet", 'q'),
                ("status", 's'),
            ],
        )?,
    };
    let mut input = Some(io::stdin(world));
    if name == "tee" {
        let data = input.take().unwrap_or_default();
        let mut out = Output::default();
        io::send(&mut out, 1, data.clone())?;
        for file in &opts.files {
            if let Err(error) = io::write(world, file, &data, actor, opts.has('a')) {
                diagnostic(&mut out, name, file, error)?;
            }
        }
        return Ok(out);
    }
    if name == "base64" && opts.files.len() > 1 {
        return Err(operand_error(
            name,
            &format!("extra operand '{}'", opts.files[1]),
        ));
    }
    if opts.files.is_empty() {
        opts.files.push("-".into());
    }
    let mut out = Output::default();
    let mut cat = Cat::default();
    let mut header_seen = false;
    let selection = if matches!(name, "head" | "tail") {
        Some(selection(name, &opts)?)
    } else {
        None
    };
    let wrap = if name == "base64" {
        opts.counts
            .last()
            .map(|(_, s)| {
                s.parse::<usize>()
                    .map_err(|_| operand_error(name, &format!("invalid wrap size: '{s}'")))
            })
            .transpose()?
            .unwrap_or(76)
    } else {
        0
    };
    for file in &opts.files {
        if crate::shell::control::cancelled() {
            out.status = 130;
            break;
        }
        let data = match io::read(world, file, actor, &mut input) {
            Ok(data) => data,
            Err(error) => {
                diagnostic(&mut out, name, file, error)?;
                continue;
            }
        };
        let bytes = match name {
            "cat" => cat.transform(&data, &opts),
            "head" | "tail" => {
                let headers = opts
                    .flags
                    .iter()
                    .rev()
                    .find(|c| **c == 'q' || **c == 'v')
                    .map_or(opts.files.len() > 1, |c| *c == 'v');
                if headers {
                    let title = if file == "-" { "standard input" } else { file };
                    io::send(
                        &mut out,
                        1,
                        format!("{}==> {title} <==\n", if header_seen { "\n" } else { "" })
                            .into_bytes(),
                    )?;
                    header_seen = true;
                }
                slice(&data, name, selection.as_ref().unwrap())
            }
            "base64" if opts.has('d') => {
                let encoded: Vec<u8> = data
                    .into_iter()
                    .filter(|b| {
                        *b != b'\n'
                            && (!opts.has('i') || b.is_ascii_alphanumeric() || b"+/=".contains(b))
                    })
                    .collect();
                match STANDARD.decode(&encoded) {
                    Ok(bytes) => bytes,
                    Err(_) => {
                        io::send(&mut out, 2, b"base64: invalid input\n".to_vec())?;
                        out.status = 1;
                        Vec::new()
                    }
                }
            }
            "base64" => {
                let encoded = STANDARD.encode(data);
                if wrap == 0 {
                    encoded.into_bytes()
                } else {
                    encoded
                        .as_bytes()
                        .chunks(wrap)
                        .flat_map(|c| c.iter().copied().chain(*b"\n"))
                        .collect()
                }
            }
            "sha256sum" if opts.has('c') => {
                check(world, &data, actor, &opts, &mut out)?;
                Vec::new()
            }
            "sha256sum" => {
                let escaped = file.contains(['\\', '\n', '\r']);
                let label = file
                    .replace('\\', "\\\\")
                    .replace('\n', "\\n")
                    .replace('\r', "\\r");
                format!(
                    "{}{:x} {}{label}\n",
                    if escaped { "\\" } else { "" },
                    Sha256::digest(data),
                    if opts.has('b') { "*" } else { " " }
                )
                .into_bytes()
            }
            _ => unreachable!(),
        };
        io::send(&mut out, 1, bytes)?;
    }
    Ok(out)
}
fn diagnostic(
    out: &mut Output,
    name: &str,
    file: &str,
    error: impl std::fmt::Display,
) -> GameResult<()> {
    out.status = 1;
    io::send(
        out,
        2,
        format!(
            "{name}: {file}: {}\n",
            crate::terminal_io::error_reason(error)
        )
        .into_bytes(),
    )
}
#[derive(Default)]
struct Cat {
    number: usize,
    line_start: bool,
    initialized: bool,
    blank_before: bool,
}
impl Cat {
    fn transform(&mut self, data: &[u8], opts: &Options) -> Vec<u8> {
        if !self.initialized {
            self.number = 1;
            self.line_start = true;
            self.initialized = true;
        }
        let mut out = Vec::new();
        let visible = opts.has('v') || opts.has('A') || opts.has('e') || opts.has('t');
        let tabs = opts.has('T') || opts.has('A') || opts.has('t');
        let ends = opts.has('E') || opts.has('A') || opts.has('e');
        for (index, &b) in data.iter().enumerate() {
            let blank = self.line_start && b == b'\n';
            if blank && self.blank_before && opts.has('s') {
                continue;
            }
            if self.line_start {
                if (opts.has('b') && !blank) || (opts.has('n') && !opts.has('b')) {
                    out.extend_from_slice(format!("{:>6}\t", self.number).as_bytes());
                    self.number += 1;
                }
                self.blank_before = blank;
            }
            if b == b'\n' {
                if ends {
                    out.push(b'$');
                }
                out.push(b'\n');
            } else if b == b'\t' {
                if tabs {
                    out.extend_from_slice(b"^I");
                } else {
                    out.push(b);
                }
            } else if ends && b == b'\r' && data.get(index + 1) == Some(&b'\n') {
                out.extend_from_slice(b"^M");
            } else if visible {
                visible_byte(b, &mut out);
            } else {
                out.push(b);
            }
            self.line_start = b == b'\n';
        }
        out
    }
}
fn visible_byte(mut b: u8, out: &mut Vec<u8>) {
    if b >= 128 {
        out.extend_from_slice(b"M-");
        b -= 128;
    }
    if b < 32 {
        out.extend_from_slice(&[b'^', b + 64]);
    } else if b == 127 {
        out.extend_from_slice(b"^?");
    } else {
        out.push(b);
    }
}
struct Selection {
    bytes: bool,
    count: usize,
    sign: char,
}
fn selection(name: &str, opts: &Options) -> GameResult<Selection> {
    let (flag, value) = opts
        .counts
        .last()
        .map(|(f, s)| (*f, s.as_str()))
        .unwrap_or(('n', "10"));
    let sign = value
        .chars()
        .next()
        .filter(|c| *c == '-' || *c == '+')
        .unwrap_or(' ');
    let digits = value.trim_start_matches(['-', '+']);
    if digits.is_empty() || !digits.bytes().all(|c| c.is_ascii_digit()) {
        return Err(operand_error(
            name,
            &format!(
                "invalid {}: '{value}'",
                if flag == 'c' {
                    "number of bytes"
                } else {
                    "number of lines"
                }
            ),
        ));
    }
    let count = digits.parse().map_err(|_| {
        operand_error(
            name,
            &format!("invalid count: '{value}': Value too large for defined data type"),
        )
    })?;
    Ok(Selection {
        bytes: flag == 'c',
        count,
        sign,
    })
}
fn slice(data: &[u8], name: &str, s: &Selection) -> Vec<u8> {
    if s.bytes {
        let cut = if name == "head" {
            if s.sign == '-' {
                data.len().saturating_sub(s.count)
            } else {
                s.count.min(data.len())
            }
        } else if s.sign == '+' {
            s.count.saturating_sub(1).min(data.len())
        } else {
            data.len().saturating_sub(s.count)
        };
        return if name == "head" {
            data[..cut].to_vec()
        } else {
            data[cut..].to_vec()
        };
    }
    let lines: Vec<&[u8]> = data.split_inclusive(|b| *b == b'\n').collect();
    let cut = if name == "head" {
        if s.sign == '-' {
            lines.len().saturating_sub(s.count)
        } else {
            s.count.min(lines.len())
        }
    } else if s.sign == '+' {
        s.count.saturating_sub(1).min(lines.len())
    } else {
        lines.len().saturating_sub(s.count)
    };
    if name == "head" {
        lines[..cut].concat()
    } else {
        lines[cut..].concat()
    }
}
fn check(
    world: &mut WorldState,
    data: &[u8],
    actor: &str,
    opts: &Options,
    out: &mut Output,
) -> GameResult<()> {
    let text =
        std::str::from_utf8(data).map_err(|_| domain("sha256sum: checksum list must be UTF-8"))?;
    let mut valid = 0;
    for line in text.lines() {
        let bytes = line.as_bytes();
        if bytes.len() < 67
            || !bytes[..64].iter().all(u8::is_ascii_hexdigit)
            || bytes[64] != b' '
            || !b" *".contains(&bytes[65])
        {
            out.status = 1;
            continue;
        }
        valid += 1;
        let path = &line[66..];
        let mut input = None;
        let matched = io::read(world, path, actor, &mut input)
            .map(|b| format!("{:x}", Sha256::digest(b)).eq_ignore_ascii_case(&line[..64]));
        let ok = matched.as_ref().is_ok_and(|ok| *ok);
        if !ok {
            out.status = 1;
        }
        if !opts.has('s') {
            if let Err(e) = matched {
                diagnostic(out, "sha256sum", path, e)?;
            }
            if !ok || !opts.has('q') {
                io::send(
                    out,
                    1,
                    format!("{path}: {}\n", if ok { "OK" } else { "FAILED" }).into_bytes(),
                )?;
            }
        }
    }
    if valid == 0 {
        out.status = 1;
        if !opts.has('s') {
            io::send(
                out,
                2,
                b"sha256sum: no properly formatted checksum lines found\n".to_vec(),
            )?;
        }
    }
    Ok(())
}
