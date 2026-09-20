use super::foundation_options::{error, parse, quote, shell_quote_always};
use super::pathnames::reason;
use crate::{
    error::{GameError, GameResult},
    terminal_io::Output,
    vfs::{self, Errno, ModeChange},
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
        Ok(o) => o,
        Err(o) => return Ok(*o),
    };
    let mut out = if let Some(kind) = opts.special {
        Output::success(super::foundation_messages::message(name, invocation, kind))
    } else if opts.files.is_empty() {
        error(name, invocation, "missing operand", false)
    } else {
        Output::default()
    };
    out.stderr.insert_str(0, &opts.prelude);
    if opts.special.is_some() || opts.files.is_empty() {
        return Ok(out);
    }
    if name == "rmdir" {
        return rmdir(world, &opts, actor, invocation);
    }
    let mask = world.terminal.shell.umask;
    let change = if let Some((_, mode)) = opts.path_values.last() {
        match ModeChange::parse(mode, 0o777, mask, true) {
            Some(mode) => mode,
            None => {
                out.status = 1;
                out.stderr
                    .push_str(&format!("mkdir: invalid mode {}\n", quote(mode)));
                return Ok(out);
            }
        }
    } else {
        ModeChange {
            mode: 0o777 & !mask,
            affected: 0o1777,
        }
    };
    let parents = opts.flags.contains(&'p');
    let verbose = opts.flags.contains(&'v');
    for operand in opts.files {
        if parents && !operand.is_empty() {
            let stripped = operand.trim_end_matches('/');
            let mut failed = false;
            for (end, _) in stripped.match_indices('/') {
                if end == 0 || stripped.as_bytes()[end - 1] == b'/' {
                    continue;
                }
                let prefix = &operand[..end];
                let parent_mode = ModeChange {
                    mode: 0o777 & !(mask & !0o300),
                    affected: 0o1777,
                };
                let path = vfs::absolute(prefix, &world.terminal.cwd)?;
                match world.fs()?.stat(&format!("{path}/."), actor) {
                    Ok(_) => continue,
                    Err(GameError::Vfs(Errno::NotFound)) => {}
                    Err(e) => {
                        failure(&mut out, prefix, "create directory", e);
                        failed = true;
                        break;
                    }
                }
                match world
                    .fs_mut()?
                    .mkdir_mode(&path, actor, parent_mode.mode, None)
                {
                    Ok(()) => announce(&mut out, invocation, prefix, verbose),
                    Err(e) => {
                        failure(&mut out, prefix, "create directory", e);
                        failed = true;
                        break;
                    }
                }
            }
            if failed {
                continue;
            }
        }
        let result = vfs::absolute(&operand, &world.terminal.cwd).and_then(|path| {
            world.fs_mut()?.mkdir_mode(
                &path,
                actor,
                change.mode
                    & !0o6000
                    & if (change.affected & 0o6000) | (change.mode & 0o1000) != 0 {
                        !0o022
                    } else {
                        0o7777
                    },
                Some(change),
            )
        });
        match result {
            Ok(()) => announce(&mut out, invocation, &operand, verbose),
            Err(e) => {
                if parents && matches!(e, GameError::Vfs(Errno::Exists)) {
                    let path = vfs::absolute(&operand, &world.terminal.cwd)?;
                    match world.fs()?.stat(&path, actor) {
                        Ok(n) if n.kind == "directory" => continue,
                        Err(GameError::Vfs(Errno::Loop)) => {
                            failure(&mut out, &operand, "stat", GameError::Vfs(Errno::Loop));
                            continue;
                        }
                        _ => {}
                    }
                }
                failure(&mut out, &operand, "create directory", e);
            }
        }
    }
    Ok(out)
}
fn announce(out: &mut Output, invocation: &str, path: &str, verbose: bool) {
    if verbose {
        out.stdout.push_str(&format!(
            "{invocation}: created directory {}\n",
            shell_quote_always(path)
        ));
    }
}
fn failure(out: &mut Output, path: &str, action: &str, e: GameError) {
    out.status = 1;
    out.stderr.push_str(&format!(
        "mkdir: cannot {action} {}: {}\n",
        shell_quote_always(path),
        reason(e)
    ));
}

fn rmdir(
    world: &mut WorldState,
    opts: &super::foundation_options::Options,
    actor: &str,
    invocation: &str,
) -> GameResult<Output> {
    let mut out = Output::default();
    let parents = opts.flags.contains(&'p');
    let verbose = opts.flags.contains(&'v');
    let ignore = opts.flags.contains(&'\u{1}');
    for operand in &opts.files {
        let mut spelling = operand.clone();
        let mut ancestor = false;
        loop {
            if verbose {
                out.stdout.push_str(&format!(
                    "{invocation}: removing directory, {}\n",
                    shell_quote_always(&spelling)
                ));
            }
            let absolute = vfs::absolute(&spelling, &world.terminal.cwd);
            let result = vfs::absolute(&spelling, &world.terminal.cwd)
                .and_then(|path| world.fs_mut()?.rmdir(&path, actor));
            if let Err(e) = result {
                let nonempty = ignore
                    && match &e {
                        GameError::Vfs(Errno::NotEmpty | Errno::Exists) => true,
                        GameError::Vfs(
                            Errno::Access | Errno::NotPermitted | Errno::ReadOnly | Errno::Busy,
                        ) => absolute
                            .as_ref()
                            .ok()
                            .and_then(|p| world.fs().ok()?.child_names(p, actor).ok())
                            .is_some_and(|names| !names.is_empty()),
                        _ => false,
                    };
                if nonempty {
                    break;
                }
                let trailing_link = !ancestor
                    && spelling.ends_with('/')
                    && matches!(e, GameError::Vfs(Errno::NotDirectory))
                    && absolute.as_ref().is_ok_and(|p| {
                        let fs = world.fs().unwrap();
                        let followed = match fs.stat(p, actor) {
                            Ok(n) => n.kind == "directory",
                            Err(GameError::Vfs(Errno::NotDirectory)) => false,
                            Err(_) => true,
                        };
                        followed
                            && fs
                                .lstat(p.trim_end_matches('/'), actor)
                                .is_ok_and(|n| n.kind == "symlink")
                    });
                let action = if ancestor && !matches!(e, GameError::Vfs(Errno::NotDirectory)) {
                    "directory "
                } else {
                    ""
                };
                out.status = 1;
                out.stderr.push_str(&format!(
                    "rmdir: failed to remove {action}{}: {}\n",
                    shell_quote_always(&spelling),
                    if trailing_link {
                        "Symbolic link not followed".into()
                    } else {
                        reason(e)
                    }
                ));
                break;
            }
            if !parents {
                break;
            }
            let trimmed = spelling.trim_end_matches('/');
            let Some(slash) = trimmed.rfind('/') else {
                break;
            };
            let parent = trimmed[..slash].trim_end_matches('/');
            spelling = if parent.is_empty() {
                "/".into()
            } else {
                parent.into()
            };
            ancestor = true;
        }
    }
    Ok(out)
}
