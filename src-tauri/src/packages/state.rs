use super::{deb, model::*, repository};
use crate::{
    error::GameResult,
    vfs::{domain, parent},
    world::WorldState,
};
use std::collections::BTreeSet;

pub fn mkdirs(world: &mut WorldState, path: &str) -> GameResult<()> {
    if world.vfs.nodes.contains_key(path) {
        world.vfs.directory(path, "root")?;
        return Ok(());
    }
    if path != "/" {
        mkdirs(world, parent(path))?;
    }
    world.vfs.mkdir(path, "root")
}
pub fn fingerprint(world: &WorldState) -> GameResult<String> {
    Ok(deb::hash(&serde_json::to_vec(&(
        &world.vfs,
        &world.packages,
    ))?))
}
pub fn content_hash(world: &WorldState, path: &str) -> Option<String> {
    world.vfs.nodes.get(path).map(|n| {
        n.blob
            .as_ref()
            .map_or_else(|| deb::hash(n.content.as_bytes()), |b| b.hash.clone())
    })
}
pub fn emit(
    world: &mut WorldState,
    transaction: &str,
    kind: &str,
    package: Option<&Definition>,
    repo: Option<&str>,
) {
    world.packages.sequence += 1;
    world.packages.events.push(Event {
        sequence: world.packages.sequence,
        transaction: transaction.into(),
        kind: kind.into(),
        package: package.map(|p| p.name.clone()),
        version: package.map(|p| p.version.clone()),
        repository: repo.map(String::from),
    });
    if world.packages.events.len() > 512 {
        world.packages.events.remove(0);
    }
}
fn projection(world: &mut WorldState, path: &str, text: &str) -> GameResult<()> {
    mkdirs(world, parent(path))?;
    if let Some(mut n) = world.vfs.nodes.get_mut(path) {
        n.metadata.remove("packageProjection");
    }
    world.vfs.write(path, text, "root")?;
    if let Some(mut n) = world.vfs.nodes.get_mut(path) {
        n.mode = 0o444;
        n.metadata.insert("packageProjection".into(), "true".into());
    }
    Ok(())
}
pub fn sync(world: &mut WorldState) -> GameResult<()> {
    world.vfs.collect();

    let installed = world
        .packages
        .installed
        .values()
        .cloned()
        .collect::<Vec<_>>();
    let status = installed
        .iter()
        .map(|i| {
            format!(
                "{}Status: {} ok {}\n\n",
                deb::control(&i.definition),
                if i.status == Status::ConfigFiles {
                    "deinstall"
                } else {
                    "install"
                },
                i.status.label()
            )
        })
        .collect::<String>();
    projection(world, "/var/lib/dpkg/status", &status)?;
    let mut expected = BTreeSet::new();
    for i in &installed {
        let path = format!("/var/lib/dpkg/info/{}.list", i.definition.name);
        let files = world
            .packages
            .ownership
            .iter()
            .filter(|(_, o)| o.owners.contains(&i.definition.name))
            .map(|(path, _)| format!("{path}\n"))
            .collect::<String>();
        projection(world, &path, &files)?;
        expected.insert(path);
    }
    world.vfs.nodes.retain(|path, node| {
        !path.starts_with("/var/lib/dpkg/info/")
            || !node.metadata.contains_key("packageProjection")
            || expected.contains(path)
    });
    let summary = world
        .packages
        .indexes
        .iter()
        .map(|(repo, entries)| {
            format!(
                "Repository: {repo}\nPackages: {}\nIndex-SHA256: {}\n\n",
                entries.len(),
                deb::hash(&serde_json::to_vec(entries).unwrap_or_default())
            )
        })
        .collect::<String>();
    projection(world, "/var/lib/apt/lists/virtual-index", &summary)?;
    let marks = installed
        .iter()
        .filter(|i| i.automatic)
        .map(|i| format!("Package: {}\nAuto-Installed: 1\n\n", i.definition.name))
        .collect::<String>();
    projection(world, "/var/lib/apt/extended_states", &marks)?;
    world.vfs.collect();
    world.packages.revision += 1;
    Ok(())
}
pub fn initialize(world: &mut WorldState) -> GameResult<()> {
    if world.packages.initialized {
        return extend_baseline_shell_binding(world);
    }
    for path in [
        "/usr/bin",
        "/usr/lib",
        "/usr/share/man/man1",
        "/usr/share/applications",
        "/etc/apt/sources.list.d",
        "/var/lib/dpkg/info",
        "/var/lib/apt/lists",
        "/var/cache/apt/archives",
    ] {
        mkdirs(world, path)?;
    }
    if !world.vfs.nodes.contains_key("/etc/apt/sources.list") {
        world.vfs.write(
            "/etc/apt/sources.list",
            &format!(
                "deb {} rolling main contrib non-free\n",
                repository::OFFICIAL
            ),
            "root",
        )?;
    }
    let legacy = world
        .settings
        .get("aptInstalled")
        .and_then(|s| serde_json::from_str::<Vec<String>>(s).ok());
    let manual = world
        .settings
        .get("aptManual")
        .and_then(|s| serde_json::from_str::<Vec<String>>(s).ok());
    let held = world
        .settings
        .get("aptHeld")
        .and_then(|s| serde_json::from_str::<Vec<String>>(s).ok())
        .unwrap_or_default();
    let mut baseline = repository::baseline();
    if let Some(legacy) = &legacy {
        baseline.retain(|p| p.essential || legacy.contains(&p.name));
        for name in legacy {
            if baseline.iter().any(|p| p.name == *name) {
                continue;
            }
            if let Some(tool) = crate::software::CATALOG
                .entries
                .iter()
                .find(|p| p.package == *name)
            {
                let mut p = repository::definition(name, "0~legacy1", &tool.description);
                for cmd in &tool.commands {
                    p.files.push(repository::file(
                        &format!("/usr/bin/{cmd}"),
                        &format!("CYBER-WAR CATALOG\n{cmd}\n"),
                        1024,
                        false,
                        Some(&format!("catalog:{cmd}")),
                    ));
                }
                baseline.push(p);
            }
        }
    }
    for p in baseline {
        for f in &p.files {
            mkdirs(world, parent(&f.path))?;
            if !world.vfs.nodes.contains_key(&f.path) {
                world.vfs.write(&f.path, &f.content, "root")?;
                world.vfs.chmod(&f.path, "root", f.mode)?;
            }
            world.packages.ownership.insert(
                f.path.clone(),
                Ownership {
                    owners: BTreeSet::from([p.name.clone()]),
                    kind: f.kind.clone(),
                    shipped_hash: deb::hash(f.content.as_bytes()),
                    conffile: f.conffile,
                },
            );
        }
        world.packages.installed.insert(
            p.name.clone(),
            Installed {
                automatic: manual.as_ref().is_some_and(|m| !m.contains(&p.name)),
                held: held.contains(&p.name),
                status: Status::Installed,
                problems: Vec::new(),
                origin: "baseline".into(),
                installed_at: world.playtime_seconds,
                definition: p,
            },
        );
    }
    world.packages.initialized = true;
    world.packages.managed = repository::REPOSITORIES
        .values()
        .flat_map(|r| r.entries.iter())
        .map(|(_, e)| e.package.name.clone())
        .chain(world.packages.installed.keys().cloned())
        .collect();
    for key in ["aptInstalled", "aptUpdatedAt", "aptHeld"] {
        world.settings.remove(key);
    }
    // aptManual remains a legacy personal setting for existing mission journals;
    // no package operation reads it after this one-time migration.
    sync(world)
}

/// Old saves predate the incremental yes engine's packaged executable. Extend
/// only that known baseline definition; preserve removed/modified/custom files.
fn extend_baseline_shell_binding(world: &mut WorldState) -> GameResult<()> {
    for name in [
        "base64",
        "basename",
        "cat",
        "chmod",
        "chown",
        "cp",
        "cut",
        "date",
        "df",
        "dirname",
        "du",
        "env",
        "groups",
        "head",
        "id",
        "ls",
        "mkdir",
        "mv",
        "printenv",
        "readlink",
        "realpath",
        "rm",
        "seq",
        "sha256sum",
        "sort",
        "stat",
        "tail",
        "tee",
        "touch",
        "tr",
        "uname",
        "uniq",
        "wc",
        "whoami",
        "yes",
        "ln",
        "rmdir",
    ] {
        extend_baseline_binding(world, name)?;
    }
    Ok(())
}
fn extend_baseline_binding(world: &mut WorldState, name: &str) -> GameResult<()> {
    let owned_path = format!("/usr/bin/{name}");
    let path = owned_path.as_str();
    let eligible = world
        .packages
        .installed
        .get("coreutils")
        .is_some_and(|installed| {
            installed.origin == "baseline"
                && installed.status == Status::Installed
                && installed.definition.version == "9.7"
                && !installed
                    .definition
                    .files
                    .iter()
                    .any(|file| file.path == path)
        });
    if !eligible
        || world.vfs.nodes.contains_key(path)
        || world.packages.ownership.contains_key(path)
    {
        return Ok(());
    }
    let file = repository::baseline()
        .into_iter()
        .find(|package| package.name == "coreutils")
        .and_then(|package| package.files.into_iter().find(|file| file.path == path))
        .ok_or_else(|| domain("missing baseline shell binding"))?;
    world.vfs.write(path, &file.content, "root")?;
    world.vfs.chmod(path, "root", file.mode)?;
    world.packages.ownership.insert(
        path.into(),
        Ownership {
            owners: BTreeSet::from(["coreutils".into()]),
            kind: file.kind.clone(),
            shipped_hash: deb::hash(file.content.as_bytes()),
            conffile: false,
        },
    );
    world
        .packages
        .installed
        .get_mut("coreutils")
        .unwrap()
        .definition
        .files
        .push(file);
    sync(world)
}
pub fn validate(world: &WorldState) -> GameResult<()> {
    if !world.packages.initialized {
        return Ok(());
    }
    for (name, installed) in &world.packages.installed {
        if name != &installed.definition.name {
            return Err(domain("invalid installed package key"));
        }
        deb::validate(&installed.definition)?;
    }
    for (path, o) in &world.packages.ownership {
        if o.owners.is_empty()
            || o.kind != "directory" && o.owners.len() != 1
            || o.owners
                .iter()
                .any(|name| !world.packages.installed.contains_key(name))
        {
            return Err(domain(format!("invalid package ownership: {path}")));
        }
    }
    let expected = world
        .packages
        .installed
        .values()
        .map(|i| {
            format!(
                "{}Status: {} ok {}\n\n",
                deb::control(&i.definition),
                if i.status == Status::ConfigFiles {
                    "deinstall"
                } else {
                    "install"
                },
                i.status.label()
            )
        })
        .collect::<String>();
    if world
        .vfs
        .nodes
        .get("/var/lib/dpkg/status")
        .is_none_or(|n| n.content != expected)
    {
        return Err(domain("package database projection mismatch"));
    }
    Ok(())
}
