use super::foundation_options::{error, parse, quote};
use super::options::{early, operand_error};
use crate::{error::GameResult, terminal_io::Output, world::WorldState};

pub(super) fn execute(
    world: &mut WorldState,
    name: &str,
    args: &[String],
    actor: &str,
    invocation: &str,
) -> GameResult<Output> {
    if name == "env" {
        if let Some(out) = early(
            name,
            args,
            "[-i] [-u NAME] [NAME=VALUE]... [COMMAND [ARG]...]",
        ) {
            return Ok(out);
        }
        return environment(world, args, actor);
    }
    let posix = exported(world, actor).contains_key("POSIXLY_CORRECT");
    let opts = match parse(name, invocation, args, posix) {
        Ok(v) => v,
        Err(out) => return Ok(*out),
    };
    if let Some(special) = opts.special {
        return Ok(Output::success(super::foundation_messages::message(
            name, invocation, special,
        )));
    }
    if name == "whoami" {
        if let Some(extra) = opts.files.first() {
            return Ok(error(
                name,
                invocation,
                &format!("extra operand {}", quote(extra)),
                false,
            ));
        }
        let fs = world.fs()?;
        let uid = fs.identity(actor).uid;
        return Ok(match fs.username_for_uid(uid) {
            Some(username) => Output::success(format!("{username}\n")),
            None => Output {
                stderr: format!("whoami: cannot find name for user ID {uid}\n"),
                status: 1,
                ..Default::default()
            },
        });
    }
    if name == "printenv" {
        let env = exported(world, actor);
        let mut out = Output::default();
        let terminator = if opts.zero { '\0' } else { '\n' };
        if opts.files.is_empty() {
            for (key, value) in ordered_environment(world, env) {
                out.stdout.push_str(&format!("{key}={value}{terminator}"));
            }
        } else {
            for key in &opts.files {
                if let Some(value) = env.get(key).filter(|_| !key.contains('=')) {
                    out.stdout.push_str(value);
                    out.stdout.push(terminator);
                } else {
                    out.status = 1;
                }
            }
        }
        return Ok(out);
    }
    if opts.files.is_empty() {
        return Ok(error(name, invocation, "missing operand", false));
    }
    if name == "basename" && !opts.multiple && opts.files.len() > 2 {
        return Ok(error(
            name,
            invocation,
            &format!("extra operand {}", quote(&opts.files[2])),
            false,
        ));
    }
    let suffix = opts.suffix.as_deref().or_else(|| {
        if !opts.multiple {
            opts.files.get(1).map(String::as_str)
        } else {
            None
        }
    });
    let files = if name == "basename" && !opts.multiple {
        &opts.files[..1]
    } else {
        &opts.files[..]
    };
    let mut text = String::new();
    for path in files {
        let trimmed = path.trim_end_matches('/');
        let value = if name == "basename" {
            let base = if !path.is_empty() && trimmed.is_empty() {
                "/"
            } else {
                trimmed.rsplit('/').next().unwrap_or("")
            };
            match suffix.filter(|s| !s.is_empty() && *s != base && base != "/") {
                Some(s) => base.strip_suffix(s).unwrap_or(base),
                None => base,
            }
        } else if trimmed.is_empty() {
            if path.is_empty() {
                "."
            } else {
                "/"
            }
        } else if let Some((parent, _)) = trimmed.rsplit_once('/') {
            let parent = parent.trim_end_matches('/');
            if parent.is_empty() {
                "/"
            } else {
                parent
            }
        } else {
            "."
        };
        text.push_str(value);
        text.push(if opts.zero { '\0' } else { '\n' });
    }
    Ok(Output::success(text))
}

fn ordered_environment(
    world: &WorldState,
    mut env: std::collections::BTreeMap<String, String>,
) -> Vec<(String, String)> {
    let mut inherited = Vec::new();
    for key in &world.terminal.shell.environment_order {
        if let Some(value) = env.remove(key) {
            inherited.push((key.clone(), value));
        }
    }
    // Implicit defaults have a deterministic initial order. Explicitly inherited
    // environments and later exports retain their process order unchanged.
    env.into_iter().chain(inherited).collect()
}

pub(super) fn exported(
    world: &WorldState,
    actor: &str,
) -> std::collections::BTreeMap<String, String> {
    let mut env = crate::terminal::virtual_env(world, actor);
    env.retain(|key, _| {
        !world.terminal.env.contains_key(key)
            || world.terminal.exported.contains(key)
            || ["HOME", "PWD", "OLDPWD", "PATH", "USER", "LANG"].contains(&key.as_str())
    });
    env
}

fn environment(world: &mut WorldState, args: &[String], actor: &str) -> GameResult<Output> {
    let mut env = exported(world, actor);
    let mut index = 0;
    while let Some(arg) = args.get(index) {
        if arg == "--" {
            index += 1;
            break;
        }
        if matches!(arg.as_str(), "-i" | "--ignore-environment" | "-") {
            env.clear();
        } else if matches!(arg.as_str(), "-u" | "--unset") {
            index += 1;
            let key = args
                .get(index)
                .ok_or_else(|| operand_error("env", "option requires an argument -- 'u'"))?;
            env.remove(key);
        } else if let Some(key) = arg
            .strip_prefix("--unset=")
            .or_else(|| arg.strip_prefix("-u").filter(|s| !s.is_empty()))
        {
            env.remove(key);
        } else if arg.starts_with('-') {
            return Err(operand_error(
                "env",
                &format!("unrecognized option '{arg}'"),
            ));
        } else {
            break;
        }
        index += 1;
    }
    while let Some((key, value)) = args.get(index).and_then(|s| s.split_once('=')) {
        if key.is_empty() {
            return Err(operand_error(
                "env",
                "invalid argument: empty variable name",
            ));
        }
        env.insert(key.into(), value.into());
        index += 1;
    }
    if index == args.len() {
        return Ok(Output::success(
            env.iter().map(|(k, v)| format!("{k}={v}\n")).collect(),
        ));
    }
    if world.terminal.shell_depth >= 16 {
        return Err(operand_error("env", "virtual execution depth limit"));
    }
    let original = world.terminal.clone();
    let defaults = crate::terminal::virtual_env(world, actor);
    world
        .terminal
        .shell
        .unset
        .extend(defaults.keys().filter(|k| !env.contains_key(*k)).cloned());
    world.terminal.exported = env.keys().cloned().collect();
    world.terminal.env = env;
    world.terminal.shell_depth += 1;
    let mut argv = args[index..].to_vec();
    let command = argv[0].clone();
    let resolved = if command.contains('/') {
        crate::vfs::normalize(&command, &world.terminal.cwd).ok()
    } else {
        crate::shell::path_file(world, &command, actor)
    };
    let result = if let Some(path) = resolved {
        argv[0] = path;
        crate::terminal::dispatch(world, &argv, actor)
    } else {
        Ok(Output {
            stderr: format!("env: '{command}': No such file or directory\n"),
            status: 127,
            ..Default::default()
        })
    };
    world.terminal = original;
    result
}
