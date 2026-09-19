//! Per-attempt effect journal. Never stores or restores an entire WorldState.
//! Personal settings, packages, forum activity and network profiles are not in
//! the journal. Only declared mission resources and explicit narrative effects are.
use crate::{error::GameResult, vfs::domain, world::WorldState};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Resource {
    Flag { key: String },
    Technique { key: String },
    Inventory { key: String },
    Decision { key: String },
    SearchDocument { id: String },
    SearchSuggestion { id: String },
    SearchRanking { id: String },
    File { host: Option<String>, path: String },
    Message { id: String },
    Contact { name: String },
    Evidence { key: String },
    Service { host: String, port: u16 },
    Patched { host: String },
    Wifi { bssid: String },
    Money,
    Reputation,
    Heat,
    Session,
    Connection,
    MemoryValue,
    MemoryCandidates,
}

impl Resource {
    pub fn file(path: &str) -> Self {
        Self::File {
            host: None,
            path: path.into(),
        }
    }
    pub fn read(&self, w: &WorldState) -> Option<Value> {
        match self {
            Self::Flag { key } => Some(json!(w.flags.contains(key))),
            Self::Technique { key } => Some(json!(w.techniques.contains(key))),
            Self::Inventory { key } => Some(json!(w.inventory.contains(key))),
            Self::Decision { key } => w.decisions.get(key).map(|v| json!(v)),
            Self::SearchDocument { id } => w.search.documents.get(id).map(|v| json!(v)),
            Self::SearchSuggestion { id } => w.search.suggestions.get(id).map(|v| json!(v)),
            Self::SearchRanking { id } => w.search.ranking.get(id).map(|v| json!(v)),
            Self::File { host, path } => match host {
                Some(host) => w
                    .network
                    .hosts
                    .get(host)
                    .and_then(|h| h.files.nodes.get(path)),
                None => w.vfs.nodes.get(path),
            }
            .map(|v| json!(v)),
            Self::Message { id } => w.messages.iter().find(|m| m.id == *id).map(|m| json!(m)),
            Self::Contact { name } => Some(json!(w.contacts.contains(name))),
            Self::Evidence { key } => Some(json!(w.evidence.contains(key))),
            Self::Service { host, port } => w
                .network
                .hosts
                .get(host)
                .and_then(|h| h.services.iter().find(|s| s.port == *port))
                .map(|s| json!(s)),
            Self::Patched { host } => w.network.hosts.get(host).map(|h| json!(h.patched)),
            Self::Wifi { bssid } => w
                .network
                .wifi
                .iter()
                .find(|v| v.bssid == *bssid)
                .map(|v| json!(v)),
            Self::Money => Some(json!(w.money)),
            Self::Reputation => Some(json!(w.reputation)),
            Self::Heat => Some(json!(w.heat)),
            Self::Session => Some(json!(w.session)),
            Self::Connection => Some(json!(w.network.connected)),
            Self::MemoryValue => Some(json!(w.memory_value)),
            Self::MemoryCandidates => Some(json!(w.memory_candidates)),
        }
    }
    fn additive(&self) -> bool {
        matches!(self, Self::Money | Self::Reputation | Self::Heat)
    }
    pub fn write(&self, w: &mut WorldState, value: Option<Value>) -> GameResult<()> {
        let present = value.as_ref().and_then(Value::as_bool).unwrap_or(false);
        let required = || {
            value
                .clone()
                .ok_or_else(|| domain("mission journal value missing"))
        };
        match self {
            Self::Flag { key } => {
                if present {
                    w.flags.insert(key.clone());
                } else {
                    w.flags.remove(key);
                }
            }
            Self::Technique { key } => {
                if present {
                    w.techniques.insert(key.clone());
                } else {
                    w.techniques.remove(key);
                }
            }
            Self::Inventory { key } => {
                if present {
                    w.inventory.insert(key.clone());
                } else {
                    w.inventory.remove(key);
                }
            }
            Self::Decision { key } => match value {
                Some(v) => {
                    w.decisions.insert(key.clone(), serde_json::from_value(v)?);
                }
                None => {
                    w.decisions.remove(key);
                }
            },
            Self::SearchDocument { id } => match value {
                Some(v) => {
                    w.search
                        .documents
                        .insert(id.clone(), serde_json::from_value(v)?);
                }
                None => {
                    w.search.documents.remove(id);
                }
            },
            Self::SearchSuggestion { id } => match value {
                Some(v) => {
                    w.search
                        .suggestions
                        .insert(id.clone(), serde_json::from_value(v)?);
                }
                None => {
                    w.search.suggestions.remove(id);
                }
            },
            Self::SearchRanking { id } => match value {
                Some(v) => {
                    w.search
                        .ranking
                        .insert(id.clone(), serde_json::from_value(v)?);
                }
                None => {
                    w.search.ranking.remove(id);
                }
            },
            Self::File { host, path } => {
                if host.is_none()
                    && (w.packages.ownership.contains_key(path)
                        || w.vfs
                            .nodes
                            .get(path)
                            .is_some_and(|n| n.metadata.contains_key("packageProjection")))
                {
                    return Ok(());
                }
                let fs = match host {
                    Some(host) => {
                        &mut w
                            .network
                            .hosts
                            .get_mut(host)
                            .ok_or_else(|| domain("mission host missing"))?
                            .files
                    }
                    None => &mut w.vfs,
                };
                match value {
                    Some(v) => {
                        fs.restore_entry(path, Some(serde_json::from_value(v)?));
                    }
                    None => {
                        fs.restore_entry(path, None);
                    }
                }
            }
            Self::Message { id } => {
                let index = w.messages.iter().position(|m| m.id == *id);
                match (index, value) {
                    (Some(i), Some(v)) => w.messages[i] = serde_json::from_value(v)?,
                    (None, Some(v)) => w.messages.push(serde_json::from_value(v)?),
                    (Some(i), None) => {
                        w.messages.remove(i);
                    }
                    (None, None) => {}
                }
            }
            Self::Contact { name } => {
                if present {
                    w.contacts.insert(name.clone());
                } else if !w.messages.iter().any(|m| m.contact == *name) {
                    w.contacts.remove(name);
                }
            }
            Self::Evidence { key } => {
                w.evidence.retain(|v| v != key);
                if present {
                    w.evidence.push(key.clone());
                }
            }
            Self::Service { host, port } => {
                let h = w
                    .network
                    .hosts
                    .get_mut(host)
                    .ok_or_else(|| domain("mission host missing"))?;
                h.services.retain(|s| s.port != *port);
                if let Some(v) = value {
                    h.services.push(serde_json::from_value(v)?);
                }
                h.services.sort_by_key(|s| s.port);
            }
            Self::Patched { host } => {
                w.network
                    .hosts
                    .get_mut(host)
                    .ok_or_else(|| domain("mission host missing"))?
                    .patched = present;
            }
            Self::Wifi { bssid } => {
                let index = w.network.wifi.iter().position(|v| v.bssid == *bssid);
                match (index, value) {
                    (Some(i), Some(v)) => w.network.wifi[i] = serde_json::from_value(v)?,
                    (None, Some(v)) => w.network.wifi.push(serde_json::from_value(v)?),
                    (Some(i), None) => {
                        w.network.wifi.remove(i);
                    }
                    (None, None) => {}
                }
            }
            Self::Money => w.money = serde_json::from_value(required()?)?,
            Self::Reputation => w.reputation = serde_json::from_value(required()?)?,
            Self::Heat => w.heat = serde_json::from_value(required()?)?,
            Self::Session => w.session = serde_json::from_value(required()?)?,
            Self::Connection => w.network.connected = serde_json::from_value(required()?)?,
            Self::MemoryValue => w.memory_value = serde_json::from_value(required()?)?,
            Self::MemoryCandidates => w.memory_candidates = serde_json::from_value(required()?)?,
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attempt {
    pub id: String,
    pub number: u32,
    pub scope: Vec<Resource>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Layer {
    /// None is a committed effect, retained only while an overlapping attempt exists.
    owner: Option<String>,
    value: Option<Value>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Entry {
    resource: Resource,
    base: Option<Value>,
    layers: Vec<Layer>,
}
impl Entry {
    fn value(&self) -> GameResult<Option<Value>> {
        if !self.resource.additive() {
            return Ok(self
                .layers
                .last()
                .map(|l| l.value.clone())
                .unwrap_or_else(|| self.base.clone()));
        }
        let mut n = self
            .base
            .as_ref()
            .and_then(Value::as_i64)
            .ok_or_else(|| domain("invalid journal counter"))?;
        for layer in &self.layers {
            n = n
                .checked_add(
                    layer
                        .value
                        .as_ref()
                        .and_then(Value::as_i64)
                        .ok_or_else(|| domain("invalid journal delta"))?,
                )
                .ok_or_else(|| domain("mission counter overflow"))?;
        }
        Ok(Some(json!(n)))
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionRuntime {
    pub attempts: BTreeMap<String, Attempt>,
    pub attempt_counts: BTreeMap<String, u32>,
    pub journal: Vec<Entry>,
}
impl MissionRuntime {
    pub fn begin(&mut self, mission: &str, scope: Vec<Resource>) {
        let count = self.attempt_counts.entry(mission.into()).or_default();
        *count += 1;
        self.attempts.insert(
            mission.into(),
            Attempt {
                id: uuid::Uuid::new_v4().to_string(),
                number: *count,
                scope,
            },
        );
    }
    pub fn record(
        &mut self,
        mission: &str,
        resource: Resource,
        before: Option<Value>,
        after: Option<Value>,
    ) -> GameResult<()> {
        let owner = self
            .attempts
            .get(mission)
            .ok_or_else(|| domain("mission attempt missing"))?
            .id
            .clone();
        self.record_owner(Some(owner), resource, before, after)
    }
    fn record_owner(
        &mut self,
        owner: Option<String>,
        resource: Resource,
        before: Option<Value>,
        after: Option<Value>,
    ) -> GameResult<()> {
        let value = if resource.additive() {
            let a = after
                .as_ref()
                .and_then(Value::as_i64)
                .ok_or_else(|| domain("invalid journal counter"))?;
            let b = before
                .as_ref()
                .and_then(Value::as_i64)
                .ok_or_else(|| domain("invalid journal counter"))?;
            Some(json!(a
                .checked_sub(b)
                .ok_or_else(|| domain("mission counter overflow"))?))
        } else {
            after
        };
        let index = match self.journal.iter().position(|e| e.resource == resource) {
            Some(index) => index,
            None => {
                self.journal.push(Entry {
                    resource: resource.clone(),
                    base: before,
                    layers: Vec::new(),
                });
                self.journal.len() - 1
            }
        };
        let entry = &mut self.journal[index];
        if let Some(last) = entry.layers.last_mut().filter(|l| l.owner == owner) {
            last.value = if resource.additive() {
                Some(json!(last
                    .value
                    .as_ref()
                    .and_then(Value::as_i64)
                    .unwrap_or(0)
                    .checked_add(value.as_ref().and_then(Value::as_i64).unwrap_or(0))
                    .ok_or_else(|| domain("mission counter overflow"))?))
            } else {
                value
            };
        } else {
            entry.layers.push(Layer { owner, value });
        }
        if self.journal.len() > 4096 {
            return Err(domain("mission journal resource limit exceeded"));
        }
        Ok(())
    }
    pub fn entry(&self, resource: &Resource) -> Option<&Entry> {
        self.journal.iter().find(|e| &e.resource == resource)
    }
    pub fn commit(&mut self, mission: &str) {
        if let Some(attempt) = self.attempts.remove(mission) {
            for entry in &mut self.journal {
                for layer in &mut entry.layers {
                    if layer.owner.as_ref() == Some(&attempt.id) {
                        layer.owner = None;
                    }
                }
            }
            self.compact();
        }
    }
    fn compact(&mut self) {
        self.journal
            .retain(|entry| entry.layers.iter().any(|l| l.owner.is_some()));
    }
}

const ATTEMPT_TAG: &str = "missionAttempt";

/// Preserve provenance through VFS copy/move/trash operations, which already
/// preserve node metadata. Ordinary personal files never receive this marker.
pub fn record_change(
    world: &mut WorldState,
    mission: &str,
    resource: Resource,
    before: Option<Value>,
) -> GameResult<()> {
    let attempt = world
        .mission_runtime
        .attempts
        .get(mission)
        .ok_or_else(|| domain("mission attempt missing"))?
        .id
        .clone();
    if let Resource::File { host, path } = &resource {
        let fs = match host {
            Some(host) => {
                &mut world
                    .network
                    .hosts
                    .get_mut(host)
                    .ok_or_else(|| domain("mission host missing"))?
                    .files
            }
            None => &mut world.vfs,
        };
        if let Some(mut node) = fs.nodes.get_mut(path) {
            node.metadata.insert(ATTEMPT_TAG.into(), attempt);
        }
    }
    let after = resource.read(world);
    world
        .mission_runtime
        .record(mission, resource, before, after)
}
pub fn inherited_files(world: &WorldState, attempt: &str) -> Vec<Resource> {
    let mut resources = Vec::new();
    for (path, node) in &world.vfs.nodes {
        if node
            .metadata
            .get(ATTEMPT_TAG)
            .is_some_and(|id| id == attempt)
        {
            resources.push(Resource::file(path));
        }
    }
    for (host, state) in &world.network.hosts {
        for (path, node) in &state.files.nodes {
            if node
                .metadata
                .get(ATTEMPT_TAG)
                .is_some_and(|id| id == attempt)
            {
                resources.push(Resource::File {
                    host: Some(host.clone()),
                    path: path.clone(),
                });
            }
        }
    }
    resources
}
pub fn commit(world: &mut WorldState, mission: &str) {
    if let Some(attempt) = world.mission_runtime.attempts.get(mission) {
        let id = attempt.id.clone();
        for fs in std::iter::once(&mut world.vfs)
            .chain(world.network.hosts.values_mut().map(|h| &mut h.files))
        {
            fs.update_all_metadata(|node| {
                if node.metadata.get(ATTEMPT_TAG) == Some(&id) {
                    node.metadata.remove(ATTEMPT_TAG);
                }
            });
        }
    }
    world.mission_runtime.commit(mission);
}

/// Forget just this attempt. Committed layers and other attempts remain intact.
pub fn discard(world: &mut WorldState, mission: &str) -> GameResult<()> {
    let mut runtime = std::mem::take(&mut world.mission_runtime);
    let result = (|| {
        let attempt = runtime
            .attempts
            .remove(mission)
            .ok_or_else(|| domain("mission attempt missing"))?;
        for entry in &mut runtime.journal {
            if !entry
                .layers
                .iter()
                .any(|l| l.owner.as_ref() == Some(&attempt.id))
            {
                continue;
            }
            // A durable write outside a mission supersedes earlier temporary effects.
            let current = entry.resource.read(world);
            if current != entry.value()? && !matches!(entry.resource, Resource::Message { .. }) {
                if entry.resource.additive() {
                    let expected = entry
                        .value()?
                        .and_then(|v| v.as_i64())
                        .ok_or_else(|| domain("invalid counter"))?;
                    let actual = current
                        .and_then(|v| v.as_i64())
                        .ok_or_else(|| domain("invalid counter"))?;
                    let base = entry
                        .base
                        .as_ref()
                        .and_then(Value::as_i64)
                        .ok_or_else(|| domain("invalid counter"))?;
                    entry.base = Some(json!(base
                        .checked_add(actual)
                        .and_then(|n| n.checked_sub(expected))
                        .ok_or_else(|| domain("mission counter overflow"))?));
                } else {
                    entry.layers.push(Layer {
                        owner: None,
                        value: current,
                    });
                }
            }
            entry
                .layers
                .retain(|l| l.owner.as_ref() != Some(&attempt.id));
            entry.resource.write(world, entry.value()?)?;
        }
        for entry in &runtime.journal {
            if matches!(entry.resource, Resource::Contact { .. }) {
                entry.resource.write(world, entry.value()?)?;
            }
        }
        runtime.compact();
        world.missions.remove(mission);
        world
            .events
            .push(format!("attempt_discarded:{mission}:{}", attempt.number));
        Ok(())
    })();
    world.mission_runtime = runtime;
    world.vfs.collect();
    for host in world.network.hosts.values_mut() {
        host.files.collect();
    }
    result
}
