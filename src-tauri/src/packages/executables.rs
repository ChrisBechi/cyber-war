use super::{model::Status, repository};
use crate::{
    error::GameResult,
    terminal_io::Output,
    vfs::{domain, normalize},
    world::WorldState,
};
use std::{collections::BTreeSet, sync::LazyLock};

static MANAGED: LazyLock<BTreeSet<String>> = LazyLock::new(|| {
    repository::REPOSITORIES
        .values()
        .flat_map(|r| r.entries.iter())
        .flat_map(|(_, e)| e.package.files.iter())
        .filter(|f| f.binding.is_some())
        .map(|f| f.path.rsplit('/').next().unwrap_or("").to_string())
        .collect()
});
pub fn managed(name: &str) -> bool {
    MANAGED.contains(name)
}
pub fn resolve(world: &WorldState, name: &str, actor: &str) -> Option<String> {
    if world.terminal.host.is_some() {
        return None;
    }
    let env = crate::terminal::virtual_env(world, actor);
    let paths = if name.contains('/') {
        vec![normalize(name, &world.terminal.cwd).ok()?]
    } else {
        env.get("PATH")?
            .split(':')
            .filter_map(|p| normalize(if p.is_empty() { "." } else { p }, &world.terminal.cwd).ok())
            .map(|p| format!("{}/{name}", p.trim_end_matches('/')))
            .collect()
    };
    for path in paths {
        let Some(owner) = world.packages.ownership.get(&path) else {
            continue;
        };
        let Some(installed) = owner
            .owners
            .iter()
            .find_map(|name| world.packages.installed.get(name))
        else {
            continue;
        };
        if installed.status != Status::Installed {
            continue;
        }
        let Some(file) = installed
            .definition
            .files
            .iter()
            .find(|f| f.path == path && f.binding.is_some())
        else {
            continue;
        };
        let Ok(node) = world.vfs.stat(&path, actor) else {
            continue;
        };
        let executable = if actor == "root" {
            node.mode & 0o111 != 0
        } else if node.owner == actor {
            node.mode & 0o100 != 0
        } else if node.group == actor {
            node.mode & 0o010 != 0
        } else {
            node.mode & 0o001 != 0
        };
        if node.kind == "file" && executable && node.content == file.content && node.blob.is_none()
        {
            return Some(path);
        }
    }
    None
}

pub fn unavailable(world: &WorldState, name: &str, actor: &str) -> Output {
    let env = crate::terminal::virtual_env(world, actor);
    let paths = if name.contains('/') {
        normalize(name, &world.terminal.cwd)
            .into_iter()
            .collect::<Vec<_>>()
    } else {
        env.get("PATH")
            .into_iter()
            .flat_map(|p| p.split(':'))
            .filter_map(|p| normalize(if p.is_empty() { "." } else { p }, &world.terminal.cwd).ok())
            .map(|p| format!("{}/{name}", p.trim_end_matches('/')))
            .collect()
    };
    let mut reason = None;
    for path in paths {
        if let Ok(fs) = world.fs() {
            if fs.nodes.contains_key(&path) {
                reason = Some(match fs.stat(&path, actor) {
                    Ok(node) if node.kind == "directory" => "Is a directory",
                    Ok(node) => {
                        let mask = if actor == "root" {
                            0o111
                        } else if node.owner == actor {
                            0o100
                        } else if node.group == actor {
                            0o010
                        } else {
                            0o001
                        };
                        if node.mode & mask == 0 {
                            "Permission denied"
                        } else {
                            "Exec format error"
                        }
                    }
                    Err(_) => "Permission denied",
                });
            }
        }
    }
    Output {
        stderr: format!("bash: {name}: {}\n", reason.unwrap_or("command not found")),
        status: if reason.is_some() { 126 } else { 127 },
        ..Output::default()
    }
}
pub fn names(world: &WorldState, actor: &str) -> Vec<String> {
    world
        .packages
        .ownership
        .keys()
        .filter_map(|path| {
            let name = path.rsplit('/').next()?;
            resolve(world, name, actor)
                .filter(|resolved| resolved == path)
                .map(|_| name.to_owned())
        })
        .collect()
}
pub fn dispatch(
    world: &WorldState,
    name: &str,
    args: &[String],
    actor: &str,
) -> Option<GameResult<Output>> {
    let base = name.rsplit('/').next().unwrap_or(name);
    if world.terminal.host.is_some()
        || !managed(base)
            && !world
                .packages
                .ownership
                .contains_key(&format!("/usr/bin/{base}"))
    {
        return None;
    }
    Some((|| {
        let Some(path) = resolve(world, name, actor) else {
            return Ok(unavailable(world, name, actor));
        };
        let owner = world
            .packages
            .ownership
            .get(&path)
            .and_then(|o| o.owners.iter().next())
            .and_then(|n| world.packages.installed.get(n))
            .ok_or_else(|| domain("package binding missing"))?;
        let file = owner
            .definition
            .files
            .iter()
            .find(|f| f.path == path)
            .ok_or_else(|| domain("package payload missing"))?;
        let binding = file
            .binding
            .as_deref()
            .ok_or_else(|| domain("package binding missing"))?;
        if args == ["--version"] && binding != "builtin" {
            return Ok(Output::success(format!(
                "{} {}\n",
                owner.definition.name, owner.definition.version
            )));
        }
        if binding == "builtin" {
            return Err(domain("internal builtin dispatch"));
        }
        if args == ["--help"] || args == ["-h"] {
            return Ok(Output::success(format!(
                "{}\nUsage: {base} [--version|--help] [VIRTUAL_TARGET]\n",
                owner.definition.description
            )));
        }
        if let Some(command) = binding.strip_prefix("catalog:") {
            let tool = crate::software::by_command(command)
                .ok_or_else(|| domain("unknown virtual handler"))?;
            return crate::software::command(world, tool, args, actor).map(Output::success);
        }
        if args.len() > 1 || args.iter().any(|a| a.starts_with('-')) {
            return Err(domain("unsupported virtual tool arguments"));
        }
        let out = match binding {
            "netscan" | "iot" => {
                if !world.network.connected {
                    return Err(domain("network unreachable"));
                }
                let target = args.first();
                let mut out = String::new();
                for host in world
                    .network
                    .hosts
                    .values()
                    .filter(|h| target.is_none_or(|t| t == &h.hostname || t == &h.address))
                {
                    out.push_str(&format!("{} ({})\n", host.hostname, host.address));
                    for s in &host.services {
                        out.push_str(&format!(
                            "  {}/tcp {} {}\n",
                            s.port,
                            if s.running && host.firewall.contains(&s.port) {
                                "open"
                            } else {
                                "closed"
                            },
                            s.name
                        ));
                    }
                }
                if out.is_empty() {
                    return Err(domain("virtual target not found"));
                }
                out
            }
            "wireless" => world
                .network
                .wifi
                .iter()
                .map(|ap| {
                    format!(
                        "{} {} channel {} {} dBm\n",
                        ap.ssid, ap.bssid, ap.channel, ap.signal
                    )
                })
                .collect(),
            _ => return Err(domain("unsupported virtual handler")),
        };
        Ok(Output::success(out))
    })())
}
pub fn is_builtin(world: &WorldState, name: &str) -> bool {
    let path = format!("/usr/bin/{}", name.rsplit('/').next().unwrap_or(name));
    world.packages.installed.values().any(|i| {
        i.definition
            .files
            .iter()
            .any(|f| f.path == path && f.binding.as_deref() == Some("builtin"))
    })
}
pub fn manual(world: &WorldState, name: &str, actor: &str) -> Option<GameResult<String>> {
    let path = format!("/usr/share/man/man1/{name}.1");
    if world.vfs.nodes.contains_key(&path) {
        return Some(world.vfs.read(&path, actor));
    }
    if managed(name) && !is_builtin(world, name) {
        return Some(Err(domain(format!("No manual entry for {name}"))));
    }
    None
}
