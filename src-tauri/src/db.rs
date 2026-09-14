use crate::error::GameResult;
use rusqlite::{params, Connection};

pub fn migrate(connection: &mut Connection) -> GameResult<()> {
    connection.pragma_update(None, "foreign_keys", "ON")?;
    connection.pragma_update(None, "journal_mode", "WAL")?;
    let tx = connection.transaction()?;
    tx.execute_batch(
        "CREATE TABLE IF NOT EXISTS save_slots (
        slot_index INTEGER PRIMARY KEY CHECK(slot_index BETWEEN 1 AND 5),
        label TEXT NOT NULL DEFAULT '', occupied INTEGER NOT NULL DEFAULT 0,
        current_mission TEXT, playtime_seconds INTEGER NOT NULL DEFAULT 0,
        state_json TEXT, checksum TEXT, created_at TEXT, updated_at TEXT
    );
    CREATE TABLE IF NOT EXISTS save_checkpoints (
        id TEXT PRIMARY KEY, slot_index INTEGER NOT NULL REFERENCES save_slots(slot_index),
        checkpoint_type TEXT NOT NULL, mission_id TEXT, label TEXT NOT NULL,
        state_json TEXT NOT NULL, checksum TEXT NOT NULL, created_at TEXT NOT NULL
    );
    CREATE INDEX IF NOT EXISTS checkpoints_slot ON save_checkpoints(slot_index);
    CREATE TABLE IF NOT EXISTS app_preferences (key TEXT PRIMARY KEY, value TEXT NOT NULL);",
    )?;
    let has_autosave = {
        let mut statement = tx.prepare("PRAGMA table_info(save_slots)")?;
        let columns = statement
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<Result<Vec<_>, _>>()?;
        columns.iter().any(|name| name == "autosave_json")
    };
    if !has_autosave {
        tx.execute_batch("ALTER TABLE save_slots ADD COLUMN autosave_json TEXT; ALTER TABLE save_slots ADD COLUMN autosave_checksum TEXT;")?;
    }
    for slot in 1..=5 {
        tx.execute(
            "INSERT OR IGNORE INTO save_slots(slot_index) VALUES (?1)",
            [slot],
        )?;
    }
    tx.execute_batch("CREATE TABLE IF NOT EXISTS vfs_blobs (hash TEXT PRIMARY KEY CHECK(length(hash)=64), data BLOB NOT NULL CHECK(length(data)<=33554432));")?;
    tx.pragma_update(None, "user_version", 3)?;
    tx.commit()?;
    Ok(())
}

pub fn validate_slot(slot: i64) -> GameResult<()> {
    if !(1..=5).contains(&slot) {
        return Err(crate::error::GameError::InvalidSaveSlot(slot));
    }
    Ok(())
}

pub fn write_snapshot(
    tx: &rusqlite::Transaction<'_>,
    slot: i64,
    label: &str,
    world: &crate::world::WorldState,
    manual: bool,
) -> GameResult<()> {
    let (json, checksum) = crate::save::encode(world)?;
    crate::binary::persist(tx, world, &json)?;
    let mission = world
        .missions
        .iter()
        .find(|(_, p)| p.status == "active")
        .map(|(id, _)| id.clone());
    let now = chrono::Utc::now().to_rfc3339();
    tx.execute("UPDATE save_slots SET label=?2, occupied=1, current_mission=?3, playtime_seconds=?4,
        autosave_json=?5, autosave_checksum=?6, created_at=COALESCE(created_at,?7), updated_at=?7 WHERE slot_index=?1",
        params![slot, label, mission, world.playtime_seconds, json, checksum, now])?;
    if manual {
        tx.execute(
            "UPDATE save_slots SET state_json=?2, checksum=?3 WHERE slot_index=?1",
            params![slot, json, checksum],
        )?;
    }
    Ok(())
}
