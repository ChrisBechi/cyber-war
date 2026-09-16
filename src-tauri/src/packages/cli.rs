use super::{deb, model::*, repository, resolver, service, state, version};
use crate::{
    archive,
    error::GameResult,
    terminal_io::{options, Output},
    vfs::domain,
    world::WorldState,
};
use std::collections::BTreeSet;

pub const COMMANDS: &[&str] = &[
    "apt",
    "apt-get",
    "apt-cache",
    "apt-mark",
    "dpkg",
    "dpkg-query",
];
pub fn manual(name: &str) -> Option<String> {
    if !COMMANDS.contains(&name) {
        return None;
    }
    Some(format!("{name}(1) — CYBER WAR virtual packages\n\napt/apt-get: update; search QUERY; show PACKAGE; list [--installed|--upgradable]; install [-y] PACKAGE[=VERSION]...; reinstall [-y] PACKAGE...; remove|purge [-y] PACKAGE...; upgrade|autoremove [-y]; --fix-broken install [-y]; clean; autoclean.\napt-cache: search QUERY; show PACKAGE; policy [PACKAGE].\napt-mark: manual|auto|hold|unhold PACKAGE...; showmanual|showauto|showhold.\ndpkg: -i FILE.deb...; -r|-P PACKAGE...; -l [PATTERN]; -s|-L PACKAGE; -S PATH; --configure -a.\ndpkg-query: -l, -s, -L, -S.\n\nQueries work offline; mutations require virtual root. APT resolves Depends; DPKG leaves missing dependencies unpacked. Remove preserves conffiles; purge removes registered configurations. Modified ordinary files are preserved on removal and block replacement. Upgrades keep local configurations and put new defaults in .dpkg-dist files. -y accepts a valid plan. Ctrl+C cancels atomically. amd64/all virtual profiles only. No host commands, binaries or network.\n"))
}
pub fn failure(name: &str, error: impl std::fmt::Display) -> Output {
    Output {
        stderr: format!("{name}: {error}\n"),
        status: if name.starts_with("apt") { 100 } else { 2 },
        ..Output::default()
    }
}
pub fn execute(
    world: &mut WorldState,
    name: &str,
    args: &[String],
    actor: &str,
) -> Option<GameResult<Output>> {
    COMMANDS.contains(&name).then(|| {
        let mut candidate = world.clone();
        Ok(match run(&mut candidate, name, args, actor) {
            Ok(out) => {
                *world = candidate;
                out
            }
            Err(e) => failure(name, e),
        })
    })
}
fn update(world: &mut WorldState) -> GameResult<Output> {
    let sources = repository::parse_sources(world)?;
    let current = sources
        .iter()
        .map(|s| s.uri.clone())
        .collect::<BTreeSet<_>>();
    let removed = world
        .packages
        .sources
        .difference(&current)
        .cloned()
        .collect::<Vec<_>>();
    let added = current
        .difference(&world.packages.sources)
        .cloned()
        .collect::<Vec<_>>();
    let mut candidate = world.clone();
    candidate
        .packages
        .indexes
        .retain(|repo, _| current.contains(repo));
    let mut out = Output::default();
    let mut visited = BTreeSet::new();
    for (index, source) in sources.iter().enumerate() {
        if !visited.insert(source.uri.clone()) {
            continue;
        }
        let runtime = candidate
            .packages
            .repositories
            .get(&source.uri)
            .cloned()
            .unwrap_or_default();
        let result = (|| -> GameResult<Vec<IndexEntry>> {
            if !candidate.network.connected || !runtime.available {
                return Err(domain("Repository offline"));
            }
            if !runtime.trusted {
                return Err(domain("Untrusted repository"));
            }
            let repo = repository::REPOSITORIES
                .get(&source.uri)
                .ok_or_else(|| domain("Repository not found"))?;
            if repo.suite != source.suite {
                return Err(domain("Suite not found"));
            }
            let components = sources
                .iter()
                .filter(|s| s.uri == source.uri && s.suite == source.suite)
                .flat_map(|s| s.components.iter())
                .collect::<BTreeSet<_>>();
            if components
                .iter()
                .any(|c| !["main", "contrib", "non-free"].contains(&c.as_str()))
            {
                return Err(domain("Component not found"));
            }
            let entries = repo
                .entries
                .iter()
                .filter(|(release, e)| {
                    *release <= runtime.release && components.contains(&e.component)
                })
                .map(|(_, e)| e.clone())
                .collect::<Vec<_>>();
            for entry in &entries {
                deb::validate(&entry.package)?;
            }
            Ok(entries)
        })();
        match result {
            Ok(entries) => {
                let same = candidate
                    .packages
                    .indexes
                    .get(&source.uri)
                    .is_some_and(|old| {
                        old.len() == entries.len()
                            && old
                                .iter()
                                .zip(&entries)
                                .all(|(a, b)| a.checksum == b.checksum)
                    });
                out.stdout.push_str(&format!(
                    "{}:{} {} {} InRelease\n",
                    if same { "Hit" } else { "Get" },
                    index + 1,
                    source.uri,
                    source.suite
                ));
                if !same {
                    out.stdout.push_str(&format!(
                        "Get:{} {} {}/main amd64 Packages [{} entries]\n",
                        index + 1,
                        source.uri,
                        source.suite,
                        entries.len()
                    ));
                }
                candidate
                    .packages
                    .indexes
                    .insert(source.uri.clone(), entries);
            }
            Err(error) => {
                out.stderr
                    .push_str(&format!("Err:{} {} {}\n", index + 1, source.uri, error));
                out.status = 100;
            }
        }
    }
    let transaction = uuid::Uuid::new_v4().to_string();
    for repo in added {
        state::emit(
            &mut candidate,
            &transaction,
            "REPOSITORY_ADDED",
            None,
            Some(&repo),
        );
    }
    for repo in removed {
        state::emit(
            &mut candidate,
            &transaction,
            "REPOSITORY_REMOVED",
            None,
            Some(&repo),
        );
    }
    candidate.packages.sources = current;
    state::emit(&mut candidate, &transaction, "APT_UPDATED", None, None);
    state::sync(&mut candidate)?;
    out.stdout.push_str("Reading package lists... Done\n");
    *world = candidate;
    Ok(out)
}
pub fn query(
    world: &WorldState,
    command: &str,
    values: &[String],
    installed_only: bool,
    upgradable: bool,
) -> GameResult<String> {
    let entries = resolver::candidates(&world.packages);
    let name = values.first().map(String::as_str).unwrap_or("");
    match command {
        "search" => {
            if name.is_empty() {
                return Err(domain("usage: apt search QUERY"));
            }
            let query = values.join(" ").to_lowercase();
            let mut results = entries
                .iter()
                .filter_map(|e| {
                    let p = &e.package;
                    let haystack = format!(
                        "{} {} {} {}",
                        p.name,
                        p.description,
                        p.section,
                        p.keywords.join(" ")
                    )
                    .to_lowercase();
                    query
                        .split_whitespace()
                        .all(|part| haystack.contains(part))
                        .then_some((
                            if p.name == query {
                                0
                            } else if p.name.starts_with(&query) {
                                1
                            } else {
                                2
                            },
                            e,
                        ))
                })
                .collect::<Vec<_>>();
            results.sort_by(|a, b| {
                a.0.cmp(&b.0)
                    .then_with(|| a.1.package.name.cmp(&b.1.package.name))
            });
            let mut seen = BTreeSet::new();
            Ok(results
                .into_iter()
                .filter(|(_, e)| seen.insert(e.package.name.clone()))
                .map(|(_, e)| {
                    format!(
                        "{}/{} {} {}\n  {}\n",
                        e.package.name,
                        e.suite,
                        e.package.version,
                        e.package.architecture,
                        e.package.description
                    )
                })
                .collect())
        }
        "show" => {
            let p = entries
                .iter()
                .find(|e| e.package.name == name)
                .map(|e| &e.package)
                .or_else(|| world.packages.installed.get(name).map(|i| &i.definition))
                .ok_or_else(|| domain(format!("E: Unable to locate package {name}")))?;
            Ok(deb::control(p))
        }
        "list" => {
            let mut names = entries
                .iter()
                .map(|e| e.package.name.clone())
                .collect::<BTreeSet<_>>();
            names.extend(world.packages.installed.keys().cloned());
            let mut out = String::from("Listing...\n");
            for name in names {
                let installed = world
                    .packages
                    .installed
                    .get(&name)
                    .filter(|i| i.status == Status::Installed);
                let candidate = entries.iter().find(|e| e.package.name == name);
                let upgrade = installed.zip(candidate).is_some_and(|(i, e)| {
                    version::compare(&e.package.version, &i.definition.version).is_gt()
                });
                if installed_only && installed.is_none()
                    || upgradable && !upgrade
                    || !values.is_empty() && !name.contains(&values[0])
                {
                    continue;
                }
                let p = if installed_only {
                    installed.map(|i| &i.definition)
                } else {
                    candidate
                        .map(|e| &e.package)
                        .or_else(|| installed.map(|i| &i.definition))
                };
                if let Some(p) = p {
                    out.push_str(&format!(
                        "{}/{} {} {}{}\n",
                        name,
                        candidate.map_or("local", |e| &e.suite),
                        p.version,
                        p.architecture,
                        if upgrade {
                            " [upgradable]"
                        } else if installed.is_some() {
                            " [installed]"
                        } else {
                            ""
                        }
                    ));
                }
            }
            Ok(out)
        }
        "policy" => Ok(entries
            .iter()
            .filter(|e| name.is_empty() || e.package.name == name)
            .map(|e| {
                format!(
                    "{}: {}\n  500 {} {}\n",
                    e.package.name, e.package.version, e.repository, e.suite
                )
            })
            .collect()),
        _ => Err(domain("E: Unsupported package query")),
    }
}
fn dpkg_query(world: &WorldState, command: &str, values: &[String]) -> GameResult<String> {
    let name = values.first().map(String::as_str).unwrap_or("");
    match command {
        "-l" => Ok(world
            .packages
            .installed
            .values()
            .filter(|i| name.is_empty() || i.definition.name.contains(name))
            .map(|i| {
                format!(
                    "{}  {:20} {:12} {} {}\n",
                    match i.status {
                        Status::Installed => "ii",
                        Status::Unpacked => "iU",
                        Status::HalfConfigured => "iF",
                        Status::ConfigFiles => "rc",
                    },
                    i.definition.name,
                    i.definition.version,
                    i.definition.architecture,
                    i.definition.description
                )
            })
            .collect()),
        "-s" => {
            let i = world
                .packages
                .installed
                .get(name)
                .ok_or_else(|| domain(format!("Package {name} is not installed")))?;
            Ok(format!(
                "{}Status: {} ok {}\n",
                deb::control(&i.definition),
                if i.status == Status::ConfigFiles {
                    "deinstall"
                } else {
                    "install"
                },
                i.status.label()
            ))
        }
        "-L" => {
            if !world.packages.installed.contains_key(name) {
                return Err(domain(format!("Package {name} is not installed")));
            }
            Ok(world
                .packages
                .ownership
                .iter()
                .filter(|(_, o)| o.owners.contains(name))
                .map(|(path, _)| format!("{path}\n"))
                .collect())
        }
        "-S" => {
            let matches = world
                .packages
                .ownership
                .iter()
                .filter(|(path, _)| *path == name || path.ends_with(&format!("/{name}")))
                .map(|(path, o)| {
                    format!(
                        "{}: {path}\n",
                        o.owners.iter().cloned().collect::<Vec<_>>().join(", ")
                    )
                })
                .collect::<String>();
            if matches.is_empty() {
                Err(domain(format!("no path found matching {name}")))
            } else {
                Ok(matches)
            }
        }
        _ => Err(domain("dpkg: unsupported option")),
    }
}
fn run(world: &mut WorldState, name: &str, args: &[String], actor: &str) -> GameResult<Output> {
    if world.terminal.host.is_some() {
        return Err(domain(
            "remote package inventory is not implemented; local system unchanged",
        ));
    }
    if args == ["--help"] || args == ["-h"] {
        return Ok(Output::success(manual(name).unwrap_or_default()));
    }
    if args == ["--version"] || args == ["-v"] {
        return Ok(Output::success(format!(
            "{name} CYBER WAR virtual packages 1.0\n"
        )));
    }
    let dpkg = name.starts_with("dpkg");
    let (command, values, yes, installed, upgradable) = if dpkg {
        let command = args
            .first()
            .ok_or_else(|| domain("usage: dpkg ACTION [PACKAGE|FILE]"))?
            .as_str();
        let values = args[1..].to_vec();
        if ["-l", "-s", "-L", "-S"].contains(&command) {
            return Ok(Output::success(dpkg_query(world, command, &values)?));
        }
        if name == "dpkg-query" {
            return Err(domain("dpkg-query supports -l, -s, -L and -S"));
        }
        let command = match command {
            "-i" => "install",
            "-r" => "remove",
            "-P" => "purge",
            "--configure" if values == ["-a"] => "configure",
            _ => return Err(domain("dpkg: unsupported option")),
        };
        (
            command.to_string(),
            if command == "configure" {
                Vec::new()
            } else {
                values
            },
            true,
            false,
            false,
        )
    } else {
        let parsed = options(
            name,
            args,
            "yfiur",
            "",
            &[
                ("assume-yes", 'y'),
                ("fix-broken", 'f'),
                ("installed", 'i'),
                ("upgradable", 'u'),
                ("reinstall", 'r'),
            ],
        )?;
        let mut command = parsed
            .files
            .first()
            .ok_or_else(|| domain("missing package command"))?
            .clone();
        if (parsed.has('i') || parsed.has('u')) && command != "list" {
            return Err(domain("--installed and --upgradable require list"));
        }
        if name == "apt-cache" && !["search", "show", "policy"].contains(&command.as_str()) {
            return Err(domain("apt-cache supports search, show and policy"));
        }
        if name == "apt-mark"
            && ![
                "manual",
                "auto",
                "hold",
                "unhold",
                "showmanual",
                "showauto",
                "showhold",
            ]
            .contains(&command.as_str())
        {
            return Err(domain("apt-mark: unsupported action"));
        }
        if parsed.has('f') {
            if command != "install" {
                return Err(domain("--fix-broken requires install"));
            }
            command = "repair".into();
        }
        if parsed.has('r') {
            if command != "install" {
                return Err(domain("--reinstall requires install"));
            }
            command = "reinstall".into();
        }
        (
            command,
            parsed.files[1..].to_vec(),
            parsed.has('y'),
            parsed.has('i'),
            parsed.has('u'),
        )
    };
    if ["search", "show", "list", "policy"].contains(&command.as_str()) {
        return Ok(Output::success(query(
            world, &command, &values, installed, upgradable,
        )?));
    }
    if name == "apt-cache" {
        return Err(domain("apt-cache supports search, show and policy"));
    }
    if name == "apt-mark" && ["showmanual", "showauto", "showhold"].contains(&command.as_str()) {
        return Ok(Output::success(
            world
                .packages
                .installed
                .values()
                .filter(|i| match command.as_str() {
                    "showhold" => i.held,
                    "showauto" => i.automatic,
                    _ => !i.automatic,
                })
                .map(|i| format!("{}\n", i.definition.name))
                .collect(),
        ));
    }
    if actor != "root" {
        return Err(domain("E: Could not open package lock: Permission denied"));
    }
    if world.package_lock.is_some() {
        return Err(domain("E: Could not get lock /var/lib/dpkg/lock-frontend"));
    }
    if command == "update" {
        if !values.is_empty() {
            return Err(domain("apt update takes no operands"));
        }
        return update(world);
    }
    if name == "apt-mark" {
        if !["manual", "auto", "hold", "unhold"].contains(&command.as_str()) || values.is_empty() {
            return Err(domain("apt-mark: invalid action"));
        }
        if values
            .iter()
            .any(|v| !world.packages.installed.contains_key(v))
        {
            return Err(domain("apt-mark: package not installed"));
        }
        for value in values {
            if let Some(i) = world.packages.installed.get_mut(&value) {
                match command.as_str() {
                    "manual" => i.automatic = false,
                    "auto" => i.automatic = true,
                    "hold" => i.held = true,
                    _ => i.held = false,
                }
            }
        }
        state::sync(world)?;
        return Ok(Output::default());
    }
    if command == "clean" || command == "autoclean" {
        let valid = resolver::candidates(&world.packages)
            .iter()
            .map(|e| service::cache_path(&e.package))
            .collect::<BTreeSet<_>>();
        let files = world
            .vfs
            .nodes
            .keys()
            .filter(|p| {
                p.starts_with("/var/cache/apt/archives/")
                    && p.ends_with(".deb")
                    && (command == "clean" || !valid.contains(*p))
            })
            .cloned()
            .collect::<Vec<_>>();
        for path in files {
            world.vfs.remove(&path, "root", false)?;
        }
        return Ok(Output::default());
    }
    if ["install", "reinstall", "remove", "purge"].contains(&command.as_str()) && values.is_empty()
    {
        return Err(domain("package or file operand required"));
    }
    let plan = service::plan(world, &command, &values, dpkg)?;
    let summary = service::summary(&plan);
    if plan.install.is_empty() && plan.remove.is_empty() {
        if command == "install" && !dpkg {
            for name in &plan.requested {
                if let Some(i) = world.packages.installed.get_mut(name) {
                    i.automatic = false;
                }
            }
            state::sync(world)?;
        }
        return Ok(Output::success("0 packages changed.\n".into()));
    }
    let pending = Pending {
        plan,
        actor: actor.into(),
    };
    if !yes {
        if !archive::jobs::may_defer() || world.terminal.shell_depth > 0 {
            return Err(domain(
                "E: Confirmation requires foreground terminal; use -y",
            ));
        }
        world.package_lock = Some(pending.plan.id.clone());
        world.terminal.package_pending = Some(pending);
        Ok(Output::success(summary))
    } else {
        start(world, pending, summary)
    }
}
pub fn start(world: &mut WorldState, pending: Pending, stdout: String) -> GameResult<Output> {
    if pending.actor != "root" {
        return Err(domain("Permission denied"));
    }
    if archive::jobs::may_defer() && world.terminal.shell_depth == 0 {
        world.package_lock = Some(pending.plan.id.clone());
        let job = archive::jobs::enqueue(
            world,
            archive::jobs::Work::Package(pending.plan.clone()),
            format!("package {}", pending.plan.operation),
            pending.plan.download_size.max(16 * 1024 * 1024),
            false,
        )?;
        world.terminal.package_job = Some(job.id);
        Ok(Output {
            stdout,
            archive_job: Some(job.id),
            ..Output::default()
        })
    } else {
        Ok(service::apply(world, &pending.plan)
            .unwrap_or_else(|error| failure(if pending.plan.dpkg { "dpkg" } else { "apt" }, error)))
    }
}
pub fn respond(world: &mut WorldState, input: &str, cancel: bool) -> GameResult<Output> {
    let pending = world
        .terminal
        .package_pending
        .take()
        .ok_or_else(|| domain("no package confirmation pending"))?;
    world.package_lock = None;
    if cancel || matches!(input.trim().to_ascii_lowercase().as_str(), "n" | "no") {
        service::record_cancellation(world, &pending.plan, if cancel { 130 } else { 1 });
        return Ok(Output {
            stdout: "Abort.\n".into(),
            status: if cancel { 130 } else { 1 },
            ..Output::default()
        });
    }
    if !matches!(input.trim().to_ascii_lowercase().as_str(), "" | "y" | "yes") {
        world.package_lock = Some(pending.plan.id.clone());
        world.terminal.package_pending = Some(pending);
        return Ok(Output::success("Please answer Y or n.\n".into()));
    }
    if state::fingerprint(world)? != pending.plan.fingerprint {
        return Err(domain(
            "E: Files changed while waiting; run the command again",
        ));
    }
    start(world, pending, String::new())
}
