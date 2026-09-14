//! Read/search/permission contracts; no host filesystem or real process access.
use crate::{
    error::GameResult,
    terminal_io::{options, Output},
    vfs::{domain, normalize, parent, VfsNode},
    world::WorldState,
};
use regex::{Regex, RegexBuilder};

pub fn manual(name: &str) -> Option<String> {
    let syntax = match name {
        "ls" => "ls [-1aAldFhrSt] [--] [PATH...]\nOne entry per line; -l symbolic permissions/virtual timestamps/sizes; -a includes . and ..; -A hidden entries; -d directory itself; -F classify; -h binary units; -r reverse; -S size; -t modification order. No symlinks, color, recursive -R or real disk allocation.",
        "grep" => "grep [-EFGivnclLqHhwx] [-e PATTERN] [-m COUNT] [--] PATTERN FILE...\nBRE by default, -E extended subset, -F literal. -i case-insensitive, -v invert, -n numbers, -c counts, -l/-L file names, -q quiet, -H/-h headers, -w words, -x whole line. Repeated -e is OR. Exit 0 match, 1 no match, 2 error. No backreferences, lookaround, PCRE, recursion, context flags or binary matching.",
        "find" => "find [PATH...] [-name GLOB] [-iname GLOB] [-type f|d] [-mindepth N] [-maxdepth N] [-print]\nPredicates combine with AND. Globs support * and ?. Includes the starting path at depth zero. No exec/delete, symlinks, OR/NOT or bracket classes.",
        "chmod" => "chmod [-Rv] [--] MODE PATH...\nOctal 000..777 or explicit symbolic classes u/g/o/a with +,-,= and r/w/x/X (comma-separated). -R recursive. No special bits, implicit umask or class-copy expressions. Atomic VFS mutation.",
        "chown" => "chown [-Rv] [--] OWNER[:GROUP] PATH...\nVirtual root only. Known users/groups: root, kali, vex. Owner alone preserves the group; :GROUP changes only group. -R recursive. Atomic VFS mutation.",
        _ => return None,
    };
    Some(format!("{name} — CYBER WAR audited virtual subset\n\n{syntax}\n\nOnly the selected local/SSH VFS is used. Unsupported options fail explicitly.\n"))
}

pub fn execute(
    world: &mut WorldState,
    name: &str,
    args: &[String],
    actor: &str,
) -> Option<GameResult<Output>> {
    let result = match name {
        "ls" => listing(world, args, actor),
        "grep" => grep(world, args, actor),
        "find" => find(world, args, actor),
        "chmod" | "chown" => permissions(world, name, args, actor),
        _ => return None,
    };
    Some(if matches!(name, "grep" | "ls") {
        Ok(result.unwrap_or_else(|e| Output {
            stderr: format!("{e}\n"),
            status: 2,
            ..Output::default()
        }))
    } else {
        result
    })
}

fn bounded(output: &Output) -> GameResult<()> {
    if output.stdout.len() + output.stderr.len() > 4 * 1024 * 1024 {
        return Err(domain("virtual output limit: 4 MiB"));
    }
    Ok(())
}
fn size(n: &VfsNode) -> usize {
    n.blob.as_ref().map_or(n.content.len(), |b| b.size)
}
fn mode_text(n: &VfsNode) -> String {
    let mut out = if n.kind == "directory" { "d" } else { "-" }.to_owned();
    for shift in [6, 3, 0] {
        for (bit, letter) in [(4, 'r'), (2, 'w'), (1, 'x')] {
            out.push(if (n.mode >> shift) & bit != 0 {
                letter
            } else {
                '-'
            });
        }
    }
    out
}
fn listing(world: &WorldState, args: &[String], actor: &str) -> GameResult<Output> {
    let opts = options(
        "ls",
        args,
        "1aAldFhrSt",
        "",
        &[
            ("all", 'a'),
            ("almost-all", 'A'),
            ("directory", 'd'),
            ("classify", 'F'),
            ("human-readable", 'h'),
            ("reverse", 'r'),
        ],
    )?;
    if opts.help {
        return Ok(Output::success(manual("ls").unwrap()));
    }
    let files = if opts.files.is_empty() {
        vec![".".into()]
    } else {
        opts.files.clone()
    };
    let mut out = Output::default();
    for file in &files {
        let result = (|| -> GameResult<String> {
            let path = normalize(file, &world.terminal.cwd)?;
            let n = world.fs()?.stat(&path, actor)?;
            let directory = n.kind == "directory" && !opts.has('d');
            let mut entries = if directory {
                world.fs()?.list(&path, actor)?
            } else {
                let mut n = n.clone();
                n.name = file.clone();
                vec![n]
            };
            if directory {
                entries.retain(|n| opts.has('a') || opts.has('A') || !n.name.starts_with('.'));
                if opts.has('a') && !opts.has('A') {
                    for (label, p) in [(".", path.as_str()), ("..", parent(&path))] {
                        let mut n = world.fs()?.stat(p, actor)?.clone();
                        n.name = label.into();
                        entries.push(n);
                    }
                }
            }
            let order = opts.flags.iter().rev().find(|f| "St".contains(**f));
            entries.sort_by(|a, b| match order {
                Some('S') => size(b).cmp(&size(a)).then(a.name.cmp(&b.name)),
                Some('t') => b.modified_at.cmp(&a.modified_at).then(a.name.cmp(&b.name)),
                _ => a.name.cmp(&b.name),
            });
            if opts.has('r') {
                entries.reverse();
            }
            let mut text = String::new();
            if directory && files.len() > 1 {
                text.push_str(&format!("{file}:\n"));
            }
            if directory && opts.has('l') {
                text.push_str(&format!(
                    "total {}\n",
                    entries
                        .iter()
                        .map(|n| size(n).div_ceil(1024))
                        .sum::<usize>()
                ));
            }
            for n in entries {
                let suffix = if opts.has('F') {
                    if n.kind == "directory" {
                        "/"
                    } else if n.mode & 0o111 != 0 {
                        "*"
                    } else {
                        ""
                    }
                } else {
                    ""
                };
                if opts.has('l') {
                    let bytes = size(&n);
                    let length = if opts.has('h') && bytes >= 1024 {
                        format!("{:.1}K", bytes as f64 / 1024.)
                    } else {
                        bytes.to_string()
                    };
                    let timestamp = chrono::DateTime::from_timestamp(n.modified_at as i64, 0)
                        .ok_or_else(|| domain("invalid virtual timestamp"))?
                        .format("%b %e %H:%M")
                        .to_string();
                    text.push_str(&format!(
                        "{} 1 {} {} {} {} {}{}\n",
                        mode_text(&n),
                        n.owner,
                        n.group,
                        length,
                        timestamp,
                        n.name,
                        suffix
                    ));
                } else {
                    text.push_str(&format!("{}{suffix}\n", n.name));
                }
            }
            Ok(text)
        })();
        match result {
            Ok(text) => {
                if files.len() > 1 && !out.stdout.is_empty() {
                    out.stdout.push('\n');
                }
                out.stdout.push_str(&text);
            }
            Err(e) => {
                out.error("ls", file, e);
                out.status = 2;
            }
        }
        bounded(&out)?;
    }
    Ok(out)
}

// Translate the promised BRE subset, preserving literal ERE operators outside
// brackets. Reject extensions whose meaning would differ in the Rust engine.
fn expression(pattern: &str, extended: bool, fixed: bool) -> GameResult<String> {
    if pattern.len() > 4096 {
        return Err(domain("grep: pattern limit: 4096 bytes"));
    }
    if fixed {
        return Ok(regex::escape(pattern));
    }
    if pattern.contains("(?") {
        return Err(domain(
            "grep: lookaround and inline regex extensions are not supported",
        ));
    }
    let mut result = String::new();
    let mut chars = pattern.chars();
    let mut bracket = false;
    while let Some(c) = chars.next() {
        if c == '\\' {
            let next = chars
                .next()
                .ok_or_else(|| domain("grep: trailing backslash"))?;
            if next.is_ascii_digit() || "pPuUx".contains(next) {
                return Err(domain("grep: unsupported regex escape/backreference"));
            }
            if !extended && !bracket && "+?(){}|".contains(next) {
                result.push(next);
            } else {
                result.push('\\');
                result.push(next);
            }
        } else {
            if !extended && !bracket && "+?(){}|".contains(c) {
                result.push('\\');
            }
            result.push(c);
            if c == '[' {
                bracket = true;
            } else if c == ']' {
                bracket = false;
            }
        }
    }
    Ok(result)
}
fn grep(world: &WorldState, args: &[String], actor: &str) -> GameResult<Output> {
    let opts = options(
        "grep",
        args,
        "EFGivnclLqHhwx",
        "em",
        &[
            ("extended-regexp", 'E'),
            ("fixed-strings", 'F'),
            ("basic-regexp", 'G'),
            ("ignore-case", 'i'),
            ("invert-match", 'v'),
            ("line-number", 'n'),
            ("count", 'c'),
            ("files-with-matches", 'l'),
            ("files-without-match", 'L'),
            ("quiet", 'q'),
            ("with-filename", 'H'),
            ("no-filename", 'h'),
            ("word-regexp", 'w'),
            ("line-regexp", 'x'),
            ("regexp", 'e'),
            ("max-count", 'm'),
        ],
    )?;
    if opts.help {
        return Ok(Output::success(manual("grep").unwrap()));
    }
    let mut files = opts.files.clone();
    let mut patterns: Vec<String> = opts
        .counts
        .iter()
        .filter(|(flag, _)| *flag == 'e')
        .flat_map(|(_, v)| v.split('\n').map(String::from))
        .collect();
    if patterns.is_empty() {
        if files.is_empty() {
            return Err(domain("grep: pattern required"));
        }
        patterns.push(files.remove(0));
    }
    if files.is_empty() {
        return Err(domain("grep: stdin is not supported; provide files"));
    }
    let matcher = opts.flags.iter().rev().find(|f| "EFG".contains(**f));
    let regexes: Vec<Regex> = patterns
        .iter()
        .map(|p| {
            let expression = expression(p, matcher == Some(&'E'), matcher == Some(&'F'))?;
            let expression = if opts.has('x') {
                format!("^(?:{expression})$")
            } else if opts.has('w') {
                format!("(?:^|\\W)(?:{expression})(?:$|\\W)")
            } else {
                expression
            };
            RegexBuilder::new(&expression)
                .case_insensitive(opts.has('i'))
                .size_limit(1024 * 1024)
                .build()
                .map_err(|e| domain(format!("grep: invalid expression: {e}")))
        })
        .collect::<GameResult<_>>()?;
    let maximum = opts
        .counts
        .iter()
        .rev()
        .find(|(f, _)| *f == 'm')
        .map(|(_, n)| {
            n.parse::<usize>()
                .map_err(|_| domain("grep: invalid max-count"))
        })
        .transpose()?
        .unwrap_or(usize::MAX);
    let headers = opts
        .flags
        .iter()
        .rev()
        .find(|f| "Hh".contains(**f))
        .map_or(files.len() > 1, |f| *f == 'H');
    let mut out = Output::default();
    let mut found = false;
    let mut failed = false;
    for file in &files {
        let content = if file == "-" {
            Err(domain("stdin is not implemented"))
        } else {
            normalize(file, &world.terminal.cwd).and_then(|p| world.fs()?.read(&p, actor))
        };
        let content = match content {
            Ok(c) => c,
            Err(e) => {
                out.error("grep", file, e);
                failed = true;
                continue;
            }
        };
        let mut count = 0;
        for (index, line) in content.split_inclusive('\n').enumerate() {
            if count >= maximum {
                break;
            }
            let text = line.strip_suffix('\n').unwrap_or(line);
            if regexes.iter().any(|r| r.is_match(text)) == opts.has('v') {
                continue;
            }
            count += 1;
            found = true;
            if opts.has('q') {
                out.stdout.clear();
                out.status = 0;
                return Ok(out);
            }
            if !opts.has('c') && !opts.has('l') && !opts.has('L') {
                if headers {
                    out.stdout.push_str(&format!("{file}:"));
                }
                if opts.has('n') {
                    out.stdout.push_str(&format!("{}:", index + 1));
                }
                out.stdout.push_str(text);
                out.stdout.push('\n');
            }
            bounded(&out)?;
        }
        if opts.has('l') || opts.has('L') {
            if (count > 0 && opts.has('l')) || (count == 0 && opts.has('L')) {
                out.stdout.push_str(&format!("{file}\n"));
            }
        } else if opts.has('c') && !opts.has('q') {
            if headers {
                out.stdout.push_str(&format!("{file}:"));
            }
            out.stdout.push_str(&format!("{count}\n"));
        }
    }
    out.status = if failed {
        2
    } else if found {
        0
    } else {
        1
    };
    bounded(&out)?;
    Ok(out)
}

pub fn glob(pattern: &str, insensitive: bool) -> GameResult<Regex> {
    if pattern.contains(['[', ']', '\\']) || pattern.len() > 4096 {
        return Err(domain("glob subset supports only * and ?"));
    }
    let mut regex = String::from("^");
    for c in pattern.chars() {
        match c {
            '*' => regex.push_str(".*"),
            '?' => regex.push('.'),
            _ => regex.push_str(&regex::escape(&c.to_string())),
        }
    }
    regex.push('$');
    RegexBuilder::new(&regex)
        .case_insensitive(insensitive)
        .build()
        .map_err(|e| domain(e.to_string()))
}
fn find(world: &WorldState, args: &[String], actor: &str) -> GameResult<Output> {
    if args == ["--help"] {
        return Ok(Output::success(manual("find").unwrap()));
    }
    let mut roots = Vec::new();
    let mut names = Vec::new();
    let mut kinds = Vec::new();
    let mut min = 0;
    let mut max = usize::MAX;
    let mut i = 0;
    while i < args.len() && !args[i].starts_with('-') {
        roots.push(args[i].clone());
        i += 1;
    }
    while i < args.len() {
        let flag = args[i].as_str();
        i += 1;
        if matches!(flag, "-print" | "-a" | "-and") {
            continue;
        }
        if !["-name", "-iname", "-type", "-mindepth", "-maxdepth"].contains(&flag) {
            return Err(domain(format!("find: unsupported predicate {flag}")));
        }
        let value = args
            .get(i)
            .ok_or_else(|| domain("find: predicate argument required"))?;
        i += 1;
        match flag {
            "-name" | "-iname" => names.push(glob(value, flag == "-iname")?),
            "-type" if value == "f" || value == "d" => kinds.push(value.clone()),
            "-type" => return Err(domain("find: supported types: f, d")),
            "-mindepth" => min = value.parse().map_err(|_| domain("find: invalid depth"))?,
            "-maxdepth" => max = value.parse().map_err(|_| domain("find: invalid depth"))?,
            _ => unreachable!(),
        }
    }
    if roots.is_empty() {
        roots.push(".".into());
    }
    let mut out = Output::default();
    for root in roots {
        let path = normalize(&root, &world.terminal.cwd)?;
        let mut pending = vec![(path, root, 0)];
        while let Some((path, display, depth)) = pending.pop() {
            let node = match world.fs()?.stat(&path, actor) {
                Ok(n) => n,
                Err(e) => {
                    out.error("find", &display, e);
                    continue;
                }
            };
            if depth >= min
                && names.iter().all(|r| r.is_match(&node.name))
                && kinds
                    .iter()
                    .all(|k| (k == "d") == (node.kind == "directory"))
            {
                out.stdout.push_str(&format!("{display}\n"));
            }
            if depth < max && node.kind == "directory" {
                match world.fs()?.list(&path, actor) {
                    Ok(children) => {
                        for child in children.into_iter().rev() {
                            pending.push((
                                child.id,
                                format!("{}/{}", display.trim_end_matches('/'), child.name),
                                depth + 1,
                            ));
                        }
                    }
                    Err(e) => out.error("find", &display, e),
                }
            }
            bounded(&out)?;
        }
    }
    Ok(out)
}

fn symbolic(old: u16, directory: bool, value: &str) -> GameResult<u16> {
    if !value.is_empty() && value.chars().all(|c| ('0'..='7').contains(&c)) {
        let mode = u16::from_str_radix(value, 8).map_err(|_| domain("chmod: invalid mode"))?;
        if mode <= 0o777 {
            return Ok(mode);
        }
    }
    let mut mode = old;
    for clause in value.split(',') {
        let index = clause
            .find(['+', '-', '='])
            .ok_or_else(|| domain("chmod: invalid symbolic mode"))?;
        let classes = &clause[..index];
        let permissions = &clause[index + 1..];
        if classes.is_empty()
            || !classes.chars().all(|c| "ugoa".contains(c))
            || !permissions.chars().all(|c| "rwxX".contains(c))
        {
            return Err(domain("chmod: explicit u/g/o/a and r/w/x/X required"));
        }
        let bits = permissions.chars().fold(0, |bits, c| {
            bits | match c {
                'r' => 4,
                'w' => 2,
                'x' => 1,
                'X' if directory || old & 0o111 != 0 => 1,
                _ => 0,
            }
        });
        for (class, shift) in [('u', 6), ('g', 3), ('o', 0)] {
            if classes.contains(class) || classes.contains('a') {
                let mask = 7 << shift;
                let bits = bits << shift;
                match clause.as_bytes()[index] {
                    b'+' => mode |= bits,
                    b'-' => mode &= !bits,
                    b'=' => mode = (mode & !mask) | bits,
                    _ => unreachable!(),
                }
            }
        }
    }
    Ok(mode)
}
fn permissions(
    world: &mut WorldState,
    name: &str,
    args: &[String],
    actor: &str,
) -> GameResult<Output> {
    let opts = options(
        name,
        args,
        "Rv",
        "",
        &[("recursive", 'R'), ("verbose", 'v')],
    )?;
    if opts.help {
        return Ok(Output::success(manual(name).unwrap()));
    }
    if opts.files.len() < 2 {
        return Err(domain(format!("{name}: mode/owner and paths required")));
    }
    let spec = &opts.files[0];
    let (owner, group) = spec
        .split_once(':')
        .map_or((Some(spec.as_str()), None), |(u, g)| {
            (if u.is_empty() { None } else { Some(u) }, Some(g))
        });
    if name == "chown"
        && (actor != "root"
            || owner
                .into_iter()
                .chain(group)
                .any(|v| !["kali", "root", "vex"].contains(&v)))
    {
        return Err(domain("chown: permission denied or unknown user/group"));
    }
    let mut changes = Vec::new();
    for file in &opts.files[1..] {
        let mut pending = vec![normalize(file, &world.terminal.cwd)?];
        while let Some(path) = pending.pop() {
            let node = world.fs()?.stat(&path, actor)?.clone();
            if opts.has('R') && node.kind == "directory" {
                pending.extend(world.fs()?.list(&path, actor)?.into_iter().map(|n| n.id));
            }
            if name == "chmod" && actor != "root" && node.owner != actor {
                return Err(domain("chmod: permission denied"));
            }
            let mode = if name == "chmod" {
                symbolic(node.mode, node.kind == "directory", spec)?
            } else {
                node.mode
            };
            changes.push((path, mode));
        }
    }
    let mut out = Output::default();
    for (path, mode) in changes {
        let node = world
            .fs_mut()?
            .nodes
            .get_mut(&path)
            .ok_or_else(|| domain("missing node"))?;
        if name == "chmod" {
            node.mode = mode;
        } else {
            if let Some(owner) = owner {
                node.owner = owner.into();
            }
            if let Some(group) = group {
                node.group = group.into();
            }
        }
        if opts.has('v') {
            out.stdout.push_str(&format!("{name}: '{path}' updated\n"));
        }
    }
    Ok(out)
}
