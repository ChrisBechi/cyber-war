use crate::{
    db::validate_slot,
    error::{GameError, GameResult},
    mission::CheckpointEvent,
    vfs::domain,
    world::WorldState,
};
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde::Serialize;
use sha2::{Digest, Sha256};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSlotSummary {
    pub slot_index: i64,
    pub label: String,
    pub occupied: bool,
    pub current_mission: Option<String>,
    pub playtime_seconds: u64,
    pub updated_at: Option<String>,
    pub session: Option<u8>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckpointSummary {
    pub id: String,
    pub checkpoint_type: String,
    pub mission_id: Option<String>,
    pub label: String,
    pub created_at: String,
}

pub fn encode(world: &WorldState) -> GameResult<(String, String)> {
    let json = serde_json::to_string(world)?;
    let hash = format!("{:x}", Sha256::digest(json.as_bytes()));
    Ok((json, hash))
}
pub fn decode(json: &str, checksum: &str) -> GameResult<WorldState> {
    if format!("{:x}", Sha256::digest(json.as_bytes())) != checksum {
        return Err(domain("snapshot failed integrity check"));
    }
    let world: WorldState = serde_json::from_str(json)?;
    world.validate()?;
    Ok(world)
}
pub fn list(connection: &Connection) -> GameResult<Vec<SaveSlotSummary>> {
    let mut statement = connection.prepare("SELECT slot_index,label,occupied,current_mission,playtime_seconds,updated_at,CASE WHEN json_valid(COALESCE(autosave_json,state_json)) THEN json_extract(COALESCE(autosave_json,state_json),'$.session') ELSE NULL END FROM save_slots ORDER BY slot_index")?;
    let rows = statement.query_map([], |row| {
        Ok(SaveSlotSummary {
            slot_index: row.get(0)?,
            label: row.get(1)?,
            occupied: row.get::<_, i64>(2)? != 0,
            current_mission: row.get(3)?,
            playtime_seconds: row.get(4)?,
            updated_at: row.get(5)?,
            session: row.get(6)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}
pub fn load(connection: &Connection, slot: i64, manual: bool) -> GameResult<(String, WorldState)> {
    validate_slot(slot)?;
    let row = connection.query_row("SELECT label,state_json,checksum,autosave_json,autosave_checksum FROM save_slots WHERE slot_index=?1 AND occupied=1", [slot], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?, row.get::<_, Option<String>>(2)?, row.get::<_, Option<String>>(3)?, row.get::<_, Option<String>>(4)?))
    }).optional()?.ok_or(GameError::EmptySaveSlot(slot))?;
    let (json, checksum) = if manual || row.3.is_none() {
        (row.1, row.2)
    } else {
        (row.3, row.4)
    };
    let json = json.ok_or_else(|| domain("snapshot is missing"))?;
    let mut world = decode(
        &json,
        &checksum.ok_or_else(|| domain("checksum is missing"))?,
    )?;
    crate::binary::hydrate(connection, &mut world, &json)?;
    Ok((row.0, world))
}
pub fn checkpoint(tx: &Transaction<'_>, slot: i64, event: &CheckpointEvent) -> GameResult<String> {
    validate_slot(slot)?;
    if !["mission_start", "mission_end", "decision", "autosave"].contains(&event.kind) {
        return Err(domain("invalid checkpoint type"));
    }
    let id = uuid::Uuid::new_v4().to_string();
    let (json, checksum) = encode(&event.state)?;
    crate::binary::persist(tx, &event.state, &json)?;
    tx.execute("INSERT INTO save_checkpoints(id,slot_index,checkpoint_type,mission_id,label,state_json,checksum,created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)", params![id,slot,event.kind,event.mission,event.label,json,checksum,chrono::Utc::now().to_rfc3339()])?;
    // A sequence tie-breaker keeps retention deterministic even for identical timestamps.
    tx.execute("DELETE FROM save_checkpoints WHERE slot_index=?1 AND rowid NOT IN (SELECT rowid FROM save_checkpoints WHERE slot_index=?1 ORDER BY rowid DESC LIMIT 20)", [slot])?;
    Ok(id)
}
pub fn checkpoints(connection: &Connection, slot: i64) -> GameResult<Vec<CheckpointSummary>> {
    validate_slot(slot)?;
    let mut statement = connection.prepare("SELECT id,checkpoint_type,mission_id,label,created_at FROM save_checkpoints WHERE slot_index=?1 ORDER BY rowid DESC")?;
    let rows = statement.query_map([slot], |r| {
        Ok(CheckpointSummary {
            id: r.get(0)?,
            checkpoint_type: r.get(1)?,
            mission_id: r.get(2)?,
            label: r.get(3)?,
            created_at: r.get(4)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn mission_start(
    connection: &Connection,
    slot: i64,
    mission: &str,
) -> GameResult<(WorldState, i64)> {
    validate_slot(slot)?;
    let (json, checksum, rowid) = connection
        .query_row(
            "SELECT state_json,checksum,rowid FROM save_checkpoints WHERE slot_index=?1 AND checkpoint_type='mission_start' AND mission_id=?2 ORDER BY rowid DESC LIMIT 1",
            params![slot, mission],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            },
        )
        .optional()?
        .ok_or_else(|| domain("mission start checkpoint not found"))?;
    let mut world = decode(&json, &checksum)?;
    crate::binary::hydrate(connection, &mut world, &json)?;
    Ok((world, rowid))
}
pub fn restore(
    connection: &mut Connection,
    slot: i64,
    id: &str,
    label: &str,
    normalize: impl FnOnce(&Connection, &mut WorldState) -> GameResult<()>,
) -> GameResult<WorldState> {
    validate_slot(slot)?;
    let tx = connection.transaction()?;
    let (json, checksum, rowid) = tx
        .query_row(
            "SELECT state_json,checksum,rowid FROM save_checkpoints WHERE id=?1 AND slot_index=?2",
            params![id, slot],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)?,
                ))
            },
        )
        .optional()?
        .ok_or_else(|| domain("checkpoint not found in this campaign"))?;
    let mut world = decode(&json, &checksum)?;
    crate::binary::hydrate(&tx, &mut world, &json)?;
    normalize(&tx, &mut world)?;
    crate::db::write_snapshot(&tx, slot, label, &world, true)?;
    tx.execute(
        "DELETE FROM save_checkpoints WHERE slot_index=?1 AND rowid>?2",
        params![slot, rowid],
    )?;
    tx.commit()?;
    Ok(world)
}
