use crate::{
    db,
    error::GameResult,
    mission::{CheckpointEvent, MissionEngine},
    save, terminal,
    vfs::domain,
    world::WorldState,
};
use rusqlite::Connection;
use std::time::Instant;
#[cfg(test)]
#[path = "service_cooperative_tests.rs"]
mod cooperative_tests;

pub struct ActiveGame {
    pub slot: i64,
    pub label: String,
    pub world: WorldState,
    clock: Instant,
    pub(crate) session_ended: bool,
}
pub struct GameService {
    pub(crate) runtime_epoch: u64,
    pub connection: Connection,
    pub engine: MissionEngine,
    pub active: Option<ActiveGame>,
}

impl GameService {
    pub(crate) fn transact_cooperatively<T: Send>(
        service: &parking_lot::Mutex<Self>,
        registration: &crate::shell::control::Registration,
        work: impl FnOnce(&mut WorldState) -> GameResult<T> + Send,
    ) -> GameResult<T> {
        let epoch = service.lock().runtime_epoch;
        crate::shell::cooperative::drive(
            registration,
            work,
            |turn| {
                let mut game = service.lock();
                if game.runtime_epoch != epoch {
                    return Err(domain("virtual session changed while command was waiting"));
                }
                game.mutate(|_, world, _| turn(world), false)
            },
            |before, after| {
                let mut game = service.lock();
                if game.runtime_epoch == epoch {
                    if let Some(active) = &mut game.active {
                        active.world.release_runtime_closed(before, after);
                    }
                }
            },
        )?
    }
    pub fn new(mut connection: Connection) -> GameResult<Self> {
        db::migrate(&mut connection)?;
        Ok(Self {
            runtime_epoch: 0,
            connection,
            engine: MissionEngine::load()?,
            active: None,
        })
    }
    pub fn world(&self) -> GameResult<&WorldState> {
        self.active
            .as_ref()
            .map(|a| &a.world)
            .ok_or_else(|| domain("no active campaign"))
    }
    pub fn new_game(
        &mut self,
        slot: i64,
        nickname: &str,
        hostname: &str,
        overwrite: bool,
    ) -> GameResult<WorldState> {
        db::validate_slot(slot)?;
        let occupied: bool = self.connection.query_row(
            "SELECT occupied FROM save_slots WHERE slot_index=?1",
            [slot],
            |r| r.get(0),
        )?;
        if occupied && !overwrite {
            return Err(domain("slot occupied; explicit overwrite required"));
        }
        let mut world = WorldState::new(nickname, hostname)?;
        let checkpoint = self.engine.start(&mut world, "first-boot")?;
        let tx = self.connection.transaction()?;
        tx.execute("DELETE FROM save_checkpoints WHERE slot_index=?1", [slot])?;
        db::write_snapshot(&tx, slot, nickname, &world, true)?;
        save::checkpoint(&tx, slot, &checkpoint)?;
        tx.commit()?;
        if let Some(active) = &mut self.active {
            crate::archive::jobs::reset(&mut active.world);
        }
        self.active = Some(ActiveGame {
            slot,
            label: nickname.into(),
            world: world.clone(),
            clock: Instant::now(),
            session_ended: false,
        });
        self.runtime_epoch += 1;
        Ok(world)
    }
    pub fn delete_slot(
        &mut self,
        slot: i64,
        confirmed: bool,
    ) -> GameResult<Vec<save::SaveSlotSummary>> {
        db::validate_slot(slot)?;
        if !confirmed {
            return Err(domain("Confirme a exclusão do save antes de continuar."));
        }
        let tx = self.connection.transaction()?;
        tx.execute("DELETE FROM save_checkpoints WHERE slot_index=?1", [slot])?;
        tx.execute(
            "UPDATE save_slots SET label='', occupied=0, current_mission=NULL, playtime_seconds=0,
             state_json=NULL, checksum=NULL, autosave_json=NULL, autosave_checksum=NULL,
             created_at=NULL, updated_at=NULL WHERE slot_index=?1",
            [slot],
        )?;
        let slots = save::list(&tx)?;
        tx.commit()?;
        if self
            .active
            .as_ref()
            .is_some_and(|active| active.slot == slot)
        {
            crate::shell::control::cancel_all();
            if let Some(active) = &mut self.active {
                crate::archive::jobs::reset(&mut active.world);
            }
            self.active = None;
            self.runtime_epoch += 1;
        }
        Ok(slots)
    }
    pub fn load(&mut self, slot: i64, manual: bool) -> GameResult<WorldState> {
        let (label, mut world) = save::load(&self.connection, slot, manual)?;
        normalize_entry(&self.engine, &self.connection, slot, &mut world)?;
        crate::virtual_web::sync_events(&mut world);
        world.vfs.ensure_trash();
        let tx = self.connection.transaction()?;
        db::write_snapshot(&tx, slot, &label, &world, false)?;
        tx.commit()?;
        if let Some(active) = &mut self.active {
            crate::archive::jobs::reset(&mut active.world);
        }
        self.active = Some(ActiveGame {
            slot,
            label,
            world: world.clone(),
            clock: Instant::now(),
            session_ended: false,
        });
        self.runtime_epoch += 1;
        Ok(world)
    }
    /// Clone and persist before publishing. Failed SQL never changes the live world.
    pub fn mutate<T>(
        &mut self,
        change: impl FnOnce(&MissionEngine, &mut WorldState, &mut Vec<CheckpointEvent>) -> GameResult<T>,
        manual: bool,
    ) -> GameResult<T> {
        let active = self
            .active
            .as_ref()
            .ok_or_else(|| domain("no active campaign"))?;
        if active.session_ended {
            return Err(domain("Sessão encerrada. Entre novamente para continuar."));
        }
        let mut world = active.world.clone();
        let elapsed = active.clock.elapsed().as_secs();
        world.playtime_seconds += elapsed;
        crate::virtual_web::sync_events(&mut world);
        let mut events = Vec::new();
        let anchors = world.cwd_anchors();
        let result = change(&self.engine, &mut world, &mut events)?;
        world.repair_cwds(anchors);
        terminal::observe(&mut world);
        self.engine.track_action(&active.world, &mut world)?;
        events.extend(self.engine.evaluate(&mut world)?);
        crate::virtual_web::sync_events(&mut world);
        terminal::observe(&mut world);
        world.vfs.collect();
        for host in world.network.hosts.values_mut() {
            host.files.collect();
        }
        world.validate()?;
        let tx = self.connection.transaction()?;
        for event in &events {
            save::checkpoint(&tx, active.slot, event)?;
        }
        // Completing a mission is a durable campaign boundary. It promotes
        // the resulting world to the manual slot even when the action itself
        // came from an ordinary command or the background autosave.
        let mission_completed = events.iter().any(|event| event.kind == "mission_end");
        db::write_snapshot(
            &tx,
            active.slot,
            &active.label,
            &world,
            manual || mission_completed,
        )?;
        tx.commit()?;
        if let Some(active) = &mut self.active {
            active.world = world;
            active.clock += std::time::Duration::from_secs(elapsed);
            crate::shell::control::notify_world(&active.world);
        }
        Ok(result)
    }
    pub fn save(&mut self, manual: bool) -> GameResult<()> {
        if self.active.as_ref().is_some_and(|a| a.session_ended) {
            return Ok(());
        }
        self.mutate(
            |_, world, events| {
                if !manual {
                    events.push(CheckpointEvent {
                        kind: "autosave",
                        mission: String::new(),
                        label: "Autosave seguro".into(),
                        state: world.clone(),
                    });
                }
                Ok(())
            },
            manual,
        )
    }
    pub fn tick_web(&mut self) -> GameResult<bool> {
        let Some(active) = &self.active else {
            return Ok(false);
        };
        if active.session_ended {
            return Ok(false);
        }
        let current = active
            .world
            .playtime_seconds
            .saturating_add(active.clock.elapsed().as_secs());
        if !crate::virtual_web::events::next_change(&active.world)
            .is_some_and(|next| next <= current)
        {
            return Ok(false);
        }
        self.mutate(|_, _, _| Ok(()), false)?;
        Ok(true)
    }
    pub fn end_session(&mut self) -> GameResult<()> {
        let Some(active) = &self.active else {
            return Ok(());
        };
        if active.session_ended {
            return Ok(());
        }
        let (slot, label) = (active.slot, active.label.clone());
        let mut world = active.world.clone();
        world.playtime_seconds += active.clock.elapsed().as_secs();
        normalize_entry(&self.engine, &self.connection, slot, &mut world)?;
        let tx = self.connection.transaction()?;
        db::write_snapshot(&tx, slot, &label, &world, false)?;
        tx.commit()?;
        if let Some(active) = &mut self.active {
            active.world = world;
            active.clock = Instant::now();
            active.session_ended = true;
            self.runtime_epoch += 1;
        }
        Ok(())
    }
    pub fn commit_system_setup(&mut self) -> GameResult<()> {
        // System settings and installer files are durable, never mission-owned.
        self.save(false)
    }
    pub fn start_session(&mut self) -> GameResult<WorldState> {
        let was_ended = self.active.as_ref().is_some_and(|a| a.session_ended);
        let previous_clock = self.active.as_ref().map(|a| a.clock);
        if let Some(active) = &mut self.active {
            active.session_ended = false;
            if was_ended {
                active.clock = Instant::now();
            }
        }
        let result = self.mutate(|_, world, _| terminal::prepare_session_start(world), false);
        if let Err(error) = result {
            if let Some(active) = &mut self.active {
                active.session_ended = was_ended;
                if let Some(clock) = previous_clock {
                    active.clock = clock;
                }
            }
            return Err(error);
        }
        Ok(self.world()?.clone())
    }
    pub fn restore(&mut self, id: &str) -> GameResult<WorldState> {
        let active = self
            .active
            .as_ref()
            .ok_or_else(|| domain("no active campaign"))?;
        let mut world = save::restore(
            &mut self.connection,
            active.slot,
            id,
            &active.label,
            |connection, world| normalize_entry(&self.engine, connection, active.slot, world),
        )?;
        world.vfs.ensure_trash();
        if let Some(active) = &mut self.active {
            crate::archive::jobs::reset(&mut active.world);
            active.world = world.clone();
            active.clock = Instant::now();
            active.session_ended = false;
        }
        self.runtime_epoch += 1;
        Ok(world)
    }
}

fn normalize_entry(
    engine: &MissionEngine,
    connection: &Connection,
    slot: i64,
    world: &mut WorldState,
) -> GameResult<()> {
    let legacy_json = world.settings.get("missionBaseline").cloned();
    engine.normalize_loaded(world, |id| {
        if let Ok((baseline, _)) = save::mission_start(connection, slot, id) { return Ok(baseline); }
        let mut candidate = legacy_json.clone();
        for _ in 0..64 {
            let Some(json) = candidate else { break; };
            let original: WorldState = serde_json::from_str(&json)?;
            if !original.missions.contains_key(id) { return Ok(original); }
            candidate = original.settings.get("missionBaseline").cloned();
        }
        Err(domain(format!("Não foi possível migrar a tentativa antiga {id}: referência ausente. O save foi preservado.")))
    })?;
    terminal::stop_session_runtime(world)?;
    crate::packages::state::initialize(world)?;
    crate::virtual_web::sync_events(world);
    world.validate()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn service() -> GameService {
        GameService::new(Connection::open_in_memory().expect("db")).expect("service")
    }
    #[test]
    fn legacy_repository_state_loads_manual_autosave_and_checkpoints_without_data_loss() {
        use sha2::{Digest, Sha256};
        let mut game = service();
        game.new_game(2, "other", "other-pc", false).unwrap();
        let other = save::encode(&save::load(&game.connection, 2, false).unwrap().1).unwrap();
        game.new_game(1, "legacy", "legacy-pc", false).unwrap();
        game.mutate(
            |_, world, _| {
                world.money = 123;
                world.playtime_seconds = 456;
                world
                    .vfs
                    .write("/home/kali/notes.txt", "keep this", "kali")?;
                Ok(())
            },
            true,
        )
        .unwrap();
        let mut value = serde_json::to_value(game.world().unwrap()).unwrap();
        value["packages"]["repositories"] = serde_json::json!({
            "legacy": {},
            "untrusted": {"trusted": false, "release": 7},
            "offline": {"available": false, "trusted": false, "release": 9}
        });
        let json = serde_json::to_string(&value).unwrap();
        let hash = format!("{:x}", Sha256::digest(json.as_bytes()));
        game.connection.execute(
            "UPDATE save_slots SET state_json=?1, checksum=?2, autosave_json=?1, autosave_checksum=?2 WHERE slot_index=1",
            rusqlite::params![json, hash],
        ).unwrap();
        game.connection
            .execute(
                "UPDATE save_checkpoints SET state_json=?1, checksum=?2 WHERE slot_index=1",
                rusqlite::params![json, hash],
            )
            .unwrap();
        assert!(save::decode(&json, "invalid checksum").is_err());
        let check = |world: &WorldState| {
            assert_eq!(world.nickname, "legacy");
            assert_eq!(world.money, 123);
            assert_eq!(world.playtime_seconds, 456);
            assert_eq!(
                world.vfs.read("/home/kali/notes.txt", "kali").unwrap(),
                "keep this"
            );
            let repos = &world.packages.repositories;
            assert!(repos["legacy"].available && repos["legacy"].trusted);
            assert_eq!(repos["legacy"].release, 1);
            assert!(repos["untrusted"].available);
            assert!(!repos["untrusted"].trusted);
            assert_eq!(repos["untrusted"].release, 7);
            assert!(!repos["offline"].available && !repos["offline"].trusted);
            assert_eq!(repos["offline"].release, 9);
        };
        for manual in [false, true] {
            check(&game.load(1, manual).unwrap());
        }
        let id = save::checkpoints(&game.connection, 1).unwrap()[0]
            .id
            .clone();
        check(&game.restore(&id).unwrap());
        check(&game.load(1, false).unwrap());
        let saved: String = game
            .connection
            .query_row(
                "SELECT autosave_json FROM save_slots WHERE slot_index=1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&saved).unwrap()["packages"]["repositories"]
                ["legacy"]["available"],
            true
        );
        assert_eq!(
            save::encode(&save::load(&game.connection, 2, false).unwrap().1).unwrap(),
            other
        );
        value["packages"]["repositories"]["legacy"]["available"] = serde_json::json!("invalid");
        assert!(serde_json::from_value::<WorldState>(value).is_err());
    }
    #[test]
    fn five_slots_roundtrip_corruption_and_atomic_rollback() {
        let mut game = service();
        assert_eq!(save::list(&game.connection).expect("list").len(), 5);
        for slot in [0, 6, -1, 100] {
            assert!(game.new_game(slot, "neo", "pc", false).is_err());
        }
        game.new_game(1, "neo", "pc", false).expect("new");
        assert!(game.new_game(1, "new", "pc", false).is_err());
        game.mutate(
            |_, w, _| {
                w.vfs.write("/home/kali/notes.txt", "persisted", "kali")?;
                Ok(())
            },
            true,
        )
        .expect("mutate");
        game.load(1, false).expect("load");
        assert_eq!(
            game.world()
                .expect("world")
                .vfs
                .read("/home/kali/notes.txt", "kali")
                .expect("read"),
            "persisted"
        );
        game.connection.execute_batch("CREATE TRIGGER reject_save BEFORE UPDATE ON save_slots BEGIN SELECT RAISE(ABORT,'test failure'); END;").expect("trigger");
        assert!(game
            .mutate(
                |_, w, _| {
                    w.money = 999;
                    Ok(())
                },
                false
            )
            .is_err());
        assert_eq!(game.world().expect("world").money, 0);
        game.connection.execute_batch("DROP TRIGGER reject_save; UPDATE save_slots SET autosave_json='{}' WHERE slot_index=1;").expect("corrupt");
        assert!(game.load(1, false).is_err());
        assert!(game.load(1, true).is_ok());
    }
    #[test]
    fn new_game_overwrite_requires_confirmation_and_replaces_only_the_selected_slot() {
        let mut game = service();
        game.new_game(2, "other", "other-pc", false).unwrap();
        game.new_game(1, "previous", "old-pc", false).unwrap();
        game.mutate(
            |_, world, _| {
                world.money = 123;
                world.playtime_seconds = 3600;
                world
                    .vfs
                    .write("/home/kali/old-save.txt", "previous campaign", "kali")?;
                Ok(())
            },
            true,
        )
        .unwrap();
        game.save(false).unwrap();

        let snapshots = |game: &GameService, slot| {
            [false, true].map(|manual| {
                let (label, world) = save::load(&game.connection, slot, manual).unwrap();
                (label, save::encode(&world).unwrap())
            })
        };
        let checkpoints = |game: &GameService, slot| {
            serde_json::to_string(&save::checkpoints(&game.connection, slot).unwrap()).unwrap()
        };
        let old_snapshots = snapshots(&game, 1);
        let old_checkpoints = checkpoints(&game, 1);
        let other_snapshots = snapshots(&game, 2);
        let other_checkpoints = checkpoints(&game, 2);
        let old_active = save::encode(game.world().unwrap()).unwrap();

        assert!(game.new_game(1, "replacement", "new-pc", false).is_err());
        assert_eq!(snapshots(&game, 1), old_snapshots);
        assert_eq!(checkpoints(&game, 1), old_checkpoints);
        assert_eq!(save::encode(game.world().unwrap()).unwrap(), old_active);

        game.connection.execute_batch("CREATE TRIGGER reject_replacement BEFORE UPDATE ON save_slots WHEN OLD.slot_index=1 BEGIN SELECT RAISE(ABORT,'test failure'); END;").unwrap();
        assert!(game.new_game(1, "replacement", "new-pc", true).is_err());
        assert_eq!(snapshots(&game, 1), old_snapshots);
        assert_eq!(checkpoints(&game, 1), old_checkpoints);
        assert_eq!(save::encode(game.world().unwrap()).unwrap(), old_active);
        game.connection
            .execute_batch("DROP TRIGGER reject_replacement;")
            .unwrap();

        let replacement = game.new_game(1, "replacement", "new-pc", true).unwrap();
        assert_eq!(replacement.money, 0);
        assert_eq!(replacement.playtime_seconds, 0);
        assert!(replacement
            .vfs
            .read("/home/kali/old-save.txt", "kali")
            .is_err());
        let expected = (
            "replacement".to_owned(),
            save::encode(&replacement).unwrap(),
        );
        assert_eq!(snapshots(&game, 1), [expected.clone(), expected]);
        let new_checkpoints = save::checkpoints(&game.connection, 1).unwrap();
        assert_eq!(new_checkpoints.len(), 1);
        assert_eq!(new_checkpoints[0].checkpoint_type, "mission_start");
        assert_eq!(new_checkpoints[0].mission_id.as_deref(), Some("first-boot"));
        assert!(!old_checkpoints.contains(&new_checkpoints[0].id));
        assert_eq!(snapshots(&game, 2), other_snapshots);
        assert_eq!(checkpoints(&game, 2), other_checkpoints);
    }

    #[test]
    fn deleting_a_save_is_confirmed_atomic_and_keeps_other_slots() {
        let mut game = service();
        game.new_game(1, "first", "pc", false).unwrap();
        game.save(false).unwrap();
        game.new_game(2, "other", "pc", false).unwrap();
        let saved = save::load(&game.connection, 1, false).unwrap();
        let first_checkpoints =
            serde_json::to_string(&save::checkpoints(&game.connection, 1).unwrap()).unwrap();
        let other = save::encode(game.world().unwrap()).unwrap();
        let other_checkpoints =
            serde_json::to_string(&save::checkpoints(&game.connection, 2).unwrap()).unwrap();
        let epoch = game.runtime_epoch;
        assert!(game.delete_slot(0, true).is_err());
        assert!(game.delete_slot(6, true).is_err());
        assert!(game.delete_slot(1, false).is_err());
        assert_eq!(
            save::encode(&save::load(&game.connection, 1, false).unwrap().1).unwrap(),
            save::encode(&saved.1).unwrap()
        );

        game.connection.execute_batch("CREATE TRIGGER reject_delete BEFORE UPDATE ON save_slots WHEN OLD.slot_index=1 BEGIN SELECT RAISE(ABORT,'test failure'); END;").unwrap();
        assert!(game.delete_slot(1, true).is_err());
        assert_eq!(
            serde_json::to_string(&save::checkpoints(&game.connection, 1).unwrap()).unwrap(),
            first_checkpoints
        );
        assert_eq!(
            save::encode(&save::load(&game.connection, 1, false).unwrap().1).unwrap(),
            save::encode(&saved.1).unwrap()
        );
        assert_eq!(game.runtime_epoch, epoch);
        game.connection
            .execute_batch("DROP TRIGGER reject_delete;")
            .unwrap();

        let slots = game.delete_slot(1, true).unwrap();
        assert_eq!(slots.len(), 5);
        assert!(!slots[0].occupied);
        assert_eq!(slots[0].label, "");
        assert_eq!(slots[0].playtime_seconds, 0);
        assert!(slots[0].updated_at.is_none());
        assert!(save::load(&game.connection, 1, false).is_err());
        assert!(save::load(&game.connection, 1, true).is_err());
        assert!(save::checkpoints(&game.connection, 1).unwrap().is_empty());
        assert_eq!(save::encode(game.world().unwrap()).unwrap(), other);
        assert_eq!(game.runtime_epoch, epoch);
        assert_eq!(
            serde_json::to_string(&save::checkpoints(&game.connection, 2).unwrap()).unwrap(),
            other_checkpoints
        );

        game.new_game(1, "reused", "new-pc", false).unwrap();
        let epoch = game.runtime_epoch;
        game.delete_slot(1, true).unwrap();
        assert!(game.active.is_none());
        assert_eq!(game.runtime_epoch, epoch + 1);
        assert!(game.save(false).is_err());
        game.end_session().unwrap();
        assert!(!save::list(&game.connection).unwrap()[0].occupied);
        game.load(2, true).unwrap();
        assert_eq!(game.world().unwrap().nickname, "other");
    }

    #[test]
    fn checkpoints_rotate_restore_and_isolate_slots() {
        let mut game = service();
        game.new_game(1, "neo", "pc", false).expect("new");
        for _ in 0..25 {
            game.save(false).expect("autosave");
        }
        let checkpoints = save::checkpoints(&game.connection, 1).expect("list");
        assert_eq!(checkpoints.len(), 20);
        game.mutate(
            |_, w, _| {
                w.money = 25;
                Ok(())
            },
            false,
        )
        .expect("mutation");
        game.restore(&checkpoints[0].id).expect("restore");
        assert_eq!(game.world().expect("world").money, 0);
        game.new_game(2, "two", "pc", false).expect("new");
        assert!(game.restore(&checkpoints[0].id).is_err());
        db::migrate(&mut game.connection).expect("idempotent migration");
        assert_eq!(save::list(&game.connection).expect("slots").len(), 5);
    }

    #[test]
    fn session_exit_rolls_back_active_attempt_and_mission_end_promotes_manual_save() {
        let mut game = service();
        game.new_game(1, "neo", "pc", false).expect("new");
        game.mutate(
            |_, world, _| {
                world
                    .vfs
                    .write("/home/kali/Documents/notes.txt", "pronto", "kali")?;
                Ok(())
            },
            false,
        )
        .expect("complete first boot");
        game.load(1, true).expect("manual mission boundary");
        game.mutate(
            |engine, world, events| {
                events.push(engine.start(world, "v1")?);
                Ok(())
            },
            false,
        )
        .expect("start mission");
        game.mutate(
            |_, world, _| {
                world
                    .vfs
                    .write("/home/kali/Documents/attempt.txt", "temporary", "kali")?;
                Ok(())
            },
            false,
        )
        .expect("mission progress");
        game.end_session().expect("end session");
        game.load(1, false).expect("load autosave");
        assert!(!game.world().expect("world").missions.contains_key("v1"));
        assert!(game
            .world()
            .expect("world")
            .vfs
            .nodes
            .contains_key("/home/kali/Documents/attempt.txt")); // Personal file, not mission progress.
        game.load(1, true).expect("load promoted manual save");
        assert!(game
            .world()
            .expect("world")
            .flags
            .contains("FIRST_BOOT_COMPLETE"));
    }
}
