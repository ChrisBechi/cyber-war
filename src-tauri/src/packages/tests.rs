use super::*;
use crate::{archive, terminal, world::WorldState};
use model::Status;
fn world() -> WorldState {
    let mut w = WorldState::new("kali", "pc").unwrap();
    w.network.connected = true;
    w
}
fn command(w: &mut WorldState, line: &str) -> terminal::CommandResult {
    archive::jobs::with_deferral(false, || terminal::execute(w, line))
}
fn ok(w: &mut WorldState, line: &str) -> String {
    let r = command(w, line);
    assert_eq!(r.exit_code, 0, "{line}: {}", r.stderr);
    r.stdout
}
#[test]
fn install_remove_purge_and_autoremove_are_real_vfs_changes() {
    let mut w = world();
    assert!(!w.vfs.nodes.contains_key("/usr/bin/netscan"));
    ok(&mut w, "sudo apt update");
    assert!(ok(&mut w, "apt search netscan").contains("2.4.1"));
    ok(&mut w, "sudo apt install -y netscan");
    assert_eq!(w.packages.installed["netscan"].status, Status::Installed);
    assert!(!w.packages.installed["netscan"].automatic);
    assert!(w.packages.installed["libpacket2"].automatic);
    assert!(w.packages.installed["cyber-core"].automatic);
    assert_eq!(ok(&mut w, "which netscan"), "/usr/bin/netscan\n");
    assert!(ok(&mut w, "netscan --version").contains("2.4.1"));
    assert!(ok(&mut w, "dpkg -S /usr/bin/netscan").contains("netscan:"));
    assert!(ok(&mut w, "dpkg -L netscan").contains("/etc/netscan"));
    assert!(ok(&mut w, "man netscan").contains("VIRTUAL_TARGET"));
    w.vfs
        .write("/etc/netscan/netscan.conf", "player=true\n", "root")
        .unwrap();
    ok(&mut w, "sudo apt remove -y netscan");
    assert!(!w.vfs.nodes.contains_key("/usr/bin/netscan"));
    assert_eq!(w.packages.installed["netscan"].status, Status::ConfigFiles);
    assert_eq!(
        w.vfs.read("/etc/netscan/netscan.conf", "root").unwrap(),
        "player=true\n"
    );
    assert!(ok(&mut w, "which netscan").is_empty());
    ok(&mut w, "sudo apt purge -y netscan");
    assert!(!w.vfs.nodes.contains_key("/etc/netscan/netscan.conf"));
    ok(&mut w, "sudo apt autoremove -y");
    assert!(!w.packages.installed.contains_key("libpacket2"));
    w.validate().unwrap();
}
#[test]
fn offline_deb_preserves_broken_state_then_repairs() {
    let mut w = world();
    let p = repository::REPOSITORIES[repository::OFFICIAL]
        .entries
        .iter()
        .find(|(_, e)| e.package.name == "netscan")
        .unwrap()
        .1
        .package
        .clone();
    let bytes = deb::encode(&p).unwrap();
    assert_eq!(deb::decode(&bytes).unwrap(), p);
    archive::write_bytes(
        &mut w,
        "/home/kali/Downloads/netscan.deb",
        bytes,
        "kali",
        p.download_size,
    )
    .unwrap();
    assert!(ok(&mut w, "file Downloads/netscan.deb").contains("Debian binary package"));
    let output = command(&mut w, "sudo dpkg -i Downloads/netscan.deb");
    assert_eq!(output.exit_code, 1, "{}", output.stderr);
    assert_eq!(w.packages.installed["netscan"].status, Status::Unpacked);
    assert!(w.vfs.nodes.contains_key("/usr/bin/netscan"));
    assert_ne!(command(&mut w, "netscan --version").exit_code, 0);
    ok(&mut w, "sudo apt update");
    ok(&mut w, "sudo apt --fix-broken install -y");
    assert_eq!(w.packages.installed["netscan"].status, Status::Installed);
    assert!(w
        .packages
        .events
        .iter()
        .any(|e| e.kind == "PACKAGE_REPAIRED"));
}
#[test]
fn repository_discovery_removal_and_upgrade_preserve_config() {
    let mut w = world();
    ok(&mut w, "sudo apt update");
    assert!(ok(&mut w, "apt search iot-discovery").is_empty());
    ok(&mut w,"echo 'deb https://repo.blackwire.net/tools stable main' | sudo tee /etc/apt/sources.list.d/blackwire.list");
    ok(&mut w, "sudo apt update");
    assert!(ok(&mut w, "apt search iot-discovery").contains("iot-discovery"));
    ok(&mut w, "sudo apt install -y iot-discovery");
    w.vfs
        .remove("/etc/apt/sources.list.d/blackwire.list", "root", false)
        .unwrap();
    ok(&mut w, "sudo apt update");
    assert!(ok(&mut w, "apt search iot-discovery").is_empty());
    assert_eq!(
        w.packages.installed["iot-discovery"].status,
        Status::Installed
    );
    ok(&mut w, "sudo apt install -y netscan=2.3.0");
    w.vfs
        .write("/etc/netscan/netscan.conf", "custom=true\n", "root")
        .unwrap();
    w.packages.repositories.insert(
        repository::OFFICIAL.into(),
        model::RepositoryState {
            release: 2,
            ..Default::default()
        },
    );
    ok(&mut w, "sudo apt update");
    assert!(ok(&mut w, "apt list --upgradable").contains("netscan"));
    ok(&mut w, "sudo apt upgrade -y");
    assert_eq!(w.packages.installed["netscan"].definition.version, "2.5.0");
    assert_eq!(
        w.vfs.read("/etc/netscan/netscan.conf", "root").unwrap(),
        "custom=true\n"
    );
}
#[test]
fn no_space_and_bad_packages_do_not_modify_state() {
    let mut w = world();
    ok(&mut w, "sudo apt update");
    w.vfs.capacity_bytes = w.vfs.used_bytes() + 1024;
    let before = serde_json::to_string(&w.packages).unwrap();
    assert_ne!(command(&mut w, "sudo apt install -y netscan").exit_code, 0);
    assert_eq!(serde_json::to_string(&w.packages).unwrap(), before);
    let mut p = repository::tool("evil-tool", "1.0", "test", "netscan");
    for path in [
        "/../../escape",
        "C:/Windows/file",
        "/usr/bin/../escape",
        "/var/lib/dpkg/status",
    ] {
        p.files[0].path = path.into();
        assert!(deb::encode(&p).is_err(), "{path}");
    }
    assert!(deb::decode(b"!<arch>\ntruncated").is_err());
    assert!(w
        .vfs
        .write("/var/lib/dpkg/status", "tampered", "root")
        .is_err());
}
#[test]
fn pipelines_redirection_permissions_and_completion() {
    let mut w = world();
    ok(&mut w, "sudo apt update");
    assert_eq!(command(&mut w, "apt install -y netscan").exit_code, 100);
    assert!(ok(&mut w, "apt list --installed | grep nmap").contains("nmap"));
    ok(&mut w, "apt search wireless > results.txt");
    assert!(w
        .vfs
        .read("/home/kali/results.txt", "kali")
        .unwrap()
        .contains("wireless"));
    let completed = crate::completion::complete(&w, "apt install net", 15).unwrap();
    assert!(completed.candidates.contains(&"netscan".to_string()));
}

fn entry(p: model::Definition) -> model::IndexEntry {
    model::IndexEntry {
        package: p,
        repository: "fixture".into(),
        suite: "stable".into(),
        component: "main".into(),
        checksum: String::new(),
    }
}
fn local(w: &mut WorldState, p: &model::Definition) -> String {
    let path = format!("/home/kali/Downloads/{}.deb", p.key());
    archive::write_bytes(w, &path, deb::encode(p).unwrap(), "kali", p.download_size).unwrap();
    path
}
#[test]
fn solver_cycles_alternatives_providers_versions_and_conflicts() {
    let mut a = repository::definition("alpha", "1.0", "Alpha");
    let mut b = repository::definition("bravo", "2.0", "Bravo");
    a.depends = vec![vec![repository::relation("bravo", ">=", "2.0")]];
    b.depends = vec![vec![repository::relation("alpha", "", "")]];
    let mut s = model::PackageState::default();
    s.indexes
        .insert("fixture".into(), vec![entry(a.clone()), entry(b.clone())]);
    assert!(resolver::resolve(&s, &["alpha".into()], false)
        .unwrap_err()
        .to_string()
        .contains("cycle"));
    let mut c = repository::definition("charlie", "1.0", "Charlie");
    c.provides = vec![repository::relation("virtual-packets", "=", "2.0")];
    a.depends[0].push(repository::relation("virtual-packets", ">=", "2.0"));
    s.indexes.insert(
        "fixture".into(),
        vec![entry(a.clone()), entry(b), entry(c.clone())],
    );
    let resolved = resolver::resolve(&s, &["alpha".into()], false).unwrap();
    assert_eq!(
        resolved
            .iter()
            .map(|e| e.package.name.as_str())
            .collect::<Vec<_>>(),
        ["charlie", "alpha"]
    );
    c.conflicts = vec![repository::relation("alpha", "", "")];
    s.indexes.insert("fixture".into(), vec![entry(a), entry(c)]);
    assert!(resolver::resolve(&s, &["alpha".into()], false)
        .unwrap_err()
        .to_string()
        .contains("conflicts"));
    assert!(resolver::resolve(&s, &["alpha=99".into()], false).is_err());
}
#[test]
fn manual_marks_hold_orphans_and_reverse_dependencies() {
    let mut w = world();
    ok(&mut w, "sudo apt update");
    ok(&mut w, "sudo apt install -y netscan");
    assert_eq!(
        command(&mut w, "sudo apt remove -y libpacket2").exit_code,
        100
    );
    ok(&mut w, "sudo apt-mark hold netscan");
    assert_eq!(command(&mut w, "sudo apt purge -y netscan").exit_code, 100);
    ok(&mut w, "sudo apt-mark unhold netscan");
    ok(&mut w, "sudo apt install -y libpacket2");
    assert!(!w.packages.installed["libpacket2"].automatic);
    ok(&mut w, "sudo apt remove -y netscan");
    ok(&mut w, "sudo apt autoremove -y");
    assert!(w.packages.installed.contains_key("libpacket2"));
    ok(&mut w, "sudo apt-mark auto libpacket2");
    ok(&mut w, "sudo apt autoremove -y");
    assert!(!w.packages.installed.contains_key("libpacket2"));
    assert_eq!(w.packages.installed["netscan"].status, Status::ConfigFiles);
}
#[test]
fn collisions_replaces_actions_and_half_configured_repair() {
    let mut w = world();
    let mut a = repository::definition("alpha", "1.0", "Alpha");
    a.files = vec![repository::file(
        "/usr/share/collision.txt",
        "alpha",
        5,
        false,
        None,
    )];
    let mut b = a.clone();
    b.name = "bravo".into();
    b.files[0].content = "bravo".into();
    let pa = local(&mut w, &a);
    let pb = local(&mut w, &b);
    assert_eq!(
        command(&mut w, &format!("sudo dpkg -i {pa} {pb}")).exit_code,
        2
    );
    assert!(!w.vfs.nodes.contains_key("/usr/share/collision.txt"));
    ok(&mut w, &format!("sudo dpkg -i {pa}"));
    assert_eq!(command(&mut w, &format!("sudo dpkg -i {pb}")).exit_code, 2);
    b.replaces.push(repository::relation("alpha", "", ""));
    local(&mut w, &b);
    ok(&mut w, &format!("sudo dpkg -i {pb}"));
    ok(&mut w, "sudo dpkg -P alpha");
    assert_eq!(
        w.vfs.read("/usr/share/collision.txt", "kali").unwrap(),
        "bravo"
    );
    let mut c = repository::definition("charlie", "1.0", "Charlie");
    c.actions = vec![model::Action::DefaultFile {
        path: "/var/lib/charlie/cache/state".into(),
        content: "ready".into(),
    }];
    state::mkdirs(&mut w, "/var/lib/charlie").unwrap();
    w.vfs
        .write("/var/lib/charlie/cache", "collision", "root")
        .unwrap();
    let pc = local(&mut w, &c);
    assert_eq!(command(&mut w, &format!("sudo dpkg -i {pc}")).exit_code, 1);
    assert_eq!(
        w.packages.installed["charlie"].status,
        Status::HalfConfigured
    );
    w.vfs
        .remove("/var/lib/charlie/cache", "root", false)
        .unwrap();
    ok(&mut w, "sudo dpkg --configure -a");
    assert_eq!(w.packages.installed["charlie"].status, Status::Installed);
    ok(&mut w, "sudo dpkg -P charlie");
    assert!(!w.vfs.nodes.contains_key("/var/lib/charlie/cache/state"));
}
#[test]
fn confirmation_lock_cancel_stale_plan_and_terminal_close() {
    let mut w = world();
    ok(&mut w, "sudo apt update");
    let before = w.vfs.clone();
    let r = terminal::execute(&mut w, "sudo apt install netscan");
    assert!(r.archive_prompt.is_some());
    assert_eq!(
        command(&mut w, "sudo apt install -y wireless-utils").exit_code,
        100
    );
    assert_eq!(archive::ipc::input(&mut w, "", true).exit_code, 130);
    assert!(w.package_lock.is_none());
    assert_eq!(w.vfs, before);
    assert_eq!(w.terminal.last_status, 130);
    assert_eq!(w.packages.transactions.last().unwrap().phase, "CANCELLED");
    let r = terminal::execute(&mut w, "sudo apt install -y netscan");
    let id = r.archive_job.unwrap();
    archive::jobs::cancel(id);
    assert_eq!(
        archive::jobs::tick(&mut w)
            .iter()
            .find(|j| j.id == id)
            .unwrap()
            .exit_code,
        130
    );
    assert!(w.package_lock.is_none());
    assert_eq!(w.vfs, before);
    terminal::execute(&mut w, "sudo apt install netscan");
    w.vfs.write("/home/kali/change", "changed", "kali").unwrap();
    assert_ne!(archive::ipc::input(&mut w, "y", false).exit_code, 0);
    assert!(w.package_lock.is_none());
    crate::terminal_sessions::open(&mut w, "package-test").unwrap();
    crate::terminal_sessions::with_session(&mut w, Some("package-test"), |w| {
        Ok(terminal::execute(w, "sudo apt install netscan"))
    })
    .unwrap();
    assert!(w.package_lock.is_some());
    crate::terminal_sessions::close(&mut w, "package-test").unwrap();
    assert!(w.package_lock.is_none());
}
#[test]
fn cache_offline_integrity_partial_repositories_and_download_names() {
    let mut w = world();
    ok(&mut w, "sudo apt update");
    ok(&mut w, "sudo apt install -y netscan");
    w.network.connected = false;
    ok(&mut w, "sudo apt reinstall -y netscan");
    let cache = service::cache_path(&w.packages.installed["netscan"].definition);
    w.vfs.write(&cache, "tampered", "root").unwrap();
    let before = w.vfs.clone();
    assert_eq!(
        command(&mut w, "sudo apt reinstall -y netscan").exit_code,
        100
    );
    assert_eq!(w.vfs, before);
    w.network.connected = true;
    ok(&mut w, "sudo apt clean");
    assert!(!w.vfs.nodes.contains_key(&cache));
    w.vfs
        .write(
            "/etc/apt/sources.list.d/missing.list",
            "deb https://unknown.game/repo stable main\n",
            "root",
        )
        .unwrap();
    assert_eq!(command(&mut w, "sudo apt update").exit_code, 100);
    assert!(!w.packages.indexes[repository::OFFICIAL].is_empty());
    let e = repository::REPOSITORIES[repository::OFFICIAL]
        .entries
        .iter()
        .find(|(_, e)| e.package.name == "netscan")
        .unwrap()
        .1
        .clone();
    let url = format!("{}/pool/{}.deb", e.repository, e.package.key());
    let page = serde_json::to_value(
        crate::browser::navigate(&mut w, "https://www.mirror.kali.game/kali").unwrap(),
    )
    .unwrap();
    assert!(page["body"].as_str().unwrap().contains("netscan"));
    let page = serde_json::to_value(
        crate::browser::navigate(&mut w, &url.replace("https://", "https://www.")).unwrap(),
    )
    .unwrap();
    assert!(page["action"]
        .as_str()
        .unwrap()
        .starts_with("package-download:"));
    let first = ipc::download(&mut w, &url, "/home/kali/Downloads/test.deb", "kali").unwrap();
    let second = ipc::download(&mut w, &url, "/home/kali/Downloads/test.deb", "kali").unwrap();
    assert_ne!(first, second);
    assert!(second.contains("(1)"));
    let info = serde_json::to_value(ipc::inspect(&w, &first).unwrap()).unwrap();
    assert_eq!(info["status"], "installed");
    assert_eq!(info["name"], "netscan");
}
#[test]
fn five_slots_checkpoint_blob_persistence_and_sql_rollback() {
    let mut game =
        crate::service::GameService::new(rusqlite::Connection::open_in_memory().unwrap()).unwrap();
    for slot in 1..=5 {
        game.new_game(slot, "kali", "pc", false).unwrap();
        game.end_session().unwrap();
        game.start_session().unwrap();
        if slot % 2 == 1 {
            game.mutate(
                |_, w, _| {
                    w.network.connected = true;
                    ok(w, "sudo apt update");
                    ok(w, "sudo apt install -y netscan");
                    Ok(())
                },
                true,
            )
            .unwrap();
        }
    }
    for slot in 1..=5 {
        game.load(slot, true).unwrap();
        assert_eq!(
            game.world()
                .unwrap()
                .packages
                .installed
                .contains_key("netscan"),
            slot % 2 == 1
        );
        if slot % 2 == 1 {
            assert!(archive::bytes(
                game.world().unwrap(),
                "/var/cache/apt/archives/netscan_2.4.1_amd64.deb",
                "root"
            )
            .is_ok());
        }
    }
    game.load(1, true).unwrap();
    game.save(false).unwrap();
    let checkpoint = crate::save::checkpoints(&game.connection, 1).unwrap()[0]
        .id
        .clone();
    game.mutate(
        |_, w, _| {
            ok(w, "sudo apt purge -y netscan");
            Ok(())
        },
        false,
    )
    .unwrap();
    game.restore(&checkpoint).unwrap();
    assert!(game
        .world()
        .unwrap()
        .packages
        .installed
        .contains_key("netscan"));
    let before = crate::save::encode(game.world().unwrap()).unwrap().0;
    game.connection.execute_batch("CREATE TRIGGER reject_packages BEFORE UPDATE ON save_slots BEGIN SELECT RAISE(ABORT,'test'); END;").unwrap();
    assert!(game
        .mutate(
            |_, w, _| {
                ok(w, "sudo apt purge -y netscan");
                Ok(())
            },
            false
        )
        .is_err());
    assert_eq!(
        crate::save::encode(game.world().unwrap()).unwrap().0,
        before
    );
}
#[test]
fn legacy_schema_migrates_once_and_security_rejects_unsafe_metadata() {
    let w = world();
    let mut json = serde_json::to_value(&w).unwrap();
    json.as_object_mut().unwrap().remove("packages");
    json["settings"]["aptInstalled"] = serde_json::json!("[\"nmap\",\"aircrack-ng\"]");
    let mut old: WorldState = serde_json::from_value(json).unwrap();
    state::initialize(&mut old).unwrap();
    old.validate().unwrap();
    assert!(old.packages.installed.contains_key("aircrack-ng"));
    let before = serde_json::to_string(&old.packages).unwrap();
    state::initialize(&mut old).unwrap();
    assert_eq!(serde_json::to_string(&old.packages).unwrap(), before);
    let mut p = repository::definition("malicious", "1.0", "Fixture");
    p.architecture = "arm64".into();
    assert!(deb::encode(&p).is_err());
    p.architecture = "all".into();
    p.files[0].kind = "symlink".into();
    p.files[0].content = "../../../../host".into();
    assert!(deb::encode(&p).is_err());
    p.files.clear();
    p.actions = vec![model::Action::EnsureDirectory {
        path: "/var/lib/malicious/../../dpkg".into(),
    }];
    assert!(deb::encode(&p).is_err());
    assert!(
        serde_json::from_str::<model::Action>(r#"{"kind":"run","command":"powershell"}"#).is_err()
    );
    assert!(serde_json::from_str::<model::Action>(
        r#"{"kind":"ensureDirectory","path":"/var/lib/a/b","command":"sh"}"#
    )
    .is_err());
    for repo in repository::REPOSITORIES.values() {
        for (_, e) in &repo.entries {
            assert_eq!(
                deb::decode(&deb::encode(&e.package).unwrap()).unwrap(),
                e.package
            );
        }
    }
}

#[test]
#[ignore = "run separately to measure without concurrent regression tests"]
fn performance_catalog_sizes() {
    use std::time::Instant;
    for count in [100, 1000, 5000, 10000] {
        let mut w = world();
        w.packages.indexes.insert(
            "benchmark".into(),
            (0..count)
                .map(|i| {
                    entry(repository::definition(
                        &format!("bench-{i:05}"),
                        "1.0",
                        "Benchmark network package",
                    ))
                })
                .collect(),
        );
        let mut lookup = Vec::new();
        let database = w.packages.indexes["benchmark"]
            .iter()
            .map(|e| (e.package.name.clone(), e.package.clone()))
            .collect::<std::collections::BTreeMap<_, _>>();
        let mut search = Vec::new();
        let mut solve = Vec::new();
        for _ in 0..21 {
            let start = Instant::now();
            let _ = std::hint::black_box(database.get("bench-00099"));
            lookup.push(start.elapsed().as_secs_f64() * 1000.0);
            let start = Instant::now();
            let result = cli::execute(
                &mut w,
                "apt",
                &["search".into(), "bench-09999".into()],
                "kali",
            )
            .unwrap()
            .unwrap();
            std::hint::black_box(result);
            search.push(start.elapsed().as_secs_f64() * 1000.0);
            let start = Instant::now();
            resolver::resolve(&w.packages, &["bench-00099".into()], false).unwrap();
            solve.push(start.elapsed().as_secs_f64() * 1000.0);
        }
        lookup.sort_by(f64::total_cmp);
        search.sort_by(f64::total_cmp);
        solve.sort_by(f64::total_cmp);
        println!("PACKAGES {count}: lookup p50={:.3}ms search p50={:.3}ms p95={:.3}ms resolver p50={:.3}ms", lookup[10], search[10], search[19], solve[10]);
        assert!(
            lookup[10] < 50.0 && search[10] < 300.0 && search[19] < 1000.0 && solve[10] < 100.0
        );
    }
}

#[test]
fn mission_package_conditions_events_and_repository_effects_are_persistent() {
    use crate::mission::{Condition, Effect, MissionEngine};
    let mut w = world();
    let mut engine = MissionEngine::load().unwrap();
    let mut mission = engine.definitions[0].clone();
    mission.id = "package-fixture".into();
    mission.requirements.clear();
    mission.start_triggers.clear();
    mission.on_start = vec![Effect::Repository {
        id: repository::COMMUNITY.into(),
        available: true,
        trusted: true,
        release: 1,
    }];
    mission.stages[0].conditions = vec![Condition::PackageInstalled {
        name: "netscan".into(),
        version: Some("2.4.1".into()),
    }];
    mission.stages[0].effects.clear();
    mission.stages[0].choices.clear();
    mission.stages.truncate(1);
    mission.outcomes.clear();
    engine.definitions.push(mission);
    engine.start(&mut w, "package-fixture").unwrap();
    assert!(w.packages.repositories.contains_key(repository::COMMUNITY));
    assert!(engine.evaluate(&mut w).unwrap().is_empty());
    let before = w.clone();
    ok(&mut w, "sudo apt update");
    ok(&mut w, "sudo apt install -y netscan");
    engine.track_action(&before, &mut w).unwrap();
    assert!(Condition::PackageEvent {
        event: "PACKAGE_INSTALLED".into(),
        package: Some("netscan".into())
    }
    .matches(&w));
    engine.abort(&mut w, "package-fixture").unwrap();
    assert!(w.vfs.nodes.contains_key("/usr/bin/netscan"));
    w.validate().unwrap();
    engine.start(&mut w, "package-fixture").unwrap();
    assert!(engine
        .evaluate(&mut w)
        .unwrap()
        .iter()
        .any(|e| e.kind == "mission_end"));
}
