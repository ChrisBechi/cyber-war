//! Bounded virtual shell. Expansion produces arguments, never another parser input.
use crate::{
    error::GameResult,
    terminal_io::Output,
    vfs::{domain, normalize},
    world::WorldState,
};
use std::collections::BTreeMap;

pub const HELP: &str = "Cyber War shell subset\nbash|sh SCRIPT [ARGS...] or -c COMMANDS [NAME [ARGS...]]; source|. FILE [ARGS...]\nSingle/double quotes, backslash escapes, $NAME/${NAME}, $0..$9/${10}, $@/$*/$#/$?, standalone assignments, export, source, exit and return.\nScripts preserve stdout/stderr and last status; ordinary command errors do not imply set -e. Child shells isolate cwd/user/host/environment; source shares the caller.\nLimits: 512 lines, 8 nested shells, 4 MiB output. No pipes, background, command substitution, general globbing, loops/functions, set, logical lists or multiline quotes. Unsupported syntax fails explicitly.\n";

pub fn identifier(value: &str) -> bool {
    let mut chars = value.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}
pub fn words(
    input: &str,
    env: &BTreeMap<String, String>,
    script: &str,
    args: &[String],
    status: i32,
) -> GameResult<Vec<String>> {
    if input.len() > 8192 {
        return Err(domain("shell: command limit: 8192 bytes"));
    }
    let mut chars = input.chars().peekable();
    let mut tokens = Vec::new();
    let mut word = String::new();
    let mut quote = None;
    let mut started = false;
    let mut empty_at = false;
    while let Some(c) = chars.next() {
        if c == '\0' || c == '\x1b' {
            return Err(domain("shell: unsupported control character"));
        }
        if quote == Some('\'') {
            if c == '\'' {
                quote = None;
            } else {
                word.push(c);
            }
            continue;
        }
        if c == '`' {
            return Err(domain("shell: command substitution is not implemented"));
        }
        if c == '\\' {
            let next = chars
                .next()
                .ok_or_else(|| domain("shell: trailing escape"))?;
            if matches!(next, '\0' | '\x1b') {
                return Err(domain("shell: unsupported control character"));
            }
            if quote == Some('"') && !['$', '`', '"', '\\'].contains(&next) {
                word.push('\\');
            }
            word.push(next);
            started = true;
            empty_at = false;
            continue;
        }
        if c == '"' || (c == '\'' && quote.is_none()) {
            if quote == Some(c) {
                quote = None;
                if empty_at && word.is_empty() {
                    started = false;
                }
            } else {
                quote = Some(c);
                started = true;
            }
            continue;
        }
        if c == '$' {
            let name = if chars.peek() == Some(&'{') {
                chars.next();
                let mut name = String::new();
                let mut closed = false;
                for c in chars.by_ref() {
                    if c == '}' {
                        closed = true;
                        break;
                    }
                    name.push(c);
                }
                if !closed {
                    return Err(domain("shell: unclosed parameter expansion"));
                }
                name
            } else if chars
                .peek()
                .is_some_and(|c| c.is_ascii_digit() || "@*#?".contains(*c))
            {
                chars.next().unwrap().to_string()
            } else {
                let mut name = String::new();
                while chars
                    .peek()
                    .is_some_and(|c| c.is_ascii_alphanumeric() || *c == '_')
                {
                    name.push(chars.next().unwrap());
                }
                name
            };
            if name.is_empty() {
                if chars.peek() == Some(&'(') {
                    return Err(domain("shell: command substitution is not implemented"));
                }
                word.push('$');
                started = true;
                continue;
            }
            let values = match name.as_str() {
                "@" => args.to_vec(),
                "*" => vec![args.join(" ")],
                "#" => vec![args.len().to_string()],
                "?" => vec![status.to_string()],
                "0" => vec![script.into()],
                n if n.chars().all(|c| c.is_ascii_digit()) => vec![n
                    .parse::<usize>()
                    .ok()
                    .and_then(|n| n.checked_sub(1))
                    .and_then(|n| args.get(n))
                    .cloned()
                    .unwrap_or_default()],
                n if identifier(n) => vec![env.get(n).cloned().unwrap_or_default()],
                _ => return Err(domain("shell: parameter operator is not implemented")),
            };
            if values.iter().any(|value| value.contains(['\0', '\x1b'])) {
                return Err(domain("shell: unsupported control character in expansion"));
            }
            let assignment = (tokens.is_empty() || tokens.first().is_some_and(|t| t == "export"))
                && word.split_once('=').is_some_and(|(key, _)| identifier(key));
            if assignment && quote.is_none() {
                word.push_str(&values.join(" "));
                started = true;
            } else if quote.is_some() {
                if name == "@" {
                    if values.is_empty() {
                        empty_at = true;
                    }
                    for (i, value) in values.iter().enumerate() {
                        if i > 0 {
                            tokens.push(std::mem::take(&mut word));
                        }
                        word.push_str(value);
                    }
                } else {
                    word.push_str(&values.join(" "));
                }
                started = true;
            } else {
                let value = values.join(" ");
                for ch in value.chars() {
                    if matches!(ch, ' ' | '\t' | '\n') {
                        if started {
                            tokens.push(std::mem::take(&mut word));
                            started = false;
                        }
                    } else {
                        word.push(ch);
                        started = true;
                    }
                }
            }
            continue;
        }
        if quote.is_some() {
            word.push(c);
            empty_at = false;
            continue;
        }
        match c {
            '#' if !started => break,
            ';' | '|' | '&' | '`' | '<' | '(' | ')' => {
                return Err(domain("shell: operators/pipelines are not implemented"))
            }
            '>' => {
                if started {
                    tokens.push(std::mem::take(&mut word));
                    started = false;
                }
                if chars.peek() == Some(&'>') {
                    chars.next();
                    tokens.push("\0>>".into());
                } else {
                    tokens.push("\0>".into());
                }
            }
            c if c.is_whitespace() => {
                if started {
                    tokens.push(std::mem::take(&mut word));
                    started = false;
                }
            }
            c => {
                word.push(c);
                started = true;
            }
        }
    }
    if quote.is_some() {
        return Err(domain("shell: unterminated quote"));
    }
    if started {
        tokens.push(word);
    }
    if tokens.len() > 4096 || tokens.iter().map(String::len).sum::<usize>() > 65536 {
        return Err(domain("shell: expanded argument limit"));
    }
    Ok(tokens)
}

pub fn invoke(
    world: &mut WorldState,
    name: &str,
    args: &[String],
    actor: &str,
) -> GameResult<Output> {
    if args.first().is_some_and(|a| a == "--help" || a == "-h") {
        return Ok(Output::success(HELP.into()));
    }
    if args.first().is_some_and(|a| a == "--version") {
        return Ok(Output::success(
            "Cyber War virtual shell (not GNU Bash)\n".into(),
        ));
    }
    if world.terminal.shell_depth >= 8 {
        return Err(domain("shell: maximum nesting depth exceeded"));
    }
    let sourced = matches!(name, "source" | ".");
    let (script, content, positional) = if args.first().is_some_and(|a| a == "-c") && !sourced {
        (
            args.get(2).cloned().unwrap_or_else(|| name.into()),
            args.get(1)
                .ok_or_else(|| domain("shell: -c requires a command string"))?
                .clone(),
            args.get(3..).unwrap_or_default().to_vec(),
        )
    } else {
        let values = if args.first().is_some_and(|a| a == "--") {
            &args[1..]
        } else {
            args
        };
        let script = values
            .first()
            .ok_or_else(|| domain("shell: script path required"))?;
        if script.starts_with('-') {
            return Err(domain("shell: unsupported interpreter option"));
        }
        let path = normalize(script, &world.terminal.cwd)?;
        (
            script.clone(),
            world.fs()?.read(&path, actor)?,
            values[1..].to_vec(),
        )
    };
    let original = world.terminal.clone();
    if !sourced {
        world.terminal.env.retain(|key, _| {
            original.exported.contains(key)
                || ["HOME", "PWD", "OLDPWD", "PATH", "USER", "LANG"].contains(&key.as_str())
        });
    }
    world.terminal.shell_depth += 1;
    world.terminal.user = actor.into();
    let result = program(
        world,
        if sourced { "bash" } else { &script },
        &content,
        &positional,
        actor,
        sourced,
    )
    .map(|result| result.0);
    if sourced {
        world.terminal.shell_depth = original.shell_depth;
        world.terminal.user = original.user;
    } else {
        world.terminal = original;
    }
    // Errors are command results, not a rollback of prior script commands.
    Ok(result.unwrap_or_else(|e| Output {
        stderr: format!("{e}\n"),
        status: 2,
        ..Output::default()
    }))
}

fn program(
    world: &mut WorldState,
    script: &str,
    content: &str,
    args: &[String],
    actor: &str,
    sourced: bool,
) -> GameResult<(Output, bool)> {
    if content.lines().count() > 512 {
        return Err(domain("shell: script exceeds 512 lines"));
    }
    if let Some(shebang) = content
        .lines()
        .next()
        .and_then(|line| line.strip_prefix("#!"))
    {
        if ![
            "/bin/bash",
            "/bin/sh",
            "/usr/bin/bash",
            "/usr/bin/sh",
            "/usr/bin/env bash",
            "/usr/bin/env sh",
        ]
        .contains(&shebang.trim())
        {
            return Err(domain("shell: unsupported shebang interpreter"));
        }
    }
    let mut out = Output::default();
    for (line_no, line) in content.lines().enumerate() {
        let tokens = match words(
            line,
            &crate::terminal::virtual_env(world, actor),
            script,
            args,
            out.status,
        ) {
            Ok(t) => t,
            Err(e) => {
                out.stderr
                    .push_str(&format!("{script}:{}: {e}\n", line_no + 1));
                out.status = 2;
                break;
            }
        };
        if tokens.is_empty() {
            continue;
        }
        if [
            "set", "if", "then", "fi", "for", "while", "until", "case", "function", "eval", "exec",
        ]
        .contains(&tokens[0].as_str())
        {
            out.stderr
                .push_str("shell: control syntax/set is not implemented\n");
            out.status = 2;
            break;
        }
        if tokens[0] == "exit" || tokens[0] == "return" {
            if tokens[0] == "return" && !sourced {
                out.stderr
                    .push_str("return: only valid in a sourced file\n");
                out.status = 1;
                continue;
            }
            out.status = if tokens.len() > 2 {
                2
            } else if let Some(status) = tokens.get(1) {
                match status.parse::<i64>() {
                    Ok(status) => status.rem_euclid(256) as i32,
                    Err(_) => {
                        out.stderr.push_str("shell: numeric status required\n");
                        2
                    }
                }
            } else {
                out.status
            };
            return Ok((out, tokens[0] == "exit"));
        }
        let result = (|| -> GameResult<(Output, bool)> {
            if matches!(tokens[0].as_str(), "source" | ".") {
                let filename = tokens
                    .get(1)
                    .ok_or_else(|| domain("source: filename required"))?;
                let path = normalize(filename, &world.terminal.cwd)?;
                let nested = world.fs()?.read(&path, actor)?;
                if world.terminal.shell_depth >= 8 {
                    return Err(domain("shell: maximum nesting depth exceeded"));
                }
                world.terminal.shell_depth += 1;
                let result = program(
                    world,
                    script,
                    &nested,
                    if tokens.len() > 2 { &tokens[2..] } else { args },
                    actor,
                    true,
                );
                world.terminal.shell_depth -= 1;
                result
            } else {
                let previous_foreground = world.terminal.foreground.clone();
                let result = crate::terminal::execute_parts(world, Ok(tokens));
                if result.interactive.is_some() {
                    world.terminal.nano = None;
                    world.terminal.foreground = previous_foreground;
                    return Err(domain(
                        "shell: interactive editors cannot run inside scripts",
                    ));
                }
                Ok((
                    Output {
                        stdout: result.stdout,
                        stderr: result.stderr,
                        status: result.exit_code,
                    },
                    false,
                ))
            }
        })();
        let (result, exited) = match result {
            Ok(result) => result,
            Err(e) => {
                out.stderr
                    .push_str(&format!("{script}:{}: {e}\n", line_no + 1));
                out.status = 2;
                break;
            }
        };
        out.stdout.push_str(&result.stdout);
        out.stderr.push_str(&result.stderr);
        out.status = result.status;
        if exited {
            return Ok((out, true));
        }
        if out.stdout.len() + out.stderr.len() > 4 * 1024 * 1024 {
            return Err(domain("shell: output limit: 4 MiB"));
        }
    }
    Ok((out, false))
}
