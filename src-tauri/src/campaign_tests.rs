//! Integration coverage through the same transaction boundary used by IPC.
use crate::{browser, save, service::GameService, terminal};
use rusqlite::Connection;

fn command(game: &mut GameService, input: &str) {
    let result = game
        .mutate(|_, w, _| Ok(terminal::execute(w, input)), false)
        .expect("transaction");
    assert_eq!(result.exit_code, 0, "{input}: {}", result.stderr);
}
fn start(game: &mut GameService, id: &str) {
    game.mutate(
        |e, w, events| {
            events.push(e.start(w, id)?);
            Ok(())
        },
        false,
    )
    .expect(id);
}
fn choose(game: &mut GameService, id: &str, choice: &str) {
    game.mutate(
        |e, w, events| {
            if let Some(checkpoint) = e.choose(w, id, choice)? {
                events.push(checkpoint);
            }
            Ok(())
        },
        false,
    )
    .expect(choice);
}
fn flag(game: &GameService, key: &str) {
    assert!(
        game.world().expect("world").flags.contains(key),
        "missing {key}"
    );
}

#[test]
fn full_session_alternative_order_and_checkpoint_branch() {
    for prior_knowledge in [false, true] {
        let mut game = GameService::new(Connection::open_in_memory().expect("db")).expect("game");
        game.new_game(1, "player", "my-pc", false).expect("new");
        assert!(game.mutate(|e, w, _| e.start(w, "girl"), false).is_err());
        command(&mut game, "echo 'minha nota' > Documents/notes.txt");
        flag(&game, "FIRST_BOOT_COMPLETE");
        start(&mut game, "v1");
        command(&mut game, "wget https://archive.org");
        command(&mut game, "lab scan 100");
        command(&mut game, "lab play");
        command(&mut game, "lab scan 99");
        command(&mut game, "lab build-v1");
        flag(&game, "V1_COMPLETE");
        if prior_knowledge {
            game.mutate(
                |_, w, _| {
                    w.vfs
                        .write("/home/kali/projects/profile_01.dat", "gold=999999", "kali")
                },
                false,
            )
            .expect("GUI edit");
            start(&mut game, "v2");
            assert!(game
                .world()
                .expect("world")
                .messages
                .iter()
                .any(|m| m.text.contains("já sabia")));
        } else {
            start(&mut game, "v2");
            command(&mut game, "lab inspect");
            command(&mut game, "lab build-v2");
        }
        flag(&game, "V2_COMPLETE");
        assert!(!game.world().expect("world").network.connected);
        if prior_knowledge {
            start(&mut game, "pendrive");
            assert!(game.mutate(|e, w, _| e.start(w, "wifi"), false).is_err());
            complete_usb(&mut game);
            start(&mut game, "wifi");
            complete_wifi(&mut game);
        } else {
            start(&mut game, "wifi");
            assert!(game
                .mutate(|e, w, _| e.start(w, "pendrive"), false)
                .is_err());
            complete_wifi(&mut game);
            start(&mut game, "pendrive");
            complete_usb(&mut game);
        }
        start(&mut game, "forum-job");
        command(&mut game, "ssh vex@vex.local lab-only");
        command(&mut game, "cat /var/log/web.log");
        command(&mut game, "sudo edit /etc/web.conf 'enabled=true'");
        command(&mut game, "sudo service web restart");
        flag(&game, "FORUM_JOB_COMPLETE");
        start(&mut game, "vex-setup");
        command(&mut game, "sudo edit /srv/www/health.txt OK");
        command(&mut game, "service web status");
        flag(&game, "VEX_SETUP_COMPLETE");
        start(&mut game, "vex-production");
        command(&mut game, "cp /etc/web.conf /home/kali/web-backup.conf");
        command(&mut game, "lab verify-backup");
        let before = game.world().expect("world").money;
        choose(&mut game, "vex-production", "partial");
        let decision = save::checkpoints(&game.connection, 1)
            .expect("checkpoints")
            .into_iter()
            .find(|c| {
                c.checkpoint_type == "decision" && c.mission_id.as_deref() == Some("vex-production")
            })
            .expect("decision");
        assert!(game
            .world()
            .expect("world")
            .missions
            .contains_key("vex-incident"));
        command(&mut game, "lab contain");
        command(&mut game, "lab restore");
        flag(&game, "VEX_INCIDENT_COMPLETE");
        game.restore(&decision.id).expect("rollback");
        assert_eq!(game.world().expect("world").money, before);
        assert!(!game
            .world()
            .expect("world")
            .missions
            .contains_key("vex-incident"));
        assert!(!game
            .world()
            .expect("world")
            .missions
            .contains_key("vex-production"));
        start(&mut game, "vex-production");
        command(&mut game, "ssh vex@vex.local lab-only");
        command(&mut game, "cp /etc/web.conf /home/kali/web-backup.conf");
        command(&mut game, "lab verify-backup");
        choose(&mut game, "vex-production", "secure");
        flag(&game, "VEX_HARDENED");
        assert_eq!(game.world().expect("world").money, before + 120);
        assert!(game
            .world()
            .expect("world")
            .network
            .ssh("root@vex.local", "lab-only")
            .is_err());
        command(&mut game, "exit");
        start(&mut game, "girl");
        assert!(game
            .mutate(|_, w, _| browser::action(w, "recover", "FOTO-17"), false)
            .is_err());
        game.mutate(|_, w, _| browser::navigate(w, "fakebook.com"), false)
            .expect("profile");
        assert!(game
            .mutate(|_, w, _| browser::action(w, "recover", "bad"), false)
            .is_err());
        game.mutate(|_, w, _| browser::action(w, "recover", "FOTO-17"), false)
            .expect("recovery");
        choose(&mut game, "girl", "share");
        flag(&game, "SESSION_1_COMPLETE");
        assert!(game
            .world()
            .expect("world")
            .evidence
            .iter()
            .any(|e| e == "EVIDENCE_001_GIRL"));
        game.save(true).expect("save");
        let mut durable = game.world().expect("world").clone();
        terminal::stop_session_runtime(&mut durable).expect("normalize runtime");
        let encoded = save::encode(&durable).expect("encode");
        game.load(1, false).expect("load");
        assert_eq!(
            save::encode(game.world().expect("world")).expect("encode"),
            encoded
        );
        assert_eq!(
            game.world()
                .expect("world")
                .vfs
                .read("/home/kali/Documents/notes.txt", "kali")
                .expect("note"),
            "minha nota\n"
        );
    }
}
fn complete_usb(game: &mut GameService) {
    command(game, "cp /dev/usb.img Documents/usb-backup.img");
    command(game, "lab recover");
    choose(game, "pendrive", "respect");
    flag(game, "PENDRIVE_COMPLETE");
}
fn complete_wifi(game: &mut GameService) {
    command(game, "lab wifi");
    command(game, "lab collect");
    command(game, "lab analyze");
    command(game, "lab connect");
    flag(game, "WIFI_COMPLETE");
}
