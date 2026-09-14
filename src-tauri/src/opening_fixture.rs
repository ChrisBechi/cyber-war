//! Development capture data is exported from the same WorldState and instruments as gameplay.
use crate::{browser, investigation, mission::MissionEngine, terminal, world::WorldState};

#[test]
fn opening_snapshot_uses_valid_gameplay_world_and_real_command_results() {
    let mut world = WorldState::new("kali", "game-hacker").unwrap();
    world.nickname = "root".into();
    world.terminal.user = "root".into();
    world.messages.clear();
    world.contacts.extend(["VEX".into(), "NULL".into()]);
    world.flags.extend([
        "V1_COMPLETE".into(),
        "WIFI_COMPLETE".into(),
        "SESSION_1_COMPLETE".into(),
        "GAME_DOWNLOADED".into(),
    ]);
    let engine = MissionEngine::load().unwrap();
    engine.start(&mut world, "signal-no-ar").unwrap();
    world.messages.clear();
    world.settings.insert("fontSize".into(), "16".into());
    world
        .vfs
        .seed("/home/kali/Downloads/recebidos", "directory", "", "kali");
    world.vfs.seed(
        "/home/kali/Downloads/recebidos/registro.txt",
        "file",
        "Origem: desconhecida\nArquivo recebido às 23:17\nPreservar original.\n",
        "kali",
    );
    world.vfs.seed("/home/kali/projects/notas.txt", "file", "Conferir os horários.\nO serviço parou depois da alteração.\nLer o log antes de mudar a configuração.\n", "kali");
    world.vfs.seed(
        "/home/kali/tools/node.trace",
        "file",
        "trace.route\nsession.local\n",
        "kali",
    );
    world.vfs.seed("/home/kali/projects/session.json", "file", "{\n  \"session\": \"orion-2317\",\n  \"origin\": \"archive.org\",\n  \"received\": \"23:17:09\",\n  \"integrity\": \"verified\",\n  \"preserve_original\": true\n}\n", "kali");
    world.validate().unwrap();
    let mut case_world = world.clone();
    let evidence = investigation::inspect(&mut case_world, "02:00:00:00:20:04").unwrap();
    let packets =
        investigation::packets(&case_world, evidence.capture_path.as_ref().unwrap()).unwrap();
    let mut results = serde_json::Map::new();
    for input in [
        "nmap vex.local",
        "tree /home/kali",
        "ss",
        "lab trace",
        "ip addr",
        "ps",
        "traceroute archive.org",
        "dig vex.local",
        "nmap archive.org",
        "curl https://archive.org",
        "find /home/kali",
        "lab inspect",
        "lab scan 100",
        "sha256sum /home/kali/Downloads/recebidos/registro.txt",
        "sha256sum /home/kali/projects/session.json",
        "base64 /home/kali/projects/session.json",
        "cat /home/kali/projects/session.json",
        "chmod 600 /home/kali/Downloads/recebidos/registro.txt",
        "stat /home/kali/Downloads/recebidos/registro.txt",
    ] {
        let result = terminal::execute(&mut world.clone(), input);
        assert_eq!(result.exit_code, 0, "{input}: {}", result.stderr);
        results.insert(input.into(), serde_json::to_value(result).unwrap());
    }
    let mut scanned = world.clone();
    terminal::execute(&mut scanned, "lab scan 100");
    let mut restricted = world.clone();
    terminal::execute(
        &mut restricted,
        "chmod 600 /home/kali/Downloads/recebidos/registro.txt",
    );
    results.insert(
        "stat /home/kali/Downloads/recebidos/registro.txt".into(),
        serde_json::to_value(terminal::execute(
            &mut restricted,
            "stat /home/kali/Downloads/recebidos/registro.txt",
        ))
        .unwrap(),
    );
    let mut remote = world.clone();
    for input in [
        "ssh vex@vex.local lab-only",
        "sudo cat /var/log/web.log",
        "sudo edit /etc/web.conf enabled=true",
        "sudo service web restart",
        "sudo id",
    ] {
        let result = terminal::execute(&mut remote, input);
        assert_eq!(result.exit_code, 0, "{input}: {}", result.stderr);
        results.insert(input.into(), serde_json::to_value(result).unwrap());
    }
    let mut pages = serde_json::Map::new();
    for address in [
        "archive.org",
        "b1.tech",
        "wipedia.org",
        "pg6mmjiyjmcrsslvykfwnntlaru7p5svn6y2ymmju6nubxndf4pscryd.onion",
        "pg6mmjiyjmcrsslvykfwnntlaru7p5svn6y2ymmju6nubxndf4pscryd.onion/archive",
    ] {
        pages.insert(
            address.into(),
            serde_json::to_value(browser::navigate(&mut world.clone(), address).unwrap()).unwrap(),
        );
    }
    let snapshot = serde_json::json!({ "world": world, "wireless": investigation::scan(&case_world).unwrap(), "evidence": evidence, "packets": packets, "commands": results, "pages": pages, "labCandidates": scanned.memory_candidates });
    if let Ok(path) = std::env::var("CYBER_WAR_EXPORT_OPENING") {
        std::fs::write(path, serde_json::to_string_pretty(&snapshot).unwrap()).unwrap();
    }
}
