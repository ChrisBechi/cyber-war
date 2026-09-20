use super::foundation_options::{error, parse, shell_quote};
use crate::{
    error::GameResult,
    terminal_io::Output,
    vfs::{self, CanonicalMode},
    world::WorldState,
};

pub(super) fn execute(
    world: &mut WorldState,
    name: &str,
    invocation: &str,
    args: &[String],
    actor: &str,
) -> GameResult<Output> {
    let posix = super::foundation::exported(world, actor).contains_key("POSIXLY_CORRECT");
    let opts = match parse(name, invocation, args, posix) {
        Ok(opts) => opts,
        Err(out) => return Ok(*out),
    };
    if let Some(kind) = opts.special {
        return Ok(Output::success(super::foundation_messages::message(
            name, invocation, kind,
        )));
    }
    if opts.files.is_empty() {
        return Ok(error(name, invocation, "missing operand", false));
    }
    if name == "realpath" {
        return realpath(world, &opts, actor);
    }
    let mut mode = None;
    let mut verbose = false;
    let mut no_newline = false;
    let mut zero = false;
    for flag in opts.flags {
        match flag {
            'f' => mode = Some(CanonicalMode::AllButLast),
            'e' => mode = Some(CanonicalMode::Existing),
            'm' => mode = Some(CanonicalMode::Missing),
            'v' => verbose = true,
            'q' | 's' => verbose = false,
            'n' => no_newline = true,
            'z' => zero = true,
            _ => unreachable!(),
        }
    }
    let mut out = Output::default();
    if opts.files.len() > 1 && no_newline {
        out.stderr
            .push_str("readlink: ignoring --no-newline with multiple arguments\n");
        no_newline = false;
    }
    for operand in opts.files {
        let result = vfs::absolute(&operand, &world.terminal.cwd).and_then(|path| match mode {
            Some(mode) => world.fs()?.canonicalize(&path, actor, mode, false),
            None => world.fs()?.readlink(&path, actor),
        });
        match result {
            Ok(value) => {
                out.stdout.push_str(&value);
                if !no_newline {
                    out.stdout.push(if zero { '\0' } else { '\n' });
                }
            }
            Err(e) => {
                out.status = 1;
                if verbose {
                    out.stderr.push_str(&format!(
                        "{name}: {}: {}\n",
                        shell_quote(&operand),
                        reason(e)
                    ));
                }
            }
        }
    }
    Ok(out)
}

fn realpath(
    world: &WorldState,
    opts: &super::foundation_options::Options,
    actor: &str,
) -> GameResult<Output> {
    let mut mode = CanonicalMode::AllButLast;
    let (mut no_links, mut logical, mut quiet, mut zero) = (false, false, false, false);
    for flag in &opts.flags {
        match flag {
            'e' => mode = CanonicalMode::Existing,
            'm' => mode = CanonicalMode::Missing,
            'L' => {
                no_links = true;
                logical = true;
            }
            's' => {
                no_links = true;
                logical = false;
            }
            'P' => {
                no_links = false;
                logical = false;
            }
            'q' => quiet = true,
            'z' => zero = true,
            _ => unreachable!(),
        }
    }
    let canon = |operand: &str| -> GameResult<String> {
        let path = vfs::absolute(operand, &world.terminal.cwd)?;
        let fs = world.fs()?;
        let name = fs.canonicalize(&path, actor, mode, no_links)?;
        if logical {
            fs.canonicalize(&name, actor, mode, false)
        } else {
            Ok(name)
        }
    };
    let mut relative_to = opts
        .path_values
        .iter()
        .rev()
        .find(|(c, _)| *c == '\u{1}')
        .map(|(_, s)| s.as_str());
    let relative_base = opts
        .path_values
        .iter()
        .rev()
        .find(|(c, _)| *c == '\u{2}')
        .map(|(_, s)| s.as_str());
    if relative_to.is_none() {
        relative_to = relative_base;
    }
    let mut to = None;
    let mut base = None;
    for (raw, destination) in [(relative_to, &mut to), (relative_base, &mut base)] {
        if let Some(raw) = raw {
            let value = match canon(raw) {
                Ok(value) => value,
                Err(e) => return Ok(path_error("realpath", raw, e)),
            };
            if mode == CanonicalMode::Existing {
                match world.fs()?.stat(&value, actor) {
                    Ok(n) if n.kind != "directory" => {
                        return Ok(path_error(
                            "realpath",
                            raw,
                            crate::error::GameError::Vfs(vfs::Errno::NotDirectory),
                        ))
                    }
                    Err(e) => {
                        return Ok(Output {
                            status: 1,
                            stderr: format!(
                                "realpath: cannot stat {}: {}\n",
                                super::foundation_options::shell_quote_always(&value),
                                reason(e)
                            ),
                            ..Output::default()
                        })
                    }
                    _ => {}
                }
            }
            *destination = Some(value);
        }
    }
    if base
        .as_ref()
        .zip(to.as_ref())
        .is_some_and(|(base, to)| !vfs::path_prefix(base, to))
    {
        to = None;
    }
    let mut out = Output::default();
    for operand in &opts.files {
        match canon(operand) {
            Ok(value) => {
                let relative = to
                    .as_ref()
                    .filter(|_| base.as_ref().is_none_or(|b| vfs::path_prefix(b, &value)));
                out.stdout
                    .push_str(&relative.map_or(value.clone(), |to| vfs::relative_path(&value, to)));
                out.stdout.push(if zero { '\0' } else { '\n' });
            }
            Err(e) => {
                out.status = 1;
                if !quiet {
                    out.stderr
                        .push_str(&path_error("realpath", operand, e).stderr);
                }
            }
        }
    }
    Ok(out)
}

fn path_error(name: &str, path: &str, error: impl std::fmt::Display) -> Output {
    Output {
        stderr: format!("{name}: {}: {}\n", shell_quote(path), reason(error)),
        status: 1,
        ..Output::default()
    }
}

pub(super) fn reason(e: impl std::fmt::Display) -> String {
    match super::wc::reason(e).as_str() {
        "File name too long" => "Filename too long".into(),
        other => other.into(),
    }
}
