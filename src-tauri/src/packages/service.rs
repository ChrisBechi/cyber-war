use super::{deb, model::*, repository, resolver, state};
use crate::{
    archive,
    error::GameResult,
    terminal_io::Output,
    vfs::{domain, normalize, parent},
    world::WorldState,
};
use std::collections::BTreeSet;

pub fn cache_path(p: &Definition) -> String {
    format!("/var/cache/apt/archives/{}.deb", p.key())
}
fn cached(world: &WorldState, e: &IndexEntry) -> bool {
    archive::bytes(world, &cache_path(&e.package), "root")
        .is_ok_and(|bytes| deb::hash(&bytes) == e.checksum && deb::decode(&bytes).is_ok())
}
pub fn plan(
    world: &WorldState,
    operation: &str,
    requests: &[String],
    dpkg: bool,
) -> GameResult<Plan> {
    if world.package_lock.is_some() {
        return Err(domain("E: Could not get lock /var/lib/dpkg/lock-frontend"));
    }
    let mut p = Plan {
        id: uuid::Uuid::new_v4().to_string(),
        fingerprint: state::fingerprint(world)?,
        operation: operation.into(),
        requested: requests
            .iter()
            .map(|s| s.split('=').next().unwrap_or(s).into())
            .collect(),
        install: Vec::new(),
        remove: Vec::new(),
        purge: operation == "purge",
        dpkg,
        download_size: 0,
        peak_size: 0,
    };
    match operation {
        "install" | "reinstall" | "repair" => {
            if dpkg {
                for path in requests {
                    let path = normalize(path, &world.terminal.cwd)?;
                    let bytes = archive::bytes(world, &path, "root")?;
                    let package = deb::decode(&bytes)?;
                    p.requested.insert(package.name.clone());
                    p.install.push(IndexEntry {
                        package,
                        repository: format!("file:{path}"),
                        suite: "local".into(),
                        component: "main".into(),
                        checksum: deb::hash(&bytes),
                    });
                }
            } else {
                p.install = resolver::resolve(&world.packages, requests, operation == "repair")?;
            }
        }
        "upgrade" => {
            let available = resolver::candidates(&world.packages);
            let updates = world
                .packages
                .installed
                .values()
                .filter(|i| i.status == Status::Installed && !i.held)
                .filter_map(|i| {
                    available
                        .iter()
                        .find(|e| {
                            e.package.name == i.definition.name
                                && super::version::compare(
                                    &e.package.version,
                                    &i.definition.version,
                                )
                                .is_gt()
                        })
                        .map(|e| e.package.name.clone())
                })
                .collect::<Vec<_>>();
            p.install = resolver::resolve(&world.packages, &updates, false)?;
        }
        "configure" => {
            p.install = world
                .packages
                .installed
                .values()
                .filter(|i| matches!(i.status, Status::Unpacked | Status::HalfConfigured))
                .map(|i| IndexEntry {
                    package: i.definition.clone(),
                    repository: "installed".into(),
                    suite: "local".into(),
                    component: "main".into(),
                    checksum: String::new(),
                })
                .collect();
        }
        "remove" | "purge" | "autoremove" => {
            p.remove = if operation == "autoremove" {
                resolver::orphans(&world.packages)
            } else {
                requests.to_vec()
            };
            for name in &p.remove {
                let i = world
                    .packages
                    .installed
                    .get(name)
                    .ok_or_else(|| domain(format!("E: Package {name} is not installed")))?;
                if i.definition.essential || i.held {
                    return Err(domain(format!("E: {name} is essential or held")));
                }
            }
            let remaining = world
                .packages
                .installed
                .iter()
                .filter(|(name, _)| !p.remove.contains(name))
                .map(|(n, i)| (n.clone(), i.clone()))
                .collect();
            if world
                .packages
                .installed
                .values()
                .filter(|i| i.status == Status::Installed && !p.remove.contains(&i.definition.name))
                .any(|i| !resolver::broken(&i.definition, &remaining).is_empty())
            {
                return Err(domain("E: Removal would break installed dependencies"));
            }
        }
        _ => return Err(domain("E: Unsupported package operation")),
    }
    if operation != "reinstall" && operation != "configure" && !dpkg {
        p.install.retain(|e| {
            world
                .packages
                .installed
                .get(&e.package.name)
                .is_none_or(|i| {
                    i.status != Status::Installed || i.definition.version != e.package.version
                })
        });
    }
    let names = p
        .install
        .iter()
        .map(|e| e.package.name.clone())
        .collect::<BTreeSet<_>>();
    let mut incoming = std::collections::BTreeMap::<&str, &str>::new();
    for e in &p.install {
        deb::validate(&e.package)?;
        for other in &p.install {
            if other.package.name != e.package.name
                && e.package
                    .conflicts
                    .iter()
                    .any(|r| resolver::satisfies(&other.package, r))
            {
                return Err(domain(format!(
                    "E: {} conflicts with {}",
                    e.package.name, other.package.name
                )));
            }
        }
        for old in world
            .packages
            .installed
            .values()
            .filter(|i| i.status != Status::ConfigFiles && !names.contains(&i.definition.name))
        {
            if e.package
                .conflicts
                .iter()
                .any(|r| resolver::satisfies(&old.definition, r))
                || old
                    .definition
                    .conflicts
                    .iter()
                    .any(|r| resolver::satisfies(&e.package, r))
            {
                return Err(domain(format!(
                    "E: {} conflicts with {}",
                    e.package.name, old.definition.name
                )));
            }
        }
        for f in &e.package.files {
            if let Some(kind) = incoming.insert(&f.path, &f.kind) {
                if kind != "directory" || f.kind != "directory" {
                    return Err(domain(format!(
                        "E: Multiple incoming packages own {}",
                        f.path
                    )));
                }
            }
            if let Some(old) = world.packages.ownership.get(&f.path) {
                if old.kind == "directory" && f.kind == "directory" {
                    continue;
                }
                if !old.owners.contains(&e.package.name)
                    && !old.owners.iter().all(|name| {
                        world.packages.installed.get(name).is_some_and(|i| {
                            e.package
                                .replaces
                                .iter()
                                .any(|r| resolver::satisfies(&i.definition, r))
                        })
                    })
                {
                    return Err(domain(format!("E: File collision: {}", f.path)));
                }
            } else if world
                .vfs
                .nodes
                .get(&f.path)
                .is_some_and(|n| n.kind != "directory" || f.kind != "directory")
            {
                return Err(domain(format!("E: Unowned file exists: {}", f.path)));
            }
        }
        if !dpkg && e.repository != "installed" && !cached(world, e) {
            p.download_size = p
                .download_size
                .checked_add(e.package.download_size)
                .ok_or_else(|| domain("E: Package size overflow"))?;
        }
        p.peak_size = p
            .peak_size
            .checked_add(e.package.installed_size())
            .ok_or_else(|| domain("E: Package size overflow"))?;
    }
    // Conservative peak: current files + complete incoming tree + cache + one extraction tree.
    p.peak_size = p
        .peak_size
        .checked_mul(2)
        .and_then(|v| v.checked_add(p.download_size))
        .ok_or_else(|| domain("E: Package size overflow"))?;
    if world
        .vfs
        .used_bytes()
        .checked_add(p.peak_size)
        .is_none_or(|v| v > world.vfs.capacity_bytes)
    {
        return Err(domain(
            "E: Not enough free space on virtual device for archives and extraction",
        ));
    }
    Ok(p)
}
pub fn summary(p: &Plan) -> String {
    let additional = p
        .install
        .iter()
        .filter(|e| !p.requested.contains(&e.package.name))
        .map(|e| e.package.name.as_str())
        .collect::<Vec<_>>();
    let mut out = String::new();
    if !additional.is_empty() {
        out.push_str(&format!(
            "The following additional packages will be installed:\n  {}\n",
            additional.join(" ")
        ));
    }
    if !p.install.is_empty() {
        out.push_str(&format!(
            "Packages to install/configure:\n  {}\n",
            p.install
                .iter()
                .map(|e| format!("{} ({})", e.package.name, e.package.version))
                .collect::<Vec<_>>()
                .join(" ")
        ));
    }
    if !p.remove.is_empty() {
        out.push_str(&format!(
            "Packages to {}:\n  {}\n",
            if p.purge { "purge" } else { "remove" },
            p.remove.join(" ")
        ));
    }
    out.push_str(&format!(
        "Need to get {} bytes of archives. Peak additional disk space: {} bytes.\n",
        p.download_size, p.peak_size
    ));
    out
}
fn remove(
    world: &mut WorldState,
    name: &str,
    purge: bool,
    upgrading: bool,
    out: &mut String,
) -> GameResult<()> {
    let paths = world
        .packages
        .ownership
        .iter()
        .filter(|(_, o)| o.owners.contains(name))
        .map(|(path, o)| (path.clone(), o.clone()))
        .collect::<Vec<_>>();
    for (path, mut ownership) in paths.into_iter().rev() {
        archive::jobs::check_cancel()?;
        if ownership.conffile && !purge {
            continue;
        }
        ownership.owners.remove(name);
        if !ownership.owners.is_empty() {
            world.packages.ownership.insert(path, ownership);
            continue;
        }
        if let Some(node) = world.vfs.nodes.get(&path) {
            if !purge
                && node.kind == "directory"
                && world.packages.ownership.iter().any(|(child, o)| {
                    child.starts_with(&format!("{path}/")) && o.owners.contains(name)
                })
            {
                continue;
            }
            let modified = node.kind == "file"
                && state::content_hash(world, &path).as_deref() != Some(&ownership.shipped_hash);
            if modified && !ownership.conffile {
                out.push_str(&format!("Preserving modified file {path}\n"));
                if upgrading {
                    return Err(domain(format!(
                        "E: Modified package file blocks upgrade: {path}"
                    )));
                }
            } else if node.kind != "directory"
                || !world
                    .vfs
                    .nodes
                    .keys()
                    .any(|p| p.starts_with(&format!("{path}/")))
            {
                world.vfs.remove(&path, "root", false)?;
            }
        }
        world.packages.ownership.remove(&path);
    }
    if !upgrading {
        let retained = world
            .packages
            .ownership
            .values()
            .any(|o| o.owners.contains(name));
        if retained {
            if let Some(i) = world.packages.installed.get_mut(name) {
                i.status = Status::ConfigFiles;
                i.problems.clear();
            }
        } else {
            world.packages.installed.remove(name);
        }
    }
    Ok(())
}
fn unpack(
    world: &mut WorldState,
    e: &IndexEntry,
    manual: bool,
    out: &mut String,
) -> GameResult<()> {
    let p = &e.package;
    let previous = world.packages.installed.get(&p.name).cloned();
    if previous.is_some() {
        remove(world, &p.name, false, true, out)?;
    }
    for f in &p.files {
        archive::jobs::check_cancel()?;
        owned_directories(world, parent(&f.path), &p.name)?;
        let old = world.packages.ownership.get(&f.path).cloned();
        if f.conffile && old.is_some() {
            let changed =
                state::content_hash(world, &f.path) != old.as_ref().map(|o| o.shipped_hash.clone());
            if changed {
                if world.vfs.nodes.contains_key(&f.path)
                    && state::content_hash(world, &f.path).as_deref()
                        != Some(&deb::hash(f.content.as_bytes()))
                {
                    let alternate = world
                        .vfs
                        .available_path(&format!("{}.dpkg-dist", f.path), false);
                    world.vfs.write(&alternate, &f.content, "root")?;
                    world.packages.ownership.insert(
                        alternate.clone(),
                        Ownership {
                            owners: BTreeSet::from([p.name.clone()]),
                            kind: "file".into(),
                            shipped_hash: deb::hash(f.content.as_bytes()),
                            conffile: true,
                        },
                    );
                    out.push_str(&format!(
                        "Keeping local configuration {}; new default: {alternate}\n",
                        f.path
                    ));
                }
                world.packages.ownership.insert(
                    f.path.clone(),
                    Ownership {
                        owners: BTreeSet::from([p.name.clone()]),
                        kind: f.kind.clone(),
                        shipped_hash: deb::hash(f.content.as_bytes()),
                        conffile: true,
                    },
                );
                continue;
            }
        }
        if f.kind == "directory" {
            state::mkdirs(world, &f.path)?;
        } else if f.kind == "symlink" {
            world.vfs.symlink(&f.path, &f.content, "root")?;
        } else {
            world.vfs.write(&f.path, &f.content, "root")?;
        }
        world.vfs.chmod(&f.path, "root", f.mode)?;
        if let Some(n) = world.vfs.nodes.get_mut(&f.path) {
            n.metadata.insert("packageOwner".into(), p.name.clone());
            n.metadata
                .insert("logicalSize".into(), f.logical_size.to_string());
        }
        let mut owners = BTreeSet::from([p.name.clone()]);
        if f.kind == "directory" {
            if let Some(o) = old {
                owners.extend(o.owners);
            }
        }
        world.packages.ownership.insert(
            f.path.clone(),
            Ownership {
                owners,
                kind: f.kind.clone(),
                shipped_hash: deb::hash(f.content.as_bytes()),
                conffile: f.conffile,
            },
        );
    }
    world.packages.installed.insert(
        p.name.clone(),
        Installed {
            definition: p.clone(),
            status: Status::Unpacked,
            automatic: !manual && previous.as_ref().is_none_or(|i| i.automatic),
            held: previous.as_ref().is_some_and(|i| i.held),
            problems: Vec::new(),
            origin: e.repository.clone(),
            installed_at: world.playtime_seconds,
        },
    );
    world.packages.managed.insert(p.name.clone());
    out.push_str(&format!("Unpacking {} ({}) ...\n", p.name, p.version));
    Ok(())
}
fn owned_directories(world: &mut WorldState, path: &str, name: &str) -> GameResult<()> {
    let mut missing = Vec::new();
    let mut current = path;
    while current != "/" {
        if !world.vfs.nodes.contains_key(current) {
            missing.push(current.to_string());
        }
        if let Some(o) = world.packages.ownership.get_mut(current) {
            if o.kind == "directory" {
                o.owners.insert(name.into());
            }
        }
        current = parent(current);
    }
    state::mkdirs(world, path)?;
    for path in missing {
        world.packages.ownership.insert(
            path,
            Ownership {
                owners: BTreeSet::from([name.into()]),
                kind: "directory".into(),
                shipped_hash: deb::hash(b""),
                conffile: false,
            },
        );
    }
    Ok(())
}
fn configure_actions(world: &mut WorldState, p: &Definition) -> GameResult<()> {
    for action in &p.actions {
        archive::jobs::check_cancel()?;
        match action {
            Action::EnsureDirectory { path } => owned_directories(world, path, &p.name)?,
            Action::DefaultFile { path, content } => {
                owned_directories(world, parent(path), &p.name)?;
                if !world.vfs.nodes.contains_key(path) {
                    world.vfs.write(path, content, "root")?;
                    world.packages.ownership.insert(
                        path.clone(),
                        Ownership {
                            owners: BTreeSet::from([p.name.clone()]),
                            kind: "file".into(),
                            shipped_hash: deb::hash(content.as_bytes()),
                            conffile: false,
                        },
                    );
                }
            }
        }
    }
    Ok(())
}
fn configure(world: &mut WorldState, names: &[String], out: &mut String) -> GameResult<bool> {
    let mut pending = names.iter().cloned().collect::<BTreeSet<_>>();
    let mut failed = BTreeSet::new();
    loop {
        let mut progressed = false;
        for name in pending.clone() {
            if failed.contains(&name) {
                continue;
            }
            archive::jobs::check_cancel()?;
            let Some(installed) = world.packages.installed.get(&name).cloned() else {
                continue;
            };
            let missing = resolver::broken(&installed.definition, &world.packages.installed);
            if !missing.is_empty() {
                if let Some(i) = world.packages.installed.get_mut(&name) {
                    i.problems = missing;
                }
                continue;
            }
            let mut configured = world.clone();
            if let Err(error) = configure_actions(&mut configured, &installed.definition) {
                if let Some(i) = world.packages.installed.get_mut(&name) {
                    i.status = Status::HalfConfigured;
                    i.problems = vec![error.to_string()];
                }
                out.push_str(&format!("Configuration failed for {name}: {error}\n"));
                failed.insert(name);
                continue;
            }
            *world = configured;
            if let Some(i) = world.packages.installed.get_mut(&name) {
                i.status = Status::Installed;
                i.problems.clear();
            }
            out.push_str(&format!(
                "Setting up {} ({}) ...\n",
                name, installed.definition.version
            ));
            pending.remove(&name);
            progressed = true;
        }
        if !progressed {
            break;
        }
    }
    Ok(pending.is_empty())
}
pub fn record_cancellation(world: &mut WorldState, p: &Plan, exit_code: i32) {
    world.packages.transactions.push(Transaction {
        id: p.id.clone(),
        operation: p.operation.clone(),
        phase: "CANCELLED".into(),
        phases: vec!["PLANNED".into(), "CANCELLED".into()],
        packages: p
            .install
            .iter()
            .map(|e| e.package.name.clone())
            .chain(p.remove.iter().cloned())
            .collect(),
        at: world.playtime_seconds,
        exit_code,
    });
    if world.packages.transactions.len() > 128 {
        world.packages.transactions.remove(0);
    }
}
pub fn apply(world: &mut WorldState, p: &Plan) -> GameResult<Output> {
    let mut phases = vec!["PLANNED".into()];
    let result = apply_atomic(world, p, &mut phases);
    if result.is_err() {
        let cancelled = archive::jobs::check_cancel().is_err();
        let phase = if cancelled { "CANCELLED" } else { "FAILED" };
        phases.push(phase.into());
        world.packages.transactions.push(Transaction {
            id: p.id.clone(),
            operation: p.operation.clone(),
            phase: phase.into(),
            phases,
            packages: p
                .install
                .iter()
                .map(|e| e.package.name.clone())
                .chain(p.remove.iter().cloned())
                .collect(),
            at: world.playtime_seconds,
            exit_code: if cancelled {
                130
            } else if p.dpkg {
                2
            } else {
                100
            },
        });
        if world.packages.transactions.len() > 128 {
            world.packages.transactions.remove(0);
        }
        world.package_lock = None;
    }
    result
}
fn apply_atomic(world: &mut WorldState, p: &Plan, phases: &mut Vec<String>) -> GameResult<Output> {
    if state::fingerprint(world)? != p.fingerprint {
        return Err(domain(
            "E: Files or package state changed; run the command again",
        ));
    }
    let mut candidate = world.clone();
    let mut out = String::new();
    if p.operation == "install" && !p.dpkg {
        for name in &p.requested {
            if let Some(installed) = candidate.packages.installed.get_mut(name) {
                installed.automatic = false;
            }
        }
    }
    for e in &p.install {
        archive::jobs::check_cancel()?;
        if e.repository == "installed" {
            continue;
        }
        let bytes = if let Some(path) = e.repository.strip_prefix("file:") {
            archive::bytes(&candidate, path, "root")?.as_ref().clone()
        } else if cached(&candidate, e) {
            out.push_str(&format!("Hit: {}\n", e.package.key()));
            archive::bytes(&candidate, &cache_path(&e.package), "root")?
                .as_ref()
                .clone()
        } else {
            phases.push("DOWNLOADING".into());
            out.push_str(&format!(
                "Get: {} {} [{} B]\n",
                e.repository, e.package.name, e.package.download_size
            ));
            repository::artifact(e, &candidate)?
        };
        phases.push("VERIFYING".into());
        if deb::hash(&bytes) != e.checksum || deb::decode(&bytes)? != e.package {
            return Err(domain("E: Package integrity/checksum mismatch"));
        }
        if !p.dpkg {
            archive::write_bytes(
                &mut candidate,
                &cache_path(&e.package),
                bytes,
                "root",
                e.package.download_size,
            )?;
            set_deb_mime(&mut candidate, &cache_path(&e.package));
        }
    }
    for name in &p.remove {
        let old = candidate
            .packages
            .installed
            .get(name)
            .map(|i| i.definition.clone());
        remove(&mut candidate, name, p.purge, false, &mut out)?;
        out.push_str(&format!(
            "{} {name} ...\n",
            if p.purge { "Purging" } else { "Removing" }
        ));
        state::emit(
            &mut candidate,
            &p.id,
            if p.purge {
                "PACKAGE_PURGED"
            } else {
                "PACKAGE_REMOVED"
            },
            old.as_ref(),
            None,
        );
    }
    if !p.install.is_empty() && p.operation != "configure" {
        phases.push("UNPACKING".into());
    }
    for e in &p.install {
        if p.operation != "configure" {
            unpack(
                &mut candidate,
                e,
                p.requested.contains(&e.package.name),
                &mut out,
            )?;
        }
    }
    let names = p
        .install
        .iter()
        .map(|e| e.package.name.clone())
        .collect::<Vec<_>>();
    if !names.is_empty() {
        phases.push("CONFIGURING".into());
    }
    let configured = configure(&mut candidate, &names, &mut out)?;
    if !configured && !p.dpkg {
        return Err(domain(
            "E: Unmet dependencies prevent configuration; transaction rolled back",
        ));
    }
    for name in &names {
        let next = candidate.packages.installed.get(name).cloned();
        if let Some(next) = next {
            let old = world.packages.installed.get(name);
            let event = if next.status != Status::Installed {
                "PACKAGE_BROKEN"
            } else if old
                .is_some_and(|i| i.status == Status::Unpacked || i.status == Status::HalfConfigured)
            {
                "PACKAGE_REPAIRED"
            } else if old.is_some_and(|i| i.definition.version != next.definition.version) {
                "PACKAGE_UPGRADED"
            } else {
                "PACKAGE_INSTALLED"
            };
            state::emit(&mut candidate, &p.id, event, Some(&next.definition), None);
        }
    }
    phases.push("COMMITTING".into());
    state::sync(&mut candidate)?;
    let phase = if configured { "COMPLETED" } else { "FAILED" };
    phases.push(phase.into());
    candidate.packages.transactions.push(Transaction {
        id: p.id.clone(),
        operation: p.operation.clone(),
        phase: phase.into(),
        phases: phases.clone(),
        packages: names.into_iter().chain(p.remove.iter().cloned()).collect(),
        at: world.playtime_seconds,
        exit_code: if configured { 0 } else { 1 },
    });
    if candidate.packages.transactions.len() > 128 {
        candidate.packages.transactions.remove(0);
    }
    archive::jobs::check_cancel()?;
    candidate.package_lock = None;
    candidate.validate()?;
    *world = candidate;
    Ok(Output {
        stdout: out,
        stderr: if configured {
            String::new()
        } else {
            "dpkg: dependency problems prevent configuration; run apt --fix-broken install or dpkg --configure -a\n".into()
        },
        status: if configured { 0 } else { 1 },
        ..Output::default()
    })
}
pub fn set_deb_mime(world: &mut WorldState, path: &str) {
    if let Some(n) = world.vfs.nodes.get_mut(path) {
        n.metadata.insert("mime".into(), deb::MIME.into());
        if let Some(b) = &mut n.blob {
            b.mime = deb::MIME.into();
        }
    }
}
