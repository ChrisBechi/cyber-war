//! Bounded text streams for the virtual coreutils profile (C / C.UTF-8).
use crate::{
    error::GameResult,
    terminal_io::{error_reason, options, Options, Output},
    vfs::{domain, normalize},
    world::WorldState,
};
use std::cmp::Ordering;

const LIMIT: usize = 4 * 1024 * 1024;
const RECORD_LIMIT: usize = 65_536;

pub(crate) fn manual(command: &str) -> Option<String> {
    let syntax = match command {
        "wc" => "wc [-lwmc] [--lines] [--words] [--chars] [--bytes] [--total=auto|always|only|never] [--] [FILE...]",
        "sort" => "sort [-bnfruscCz] [-o FILE] [--numeric-sort] [--reverse] [--unique] [--stable] [--ignore-case] [--ignore-leading-blanks] [--check[=diagnose-first|quiet|silent]] [--output=FILE] [--zero-terminated] [--] [FILE...]",
        "uniq" => "uniq [-cduiz] [-f N] [-s N] [-w N] [--count] [--repeated] [--unique] [--ignore-case] [--skip-fields=N] [--skip-chars=N] [--check-chars=N] [--zero-terminated] [--] [INPUT [OUTPUT]]",
        _ => return None,
    };
    let detail = match command {
        "wc" => "Counts newlines, words, characters and bytes in that fixed order. Defaults to lines/words/bytes. Multiple operands produce totals; - consumes stdin once. Errors preserve successful counts. C uses byte characters and ASCII whitespace; C.UTF-8 uses Unicode characters/whitespace. Display-width (-L), binary input and files0-from are unsupported.",
        "sort" => "Combines all inputs; supplies a missing final delimiter. Numeric keys use exact decimal prefixes, not floating point. -s disables the final full-line comparison; -u keeps the first equal key. -c/-C return 1 for disorder, 2 for errors. -o reads inputs before writing, including the same file. Byte collation for C/C.UTF-8 only. Field keys, merge, other numeric modes and external sorting are unsupported.",
        _ => "Counts only adjacent groups; -d keeps repeated groups, -u keeps single groups. Field skips happen before character skips; comparison width limits the remaining key. Case comparison is ASCII for C and Unicode lowercase for C.UTF-8. In this text subset -s/-w operate on Unicode characters; arbitrary byte slicing, locale-specific folding and group separator flags are unsupported. OUTPUT is truncated when opened.",
    };
    Some(format!("{command}(1) — CYBER WAR virtual subset\n\nSYNOPSIS\n  {syntax}\n\n{detail}\n\nNo file or '-' reads virtual stdin. Interactive stdin is not implemented.\nInput and output are bounded to 4 MiB; files remain subject to VFS limits.\n--help and --version are supported; the version banner contains the baseline line only.\n"))
}

fn send(output: &mut Output, fd: u8, text: String) {
    if fd == 1 {
        output.stdout.push_str(&text);
    } else {
        output.stderr.push_str(&text);
    }
    if !text.is_empty() {
        output.ordered.push((fd, text));
    }
}

fn read(
    world: &WorldState,
    file: &str,
    actor: &str,
    stdin: &mut Option<String>,
) -> GameResult<String> {
    if file == "-" {
        if world.terminal.stdin.is_none() && world.terminal.io.stdin_tty {
            return Err(domain("interactive stdin is not implemented"));
        }
        Ok(stdin.take().unwrap_or_default())
    } else {
        world
            .fs()?
            .read(&normalize(file, &world.terminal.cwd)?, actor)
    }
}

fn inputs(opts: &Options) -> Vec<String> {
    if opts.files.is_empty() {
        vec!["-".into()]
    } else {
        opts.files.clone()
    }
}

fn locale(world: &WorldState, category: &str) -> GameResult<bool> {
    let env = crate::terminal::virtual_env(world, &world.terminal.user);
    let name = env
        .get("LC_ALL")
        .filter(|s| !s.is_empty())
        .or_else(|| env.get(category).filter(|s| !s.is_empty()))
        .or_else(|| env.get("LANG"))
        .map(String::as_str)
        .unwrap_or("C.UTF-8");
    match name {
        "C" | "POSIX" => Ok(true),
        "C.UTF-8" | "C.utf8" => Ok(false),
        _ => Err(domain(format!(
            "unsupported virtual locale '{name}'; use C or C.UTF-8"
        ))),
    }
}

fn bounded(size: usize) -> GameResult<()> {
    if size > LIMIT {
        Err(domain("virtual terminal stream limit: 4 MiB"))
    } else {
        Ok(())
    }
}

pub(crate) fn execute(
    world: &mut WorldState,
    command: &str,
    args: &[String],
    actor: &str,
) -> GameResult<Output> {
    let result = match command {
        "wc" => wc(world, args, actor),
        "sort" => sort(world, args, actor),
        "uniq" => uniq(world, args, actor),
        _ => unreachable!(),
    };
    match result {
        Ok(output) => Ok(output),
        Err(error) => Ok(Output {
            stderr: format!("{}\n", error_reason(error)),
            status: if command == "sort" { 2 } else { 1 },
            ..Output::default()
        }),
    }
}

fn early(command: &str, opts: &Options) -> Option<Output> {
    if opts.help {
        Some(Output::success(manual(command).unwrap_or_default()))
    } else if opts.has('V') {
        Some(Output::success(format!("{command} (GNU coreutils) 9.7\n")))
    } else {
        None
    }
}

fn wc(world: &WorldState, args: &[String], actor: &str) -> GameResult<Output> {
    let opts = options(
        "wc",
        args,
        "lwmc",
        "\u{1}",
        &[
            ("lines", 'l'),
            ("words", 'w'),
            ("chars", 'm'),
            ("bytes", 'c'),
            ("total", '\u{1}'),
            ("version", 'V'),
        ],
    )?;
    if let Some(output) = early("wc", &opts) {
        return Ok(output);
    }
    let total = opts
        .counts
        .last()
        .map(|(_, value)| value.as_str())
        .unwrap_or("auto");
    if !["auto", "always", "only", "never"].contains(&total) {
        return Err(domain(format!(
            "wc: invalid argument '{total}' for 'total type'"
        )));
    }
    let c_locale = locale(world, "LC_CTYPE")?;
    let files = inputs(&opts);
    let selected: Vec<usize> = "lwmc"
        .chars()
        .enumerate()
        .filter(|(_, f)| opts.has(*f) || (opts.flags.is_empty() && *f != 'm'))
        .map(|(i, _)| i)
        .collect();
    let mut stdin = world.terminal.stdin.clone();
    let mut rows = Vec::new();
    let mut sums = [0usize; 4];
    let mut size = 0;
    for file in &files {
        match read(world, file, actor, &mut stdin) {
            Ok(text) => {
                size += text.len();
                bounded(size)?;
                let blank = |c: char| {
                    if c_locale {
                        c.is_ascii_whitespace()
                    } else {
                        c.is_whitespace()
                            || (!world.terminal.env.contains_key("POSIXLY_CORRECT")
                                && c == '\u{2060}')
                    }
                };
                let counts = [
                    text.bytes().filter(|b| *b == b'\n').count(),
                    text.split(blank).filter(|word| !word.is_empty()).count(),
                    if c_locale {
                        text.len()
                    } else {
                        text.chars().count()
                    },
                    text.len(),
                ];
                for (sum, count) in sums.iter_mut().zip(counts) {
                    *sum += count;
                }
                rows.push((file.clone(), Ok(counts)));
            }
            Err(error) => rows.push((file.clone(), Err(error))),
        }
    }
    let width = if total == "only" || (files.len() == 1 && selected.len() == 1) {
        1
    } else if files.iter().any(|f| f == "-") {
        7
    } else {
        sums[3].to_string().len()
    };
    let format_counts = |counts: &[usize; 4], label: &str| {
        let values = selected
            .iter()
            .map(|i| format!("{:>width$}", counts[*i]))
            .collect::<Vec<_>>()
            .join(" ");
        format!(
            "{values}{}\n",
            if label.is_empty() {
                String::new()
            } else {
                format!(" {label}")
            }
        )
    };
    let mut output = Output::default();
    for (file, counts) in rows {
        match counts {
            Ok(counts) if total != "only" => send(
                &mut output,
                1,
                format_counts(&counts, if opts.files.is_empty() { "" } else { &file }),
            ),
            Ok(_) => {}
            Err(error) => {
                output.status = 1;
                send(
                    &mut output,
                    2,
                    format!("wc: {file}: {}\n", error_reason(error)),
                );
            }
        }
    }
    if total == "always" || total == "only" || (total == "auto" && files.len() > 1) {
        send(
            &mut output,
            1,
            format_counts(&sums, if total == "only" { "" } else { "total" }),
        );
    }
    Ok(output)
}

// Compare arbitrarily long decimal prefixes without precision loss. The C
// numeric format allows blanks, a minus sign and a dot (not exponent/plus).
fn numeric(left: &str, right: &str) -> Ordering {
    fn parts(value: &str) -> (bool, &str, &str) {
        let value = value.trim_start_matches([' ', '\t']);
        let negative = value.starts_with('-');
        let value = value.strip_prefix('-').unwrap_or(value);
        let end = value.bytes().take_while(u8::is_ascii_digit).count();
        let integer = value[..end].trim_start_matches('0');
        let fraction = if value.as_bytes().get(end) == Some(&b'.') {
            let rest = &value[end + 1..];
            &rest[..rest.bytes().take_while(u8::is_ascii_digit).count()]
        } else {
            ""
        }
        .trim_end_matches('0');
        (
            negative && !(integer.is_empty() && fraction.is_empty()),
            integer,
            fraction,
        )
    }
    let (an, ai, af) = parts(left);
    let (bn, bi, bf) = parts(right);
    if an != bn {
        return if an {
            Ordering::Less
        } else {
            Ordering::Greater
        };
    }
    let order = ai
        .len()
        .cmp(&bi.len())
        .then_with(|| ai.cmp(bi))
        .then_with(|| {
            af.bytes()
                .chain(std::iter::repeat(b'0'))
                .zip(bf.bytes().chain(std::iter::repeat(b'0')))
                .take(af.len().max(bf.len()))
                .find_map(|(a, b)| (a != b).then(|| a.cmp(&b)))
                .unwrap_or(Ordering::Equal)
        });
    if an {
        order.reverse()
    } else {
        order
    }
}

fn sort(world: &mut WorldState, args: &[String], actor: &str) -> GameResult<Output> {
    let opts = options(
        "sort",
        args,
        "bnfruscCz",
        "o",
        &[
            ("ignore-leading-blanks", 'b'),
            ("numeric-sort", 'n'),
            ("ignore-case", 'f'),
            ("reverse", 'r'),
            ("unique", 'u'),
            ("stable", 's'),
            ("check", 'c'),
            ("check=quiet", 'C'),
            ("check=silent", 'C'),
            ("check=diagnose-first", 'c'),
            ("zero-terminated", 'z'),
            ("output", 'o'),
            ("version", 'V'),
        ],
    )?;
    if let Some(output) = early("sort", &opts) {
        return Ok(output);
    }
    let c_locale = locale(world, "LC_CTYPE")?;
    locale(world, "LC_COLLATE")?;
    if opts.has('n') {
        locale(world, "LC_NUMERIC")?;
    }
    let checking = opts.has('c') || opts.has('C');
    let files = inputs(&opts);
    if checking && (files.len() > 1 || !opts.counts.is_empty()) {
        return Err(domain(
            "sort: checking requires one input and no output file",
        ));
    }
    let delimiter = if opts.has('z') { '\0' } else { '\n' };
    let mut stdin = world.terminal.stdin.clone();
    let mut lines = Vec::new();
    let mut size = 0;
    for file in &files {
        let text = read(world, file, actor, &mut stdin).map_err(|error| {
            domain(format!(
                "sort: cannot read: {file}: {}",
                error_reason(error)
            ))
        })?;
        size += text.len();
        bounded(size)?;
        for line in text.split_terminator(delimiter) {
            if lines.len() >= RECORD_LIMIT {
                return Err(domain("sort: virtual record limit: 65536"));
            }
            lines.push(line.to_owned());
        }
    }
    let key = |line: &str| {
        let line = if opts.has('b') {
            line.trim_start_matches([' ', '\t'])
        } else {
            line
        };
        if opts.has('f') {
            if c_locale {
                line.to_ascii_uppercase()
            } else {
                line.to_uppercase()
            }
        } else {
            line.to_owned()
        }
    };
    let primary = |a: &str, b: &str| {
        if opts.has('n') {
            numeric(a, b)
        } else if !opts.has('b') && !opts.has('f') {
            a.cmp(b)
        } else {
            key(a).cmp(&key(b))
        }
    };
    let compare = |a: &str, b: &str| {
        let order = primary(a, b);
        let order = if !opts.has('s') && !opts.has('u') {
            order.then_with(|| a.cmp(b))
        } else {
            order
        };
        if opts.has('r') {
            order.reverse()
        } else {
            order
        }
    };
    if checking {
        for (index, pair) in lines.windows(2).enumerate() {
            if compare(&pair[0], &pair[1]) == Ordering::Greater
                || (opts.has('u') && primary(&pair[0], &pair[1]) == Ordering::Equal)
            {
                return Ok(Output {
                    status: 1,
                    stderr: if opts.has('C') {
                        String::new()
                    } else {
                        format!("sort: {}:{}: disorder: {}\n", files[0], index + 2, pair[1])
                    },
                    ..Output::default()
                });
            }
        }
        return Ok(Output::default());
    }
    lines.sort_by(|a, b| compare(a, b));
    if opts.has('u') {
        lines.dedup_by(|a, b| primary(a, b) == Ordering::Equal);
    }
    let mut text = String::new();
    for line in lines {
        text.push_str(&line);
        text.push(delimiter);
    }
    bounded(text.len())?;
    if let Some((_, file)) = opts.counts.last() {
        let path = normalize(file, &world.terminal.cwd)?;
        world
            .fs_mut()?
            .write(&path, &text, actor)
            .map_err(|error| {
                domain(format!(
                    "sort: open failed: {file}: {}",
                    error_reason(error)
                ))
            })?;
        Ok(Output::default())
    } else {
        Ok(Output::success(text))
    }
}

fn uniq(world: &mut WorldState, args: &[String], actor: &str) -> GameResult<Output> {
    let opts = options(
        "uniq",
        args,
        "cduiz",
        "fsw",
        &[
            ("count", 'c'),
            ("repeated", 'd'),
            ("unique", 'u'),
            ("ignore-case", 'i'),
            ("zero-terminated", 'z'),
            ("skip-fields", 'f'),
            ("skip-chars", 's'),
            ("check-chars", 'w'),
            ("version", 'V'),
        ],
    )?;
    if let Some(output) = early("uniq", &opts) {
        return Ok(output);
    }
    let c_locale = locale(world, "LC_CTYPE")?;
    if opts.files.len() > 2 {
        return Err(domain(format!("uniq: extra operand '{}'", opts.files[2])));
    }
    let count = |flag: char, default: usize| -> GameResult<usize> {
        opts.counts
            .iter()
            .rev()
            .find(|(f, _)| *f == flag)
            .map(|(_, value)| {
                value.parse::<usize>().map_err(|_| {
                    domain(format!(
                        "uniq: invalid number of fields/characters: '{value}'"
                    ))
                })
            })
            .unwrap_or(Ok(default))
    };
    let fields = count('f', 0)?;
    let skip = count('s', 0)?;
    let width = count('w', usize::MAX)?;
    let source = opts.files.first().map(String::as_str).unwrap_or("-");
    let destination = opts
        .files
        .get(1)
        .filter(|s| *s != "-")
        .map(|s| normalize(s, &world.terminal.cwd))
        .transpose()?;
    if let Some(path) = &destination {
        world.fs_mut()?.prepare_output(path, actor, false)?;
    }
    let text = read(world, source, actor, &mut world.terminal.stdin.clone())
        .map_err(|error| domain(format!("uniq: {source}: {}", error_reason(error))))?;
    bounded(text.len())?;
    let delimiter = if opts.has('z') { '\0' } else { '\n' };
    let key = |line: &str| {
        let mut rest = line;
        for _ in 0..fields.min(line.len() + 1) {
            rest = rest.trim_start_matches([' ', '\t']);
            rest = rest.find([' ', '\t']).map(|i| &rest[i..]).unwrap_or("");
            if rest.is_empty() {
                break;
            }
        }
        let selected: String = rest.chars().skip(skip).take(width).collect();
        if opts.has('i') {
            if c_locale {
                selected.to_ascii_lowercase()
            } else {
                selected.to_lowercase()
            }
        } else {
            selected
        }
    };
    let mut groups: Vec<(&str, String, usize)> = Vec::new();
    for line in text.split_terminator(delimiter) {
        let k = key(line);
        if let Some(last) = groups.last_mut().filter(|last| last.1 == k) {
            last.2 += 1;
        } else {
            if groups.len() >= RECORD_LIMIT {
                return Err(domain("uniq: virtual group limit: 65536"));
            }
            groups.push((line, k, 1));
        }
    }
    let mut output = String::new();
    for (line, _, count) in groups {
        if (opts.has('d') && count == 1) || (opts.has('u') && count != 1) {
            continue;
        }
        if opts.has('c') {
            output.push_str(&format!("{count:>7} "));
        }
        output.push_str(line);
        output.push(delimiter);
    }
    bounded(output.len())?;
    if let Some(path) = destination {
        world.fs_mut()?.write(&path, &output, actor)?;
        Ok(Output::default())
    } else {
        Ok(Output::success(output))
    }
}
