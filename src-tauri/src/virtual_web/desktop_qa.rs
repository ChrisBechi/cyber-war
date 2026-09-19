//! Explicitly opted-in local QA. Never available in release or against player saves.
#![cfg(debug_assertions)]
use crate::{error::GameResult, service::GameService, vfs::domain};
use serde_json::{json, Value};
use std::path::PathBuf;
fn io<T>(result: std::io::Result<T>) -> GameResult<T> {
    result.map_err(|error| domain(format!("QA report: {error}")))
}

fn directory(game: &GameService) -> GameResult<PathBuf> {
    let path = option_env!("CYBER_WAR_QA_DATA_DIR")
        .map(PathBuf::from)
        .ok_or_else(|| domain("Compile an isolated QA build first."))?;
    let database = io(path.join("game-hacker.db").canonicalize())?;
    if game
        .connection
        .path()
        .map(PathBuf::from)
        .and_then(|p| p.canonicalize().ok())
        != Some(database)
    {
        return Err(domain("QA cannot operate on the player database."));
    }
    Ok(path)
}

pub fn prepare(game: &mut GameService) -> GameResult<Value> {
    let path = directory(game)?;
    game.end_session()?;
    for slot in 1..=5 {
        let occupied: bool = game.connection.query_row(
            "SELECT occupied FROM save_slots WHERE slot_index=?1",
            [slot],
            |r| r.get(0),
        )?;
        if !occupied {
            game.new_game(slot, &format!("webqa{slot}"), "qa-pc", false)?;
            game.mutate(
                |_, world, _| {
                    world.network.connected = true;
                    world.flags.extend([
                        "V1_COMPLETE".into(),
                        "WIFI_COMPLETE".into(),
                        "SESSION_1_COMPLETE".into(),
                        "GAME_DOWNLOADED".into(),
                    ]);
                    world.missions.clear();
                    world.playtime_seconds = 700;
                    super::sync_events(world);
                    Ok(())
                },
                false,
            )?;
            game.end_session()?;
        }
    }
    game.load(1, false)?;
    game.start_session()?;
    Ok(json!({"directory":path,"slots":5}))
}

pub fn report(game: &GameService, sample: Value) -> GameResult<Value> {
    let directory = directory(game)?;
    if serde_json::to_vec(&sample)?.len() > 128_000 {
        return Err(domain("QA report too large."));
    }
    let world = game.world()?;
    let db_pages: u64 = game
        .connection
        .query_row("PRAGMA page_count", [], |r| r.get(0))?;
    let db_page_size: u64 = game
        .connection
        .query_row("PRAGMA page_size", [], |r| r.get(0))?;
    let quick_check: String = game
        .connection
        .query_row("PRAGMA quick_check", [], |r| r.get(0))?;
    let cache = world.search.cache.lock().ok().map(|c| c.diagnostics());
    let report = json!({"sample":sample,"backend":{"slot":game.active.as_ref().map(|a|a.slot),"history":world.web.history.len(),"materializationCache":super::materialize::cache_len(),"searchCache":cache,"deltaBytes":serde_json::to_vec(&world.web)?.len(),"databaseBytes":db_pages*db_page_size,"quickCheck":quick_check,"autocommit":game.connection.is_autocommit(),"connectionOwnership":"one GameService connection guarded by Mutex"}});
    let line = serde_json::to_string(&report)?;
    use std::io::Write;
    let mut file = io(std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(directory.join("web-soak.jsonl")))?;
    io(writeln!(file, "{line}"))?;
    io(std::fs::write(
        directory.join("web-soak-latest.json"),
        serde_json::to_vec_pretty(&report)?,
    ))?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qa_commands_reject_a_database_outside_the_opted_in_directory() {
        let mut game = GameService::new(rusqlite::Connection::open_in_memory().unwrap()).unwrap();
        game.new_game(1, "player", "pc", false).unwrap();
        let before = serde_json::to_string(game.world().unwrap()).unwrap();
        assert!(prepare(&mut game).is_err());
        assert!(report(&game, json!({"state":"started"})).is_err());
        assert_eq!(
            before,
            serde_json::to_string(game.world().unwrap()).unwrap()
        );
        assert!(!game.active.as_ref().unwrap().session_ended);
    }
}
