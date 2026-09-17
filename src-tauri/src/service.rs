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

pub struct ActiveGame {
    pub slot: i64,
    pub label: String,
    pub world: WorldState,
    clock: Instant,
    pub(crate) session_ended: bool,
}
pub struct GameService {
    pub connection: Connection,
    pub engine: MissionEngine,
    pub active: Option<ActiveGame>,
}

impl GameService {
    pub fn new(mut connection: Connection) -> GameResult<Self> {
        db::migrate(&mut connection)?;
        Ok(Self {
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
        Ok(world)
    }
    pub fn load(&mut self, slot: i64, manual: bool) -> GameResult<WorldState> {
        let (label, mut world) = save::load(&self.connection, slot, manual)?;
        normalize_entry(&self.engine, &self.connection, slot, &mut world)?;
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
        let mut events = Vec::new();
        let anchors = world.cwd_anchors();
        let result = change(&self.engine, &mut world, &mut events)?;
        world.repair_cwds(anchors);
        terminal::observe(&mut world);
        self.engine.track_action(&active.world, &mut world)?;
        events.extend(self.engine.evaluate(&mut world)?);
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
        }
        Ok(())
    }
    pub fn commit_system_setup(&mut self) -> GameResult<()> {
        // System settings and installer files are durable, never mission-owned.
        self.save(false)
    }
    pub fn start_session(&mut self) -> GameResult<WorldState> {
        let was_ended = self.active.as_ref().is_some_and(|a| a.session_ended);
        if let Some(active) = &mut self.active {
            active.session_ended = false;
        }
        let result = self.mutate(|_, world, _| terminal::prepare_session_start(world), false);
        if let Err(error) = result {
            if let Some(active) = &mut self.active {
                active.session_ended = was_ended;
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
    world.validate()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn service() -> GameService {
        GameService::new(Connection::open_in_memory().expect("db")).expect("service")
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
