//! Regression contract: personal state is durable, an attempt is not, and only
//! one mission may run. Test fixtures here are not new campaign content.
use crate::{
    db,
    mission::{Condition, Effect, MissionDefinition, Stage},
    save,
    service::GameService,
    terminal, terminal_sessions,
};
use rusqlite::Connection;

fn definition(id: &str) -> MissionDefinition {
    MissionDefinition {
        id: id.into(),
        title: id.into(),
        contact: "TEST".into(),
        session: 1,
        description: "test fixture".into(),
        requirements: vec![],
        start_triggers: vec![],
        on_start: vec![Effect::Message {
            contact: "TEST".into(),
            text: format!("start {id}"),
        }],
        stages: vec![
            Stage {
                objective: "partial".into(),
                hint: String::new(),
                choices: vec![],
                conditions: vec![Condition::Flag {
                    key: format!("{id}_PARTIAL"),
                }],
                effects: vec![
                    Effect::File {
                        path: format!("/home/kali/Documents/{id}.tmp"),
                        text: "temporary".into(),
                    },
                    Effect::Reward {
                        money: 5,
                        reputation: 2,
                    },
                ],
            },
            Stage {
                objective: "finish".into(),
                hint: String::new(),
                choices: vec![],
                effects: vec![],
                conditions: vec![Condition::Flag {
                    key: format!("{id}_FINISH"),
                }],
            },
        ],
        outcomes: vec![
            Effect::Flag {
                key: format!("{id}_DONE"),
            },
            Effect::Reward {
                money: 10,
                reputation: 3,
            },
        ],
    }
}
fn game() -> GameService {
    let mut game = GameService::new(Connection::open_in_memory().unwrap()).unwrap();
    game.new_game(1, "neo", "pc", false).unwrap();
    game.mutate(
        |_, w, _| {
            w.vfs
                .write("/home/kali/Documents/notes.txt", "durable", "kali")
        },
        false,
    )
    .unwrap();
    game.engine.definitions = vec![definition("A"), definition("B")];
    game
}
fn start(game: &mut GameService, id: &str) {
    game.mutate(
        |engine, w, events| {
            events.push(engine.start(w, id)?);
            Ok(())
        },
        false,
    )
    .unwrap();
}
fn partial(game: &mut GameService) {
    game.mutate(
        |_, w, _| {
            w.flags.insert("A_PARTIAL".into());
            w.vfs.mkdir("/home/kali/Documents/personal", "kali")?;
            w.vfs
                .write("/home/kali/Documents/personal/note.txt", "keep me", "kali")?;
            w.settings.insert("wallpaper".into(), "maze".into());
            w.settings.insert("fileSort".into(), "modified".into());
            w.settings
                .insert("servicesEnabled".into(), "[\"ssh\"]".into());
            w.settings.insert("aptManual".into(), "[\"curl\"]".into());
            w.network.wifi[0].access = true;
            Ok(())
        },
        false,
    )
    .unwrap();
}

#[test]
fn binary_before_image_survives_journal_serialization_and_attempt_discard() {
    use base64::{engine::general_purpose::STANDARD, Engine};
    let mut game = game();
    let path = "/home/kali/Documents/A.tmp";
    let bytes = [0, 255, 128, 13, 10];
    game.mutate(
        |_, w, _| {
            crate::binary::import(
                w,
                path,
                &STANDARD.encode(bytes),
                "application/octet-stream",
                None,
                "kali",
            )
        },
        false,
    )
    .unwrap();
    start(&mut game, "A");
    partial(&mut game);
    assert_eq!(
        game.world().unwrap().vfs.read(path, "kali").unwrap(),
        "temporary"
    );
    assert!(game.world().unwrap().vfs.nodes[path].blob.is_none());
    game.save(true).unwrap();
    // Loading must hydrate the old bytes held only in the attempt journal,
    // then discard the temporary text, before publishing the resumed world.
    game.load(1, true).unwrap();
    assert_eq!(
        crate::binary::read(game.world().unwrap(), path, "kali")
            .unwrap()
            .base64,
        STANDARD.encode(bytes)
    );
    assert!(!game.world().unwrap().missions.contains_key("A"));
}
fn assert_discarded(game: &GameService) {
    let w = game.world().unwrap();
    assert!(!w.missions.contains_key("A"));
    assert!(!w.flags.contains("A_PARTIAL"));
    assert!(!w.vfs.nodes.contains_key("/home/kali/Documents/A.tmp"));
    assert!(w.messages.iter().all(|m| m.text != "start A"));
    assert_eq!(w.money, 0);
    assert_eq!(w.reputation, 0);
    assert_eq!(
        w.vfs
            .read("/home/kali/Documents/personal/note.txt", "kali")
            .unwrap(),
        "keep me"
    );
    assert_eq!(w.settings["wallpaper"], "maze");
    assert_eq!(w.settings["fileSort"], "modified");
    assert_eq!(w.settings["servicesEnabled"], "[\"ssh\"]");
    assert_eq!(w.settings["aptManual"], "[\"curl\"]");
    assert!(w.network.wifi[0].access);
    assert!(w.flags.contains("FIRST_BOOT_COMPLETE"));
}

#[test]
fn abort_preserves_personal_files_settings_profiles_and_removes_only_temporary_effects() {
    let mut game = game();
    start(&mut game, "A");
    partial(&mut game);
    game.mutate(
        |_, w, _| {
            for m in &mut w.messages {
                m.read = true;
            }
            Ok(())
        },
        false,
    )
    .unwrap();
    game.mutate(|e, w, _| e.abort(w, "A"), false).unwrap();
    assert_discarded(&game);
    start(&mut game, "A");
    assert_eq!(game.world().unwrap().missions["A"].attempts, 2);
    assert_eq!(game.world().unwrap().missions["A"].stage, 0);
}
#[test]
fn second_mission_is_rejected_without_changing_any_state_and_unlocks_after_abort() {
    let mut game = game();
    start(&mut game, "A");
    partial(&mut game);
    let before = save::encode(game.world().unwrap()).unwrap();
    assert!(game
        .mutate(|e, w, _| e.start(w, "B"), false)
        .unwrap_err()
        .to_string()
        .contains("Já existe"));
    assert_eq!(before, save::encode(game.world().unwrap()).unwrap());
    game.mutate(|e, w, _| e.abort(w, "A"), false).unwrap();
    start(&mut game, "B");
    assert_eq!(game.world().unwrap().missions["B"].status, "active");
}
#[test]
fn completion_promotes_rewards_and_unlocks_next_mission_without_reverting_them() {
    let mut game = game();
    start(&mut game, "A");
    partial(&mut game);
    game.mutate(
        |_, w, _| {
            w.flags.insert("A_FINISH".into());
            Ok(())
        },
        false,
    )
    .unwrap();
    assert_eq!(game.world().unwrap().money, 15);
    start(&mut game, "B");
    game.end_session().unwrap();
    game.load(1, false).unwrap();
    assert!(game.world().unwrap().flags.contains("A_DONE"));
    assert_eq!(game.world().unwrap().missions["A"].status, "completed");
    assert!(!game.world().unwrap().missions.contains_key("B"));
    assert_eq!(game.world().unwrap().money, 15);
    game.load(1, true).unwrap();
    assert!(game.world().unwrap().flags.contains("A_DONE"));
    assert!(game
        .world()
        .unwrap()
        .vfs
        .nodes
        .contains_key("/home/kali/Documents/A.tmp"));
}
#[test]
fn manual_save_and_autosave_crash_reload_both_discard_the_attempt() {
    for manual in [false, true] {
        let mut game = game();
        start(&mut game, "A");
        partial(&mut game);
        game.save(manual).unwrap();
        // New service over the same database, without end_session: forced exit.
        let mut reopened = GameService::new(game.connection).unwrap();
        reopened.engine.definitions = vec![definition("A"), definition("B")];
        reopened.load(1, manual).unwrap();
        assert_discarded(&reopened);
        assert!(reopened.world().unwrap().terminal_sessions.is_empty());
        assert!(reopened.world().unwrap().processes.is_empty());
    }
}
#[test]
fn failed_logout_or_load_does_not_publish_partial_rollback() {
    let mut game = game();
    start(&mut game, "A");
    partial(&mut game);
    let before = save::encode(game.world().unwrap()).unwrap();
    game.connection.execute_batch("CREATE TRIGGER fail_save BEFORE UPDATE ON save_slots BEGIN SELECT RAISE(ABORT,'test failure'); END;").unwrap();
    assert!(game.end_session().is_err());
    assert_eq!(before, save::encode(game.world().unwrap()).unwrap());
    assert!(game.load(1, false).is_err());
    assert_eq!(before, save::encode(game.world().unwrap()).unwrap());
    game.connection
        .execute_batch("DROP TRIGGER fail_save;")
        .unwrap();
    game.end_session().unwrap();
    game.end_session().unwrap();
    assert_discarded(&game);
    assert!(game
        .mutate(
            |_, w, _| {
                w.flags.insert("LATE_COMMAND".into());
                Ok(())
            },
            false
        )
        .is_err());
    game.save(false).unwrap();
    assert!(!game.world().unwrap().flags.contains("LATE_COMMAND"));
    game.start_session().unwrap();
    game.mutate(
        |_, w, _| {
            w.flags.insert("NEW_SESSION".into());
            Ok(())
        },
        false,
    )
    .unwrap();
}
#[test]
fn clean_login_rebuilds_only_enabled_services_and_clears_terminals() {
    let mut game = game();
    game.mutate(
        |_, w, _| {
            terminal_sessions::open(w, "old")?;
            terminal_sessions::with_session(w, Some("old"), |w| {
                terminal::execute(w, "ssh vex@vex.local lab-only");
                Ok(())
            })?;
            terminal::execute(w, "sudo systemctl enable ssh");
            terminal::execute(w, "sudo systemctl start apache2");
            Ok(())
        },
        false,
    )
    .unwrap();
    let w = game.start_session().unwrap();
    assert!(w.terminal_sessions.is_empty());
    assert!(w.terminal.host.is_none());
    assert!(w.processes.iter().any(|p| p.name == "ssh" && p.running));
    assert!(w.processes.iter().all(|p| p.name != "apache2"));
    assert_eq!(w.processes.len(), 2);
}
#[test]
fn legacy_whole_world_baseline_is_migrated_without_restoring_personal_data() {
    let mut game = game();
    let baseline = serde_json::to_string(game.world().unwrap()).unwrap();
    start(&mut game, "A");
    partial(&mut game);
    let mut legacy = game.world().unwrap().clone();
    legacy.schema_version = 1;
    legacy.mission_runtime = Default::default();
    legacy.settings.insert("missionBaseline".into(), baseline);
    // Legacy reward cannot be inferred from a missing journal; use an authored
    // stage reconstruction in normalize_loaded, never a whole-world replacement.
    let tx = game.connection.transaction().unwrap();
    db::write_snapshot(&tx, 1, "neo", &legacy, true).unwrap();
    tx.commit().unwrap();
    game.load(1, false).unwrap();
    assert_eq!(game.world().unwrap().schema_version, 2);
    assert!(!game
        .world()
        .unwrap()
        .settings
        .contains_key("missionBaseline"));
    assert_eq!(
        game.world()
            .unwrap()
            .vfs
            .read("/home/kali/Documents/personal/note.txt", "kali")
            .unwrap(),
        "keep me"
    );
    assert!(!game.world().unwrap().missions.contains_key("A"));
    assert!(!game.world().unwrap().flags.contains("A_PARTIAL"));
    assert_eq!(game.world().unwrap().money, 0);
}

#[test]
fn copies_and_moves_of_temporary_artifacts_remain_temporary_but_personal_copies_survive() {
    let mut game = game();
    start(&mut game, "A");
    partial(&mut game);
    game.mutate(
        |_, w, _| {
            assert_eq!(
                terminal::execute(w, "cp Documents/A.tmp Documents/copied.tmp").exit_code,
                0
            );
            assert_eq!(
                terminal::execute(w, "mv Documents/copied.tmp Documents/moved.tmp").exit_code,
                0
            );
            assert_eq!(
                terminal::execute(
                    w,
                    "cp Documents/personal/note.txt Documents/personal/copy.txt"
                )
                .exit_code,
                0
            );
            Ok(())
        },
        false,
    )
    .unwrap();
    game.end_session().unwrap();
    let w = game.world().unwrap();
    assert!(!w.vfs.nodes.contains_key("/home/kali/Documents/moved.tmp"));
    assert!(!w.vfs.nodes.contains_key("/home/kali/Documents/copied.tmp"));
    assert_eq!(
        w.vfs
            .read("/home/kali/Documents/personal/copy.txt", "kali")
            .unwrap(),
        "keep me"
    );
}

#[test]
fn old_saves_with_two_active_missions_are_normalized_to_zero_active_attempts() {
    let mut game = game();
    let before_a = game.world().unwrap().clone();
    start(&mut game, "A");
    partial(&mut game);
    let before_b = game.world().unwrap().clone();
    // Simulate data written by the old engine, which had no exclusivity guard.
    let mut legacy = before_b.clone();
    legacy.schema_version = 1;
    let a = legacy.missions.remove("A").unwrap();
    game.engine.start(&mut legacy, "B").unwrap();
    legacy.missions.insert("A".into(), a);
    legacy.mission_runtime = Default::default();
    game.engine
        .normalize_loaded(&mut legacy, |id| {
            Ok(if id == "A" {
                before_a.clone()
            } else {
                before_b.clone()
            })
        })
        .unwrap();
    assert_eq!(legacy.schema_version, 2);
    assert!(legacy.missions.values().all(|m| m.status != "active"));
    assert!(legacy.mission_runtime.attempts.is_empty());
    assert_eq!(
        legacy
            .vfs
            .read("/home/kali/Documents/personal/note.txt", "kali")
            .unwrap(),
        "keep me"
    );
    assert!(!legacy.vfs.nodes.contains_key("/home/kali/Documents/A.tmp"));
}
