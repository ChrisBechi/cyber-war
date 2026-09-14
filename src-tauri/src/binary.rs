//! Immutable, content-addressed bytes. Snapshots contain references, never payloads.
use crate::{
    error::GameResult,
    vfs::{domain, normalize, VirtualFileSystem, HOME},
    world::WorldState,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, sync::Arc};

pub const MAX_BLOB: usize = 32 * 1024 * 1024;
const MAX_REFERENCED: usize = 128 * 1024 * 1024;
const MAX_DATABASE: usize = 512 * 1024 * 1024;
pub type BlobCache = BTreeMap<String, Arc<Vec<u8>>>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlobRef {
    pub hash: String,
    pub size: usize,
    pub mime: String,
}

impl BlobRef {
    pub fn validate(&self) -> GameResult<()> {
        if self.hash.len() != 64
            || !self
                .hash
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
            || self.size > MAX_BLOB
        {
            return Err(domain("invalid binary reference"));
        }
        let Some((kind, subtype)) = self.mime.split_once('/') else {
            return Err(domain("invalid content type"));
        };
        if kind.is_empty()
            || subtype.is_empty()
            || self.mime.len() > 100
            || !self
                .mime
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"/.-+".contains(&b))
        {
            return Err(domain("invalid content type"));
        }
        Ok(())
    }
}

pub fn import(
    world: &mut WorldState,
    path: &str,
    encoded: &str,
    mime: &str,
    expected: Option<u64>,
    actor: &str,
) -> GameResult<()> {
    if encoded.len() > MAX_BLOB.div_ceil(3) * 4 {
        return Err(domain("binary file limit: 32 MiB"));
    }
    let bytes = STANDARD
        .decode(encoded)
        .map_err(|_| domain("invalid base64 payload"))?;
    let reference = BlobRef {
        hash: format!("{:x}", Sha256::digest(&bytes)),
        size: bytes.len(),
        mime: if mime.is_empty() {
            "application/octet-stream".into()
        } else {
            mime.to_ascii_lowercase()
        },
    };
    reference.validate()?;
    let path = normalize(path, HOME)?;
    if VirtualFileSystem::is_trash_path(&path) {
        return Err(domain("restore trash items before replacing them"));
    }
    // Expected version is mandatory for replacement; absent means create-only.
    match (world.vfs.nodes.get(&path), expected) {
        (None, None) => {}
        (Some(node), Some(version)) if node.modified_at == version => {}
        _ => return Err(domain("file exists or changed; reopen before replacing")),
    }
    world.vfs.write_blob(&path, reference.clone(), actor)?;
    world.blobs.insert(reference.hash, Arc::new(bytes));
    Ok(())
}

#[derive(Serialize)]
pub struct BinaryRead {
    pub base64: String,
    pub mime: String,
    pub size: usize,
}
pub fn read(world: &WorldState, path: &str, actor: &str) -> GameResult<BinaryRead> {
    let path = normalize(path, HOME)?;
    if VirtualFileSystem::is_trash_path(&path) {
        return Err(domain("restore trash items before opening"));
    }
    let node = world.vfs.readable(&path, actor)?;
    let reference = node
        .blob
        .as_ref()
        .ok_or_else(|| domain("not a binary file"))?;
    let data = world
        .blobs
        .get(&reference.hash)
        .ok_or_else(|| domain("binary payload unavailable"))?;
    Ok(BinaryRead {
        base64: STANDARD.encode(data.as_slice()),
        mime: reference.mime.clone(),
        size: data.len(),
    })
}

// Includes typed VFS nodes held in mission undo journals and checkpoints, not
// only the visible filesystem. These bytes must survive temporary replacement.
fn references(json: &str) -> GameResult<BTreeMap<String, BlobRef>> {
    fn visit(value: &serde_json::Value, found: &mut BTreeMap<String, BlobRef>) -> GameResult<()> {
        match value {
            serde_json::Value::Object(map) => {
                if map.get("kind").and_then(|v| v.as_str()) == Some("file")
                    && map.contains_key("parentId")
                {
                    if let Some(blob) = map.get("blob").filter(|v| !v.is_null()) {
                        let reference: BlobRef = serde_json::from_value(blob.clone())?;
                        reference.validate()?;
                        if found
                            .get(&reference.hash)
                            .is_some_and(|old| old.size != reference.size)
                        {
                            return Err(domain("conflicting binary sizes"));
                        }
                        found.insert(reference.hash.clone(), reference);
                    }
                }
                for child in map.values() {
                    visit(child, found)?;
                }
            }
            serde_json::Value::Array(values) => {
                for child in values {
                    visit(child, found)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    let mut found = BTreeMap::new();
    visit(&serde_json::from_str(json)?, &mut found)?;
    if found.values().map(|r| r.size).sum::<usize>() > MAX_REFERENCED {
        return Err(domain("snapshot binary limit: 128 MiB"));
    }
    Ok(found)
}

pub fn persist(tx: &Transaction<'_>, world: &WorldState, json: &str) -> GameResult<()> {
    let mut total: usize = tx.query_row(
        "SELECT COALESCE(SUM(length(data)),0) FROM vfs_blobs",
        [],
        |r| r.get(0),
    )?;
    for (hash, reference) in references(json)? {
        let existing: Option<usize> = tx
            .query_row(
                "SELECT length(data) FROM vfs_blobs WHERE hash=?1",
                [&hash],
                |r| r.get(0),
            )
            .optional()?;
        if let Some(size) = existing {
            if size != reference.size {
                return Err(domain("stored binary size mismatch"));
            }
            continue;
        }
        let data = world
            .blobs
            .get(&hash)
            .ok_or_else(|| domain("missing binary payload; snapshot not saved"))?;
        if data.len() != reference.size || format!("{:x}", Sha256::digest(data.as_slice())) != hash
        {
            return Err(domain("binary integrity failure"));
        }
        total += data.len();
        if total > MAX_DATABASE {
            return Err(domain(
                "binary storage capacity: 512 MiB including saved checkpoints",
            ));
        }
        tx.execute(
            "INSERT INTO vfs_blobs(hash,data) VALUES (?1,?2)",
            params![hash, data.as_slice()],
        )?;
    }
    Ok(())
}

pub fn hydrate(connection: &Connection, world: &mut WorldState, json: &str) -> GameResult<()> {
    let mut cache = BlobCache::new();
    for (hash, reference) in references(json)? {
        let size: Option<usize> = connection
            .query_row(
                "SELECT length(data) FROM vfs_blobs WHERE hash=?1",
                [&hash],
                |r| r.get(0),
            )
            .optional()?;
        if size != Some(reference.size) {
            return Err(domain("missing or corrupt binary payload; save preserved"));
        }
        let data: Vec<u8> =
            connection.query_row("SELECT data FROM vfs_blobs WHERE hash=?1", [&hash], |r| {
                r.get(0)
            })?;
        if format!("{:x}", Sha256::digest(&data)) != hash {
            return Err(domain("binary integrity failure; save preserved"));
        }
        cache.insert(hash, Arc::new(data));
    }
    world.blobs = cache;
    Ok(())
}
