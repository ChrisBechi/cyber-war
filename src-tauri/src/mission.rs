use crate::{
    error::GameResult,
    mission_runtime::{self, Resource},
    vfs::domain,
    world::{MissionProgress, WorldState},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Condition {
    PackageInstalled {
        name: String,
        version: Option<String>,
    },
    PackageEvent {
        event: String,
        package: Option<String>,
    },
    ArchiveEvent {
        event: String,
        path: String,
    },
    Flag {
        key: String,
    },
    FileContains {
        path: String,
        text: String,
    },
    Technique {
        key: String,
    },
    Inventory {
        key: String,
    },
    Money {
        minimum: i64,
    },
    Reputation {
        minimum: i64,
    },
    Decision {
        key: String,
        value: String,
    },
    HostService {
        host: String,
        port: u16,
        running: bool,
    },
}

impl Condition {
    pub fn matches(&self, world: &WorldState) -> bool {
        match self {
            Self::PackageInstalled { name, version } => {
                world.packages.installed.get(name).is_some_and(|p| {
                    p.status == crate::packages::model::Status::Installed
                        && version.as_ref().is_none_or(|v| *v == p.definition.version)
                })
            }
            Self::PackageEvent { event, package } => world.packages.events.iter().any(|e| {
                e.kind == *event
                    && package
                        .as_ref()
                        .is_none_or(|p| e.package.as_ref() == Some(p))
            }),
            Self::ArchiveEvent { event, path } => world
                .archive_events
                .iter()
                .any(|e| e.kind == *event && e.path == *path),
            Self::Flag { key } => world.flags.contains(key),
            Self::FileContains { path, text } => world
                .vfs
                .nodes
                .get(path)
                .is_some_and(|n| n.kind == "file" && n.content.contains(text)),
            Self::Technique { key } => world.techniques.contains(key),
            Self::Inventory { key } => world.inventory.contains(key),
            Self::Money { minimum } => world.money >= *minimum,
            Self::Reputation { minimum } => world.reputation >= *minimum,
            Self::Decision { key, value } => world.decisions.get(key) == Some(value),
            Self::HostService {
                host,
                port,
                running,
            } => world.network.hosts.get(host).is_some_and(|h| {
                h.services
                    .iter()
                    .any(|s| s.port == *port && s.running == *running)
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Effect {
    Search {
        effect: crate::search::mission_modifiers::SearchEffect,
    },
    Repository {
        id: String,
        available: bool,
        release: u32,
        trusted: bool,
    },
    Flag {
        key: String,
    },
    File {
        path: String,
        text: String,
    },
    Message {
        contact: String,
        text: String,
    },
    Reward {
        money: i64,
        reputation: i64,
    },
    Connection {
        online: bool,
    },
    Session {
        number: u8,
    },
    Inventory {
        key: String,
    },
    Evidence {
        key: String,
    },
    Decision {
        key: String,
        value: String,
    },
}

impl Effect {
    fn resources(&self) -> Vec<Resource> {
        match self {
            Self::Search { effect } => vec![effect.resource()],
            Self::Repository { .. } => Vec::new(),
            Self::Flag { key } => vec![Resource::Flag { key: key.clone() }],
            Self::File { path, .. } => vec![Resource::file(path)],
            Self::Message { contact, .. } => vec![Resource::Contact {
                name: contact.clone(),
            }],
            Self::Reward { .. } => vec![Resource::Money, Resource::Reputation],
            Self::Connection { .. } => vec![Resource::Connection],
            Self::Session { .. } => vec![Resource::Session],
            Self::Inventory { key } => vec![Resource::Inventory { key: key.clone() }],
            Self::Evidence { key } => vec![Resource::Evidence { key: key.clone() }, Resource::Heat],
            Self::Decision { key, .. } => vec![Resource::Decision { key: key.clone() }],
        }
    }
    fn apply_scoped(&self, world: &mut WorldState, mission: &str) -> GameResult<()> {
        let before: Vec<_> = self
            .resources()
            .into_iter()
            .map(|r| {
                let v = r.read(world);
                (r, v)
            })
            .collect();
        let messages = world.messages.len();
        self.apply(world)?;
        for (resource, old) in before {
            mission_runtime::record_change(world, mission, resource, old)?;
        }
        let added: Vec<_> = world.messages[messages..]
            .iter()
            .map(|m| {
                let resource = Resource::Message { id: m.id.clone() };
                let value = resource.read(world);
                (resource, value)
            })
            .collect();
        for (resource, value) in added {
            world
                .mission_runtime
                .record(mission, resource, None, value)?;
        }
        Ok(())
    }
    fn apply(&self, world: &mut WorldState) -> GameResult<()> {
        match self {
            Self::Search { effect } => effect.apply(world)?,
            Self::Repository {
                id,
                available,
                release,
                trusted,
            } => {
                world.packages.repositories.insert(
                    id.clone(),
                    crate::packages::model::RepositoryState {
                        available: *available,
                        trusted: *trusted,
                        release: *release,
                    },
                );
            }
            Self::Flag { key } => {
                world.flags.insert(key.clone());
            }
            Self::File { path, text } => {
                world.vfs.seed(path, "file", text, "kali");
            }
            Self::Message { contact, text } => world.notify(contact, text),
            Self::Reward { money, reputation } => {
                world.money += money;
                world.reputation += reputation;
            }
            Self::Connection { online } => world.network.connected = *online,
            Self::Session { number } => world.session = *number,
            Self::Inventory { key } => {
                world.inventory.insert(key.clone());
            }
            Self::Evidence { key } => {
                if !world.evidence.contains(key) {
                    world.evidence.push(key.clone());
                    world.heat += 1;
                }
            }
            Self::Decision { key, value } => {
                world.decisions.insert(key.clone(), value.clone());
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Choice {
    pub id: String,
    pub label: String,
    pub irreversible: bool,
    pub effects: Vec<Effect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stage {
    pub objective: String,
    pub hint: String,
    pub conditions: Vec<Condition>,
    #[serde(default)]
    pub choices: Vec<Choice>,
    #[serde(default)]
    pub effects: Vec<Effect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionDefinition {
    pub id: String,
    pub title: String,
    pub contact: String,
    pub session: u8,
    pub description: String,
    pub requirements: Vec<Condition>,
    #[serde(default)]
    pub start_triggers: Vec<Condition>,
    pub on_start: Vec<Effect>,
    pub stages: Vec<Stage>,
    pub outcomes: Vec<Effect>,
}

pub struct MissionEngine {
    pub definitions: Vec<MissionDefinition>,
}

#[derive(Debug, Clone)]
pub struct CheckpointEvent {
    pub kind: &'static str,
    pub mission: String,
    pub label: String,
    pub state: WorldState,
}

impl MissionEngine {
    /// Requirements are durable prerequisites, not resources owned by the next mission.
    pub fn scope(&self, id: &str) -> GameResult<Vec<Resource>> {
        let mission = self
            .definitions
            .iter()
            .find(|m| m.id == id)
            .ok_or_else(|| domain("mission not found"))?;
        let mut scope = BTreeSet::new();
        for stage in &mission.stages {
            for condition in &stage.conditions {
                match condition {
                    Condition::ArchiveEvent { path, .. } => {
                        scope.insert(Resource::file(path));
                    }
                    Condition::Flag { key } => {
                        scope.insert(Resource::Flag { key: key.clone() });
                    }
                    Condition::FileContains { path, .. } => {
                        scope.insert(Resource::file(path));
                    }
                    Condition::Decision { key, .. } => {
                        scope.insert(Resource::Decision { key: key.clone() });
                    }
                    Condition::Inventory { key } => {
                        scope.insert(Resource::Inventory { key: key.clone() });
                    }
                    Condition::Technique { key } => {
                        scope.insert(Resource::Technique { key: key.clone() });
                    }
                    Condition::HostService { host, port, .. } => {
                        scope.insert(Resource::Service {
                            host: host.clone(),
                            port: *port,
                        });
                    }
                    Condition::Money { .. }
                    | Condition::Reputation { .. }
                    | Condition::PackageInstalled { .. }
                    | Condition::PackageEvent { .. } => {}
                }
            }
        }
        let effects = mission
            .on_start
            .iter()
            .chain(mission.stages.iter().flat_map(|s| {
                s.effects
                    .iter()
                    .chain(s.choices.iter().flat_map(|c| c.effects.iter()))
            }));
        for effect in effects {
            scope.extend(effect.resources().into_iter().filter(|r| {
                !matches!(
                    r,
                    Resource::Money
                        | Resource::Reputation
                        | Resource::Heat
                        | Resource::Connection
                        | Resource::Contact { .. }
                )
            }));
        }
        // Resources produced by existing lab mechanics, beyond JSON conditions.
        let (flags, files, techniques): (&[&str], &[&str], &[&str]) = match id {
            "v1" => (
                &[],
                &[
                    "/home/kali/projects/v1.sim",
                    "/home/kali/Downloads/cyber-siege.manifest",
                ],
                &[],
            ),
            "v2" => (
                &["ROUTINE_ANALYZED"],
                &["/home/kali/projects/profile_01.dat"],
                &["runtime-patch", "save-editing"],
            ),
            "wifi" => (&["WIFI_SAMPLES", "WIFI_KEY"], &[], &["wireless-analysis"]),
            "pendrive" => (
                &[],
                &["/home/kali/Documents/fotos-recuperadas.txt"],
                &["file-recovery"],
            ),
            "vex-production" => (&["VEX_SECURE", "VEX_HARDENED"], &[], &[]),
            "vex-incident" => (&["VEX_CONTAINED"], &[], &[]),
            "girl" => (
                &["GIRL_RESEARCHED", "GIRL_ACCESS"],
                &[],
                &["recovery-correlation"],
            ),
            "signal-no-ar" => (
                &[],
                &[
                    crate::investigation::SIGNAL_PATH,
                    "/home/kali/Downloads/orion.capture.json",
                ],
                &[],
            ),
            _ => (&[], &[], &[]),
        };
        scope.extend(
            flags
                .iter()
                .map(|key| Resource::Flag { key: (*key).into() }),
        );
        scope.extend(files.iter().map(|path| Resource::file(path)));
        scope.extend(
            techniques
                .iter()
                .map(|key| Resource::Technique { key: (*key).into() }),
        );
        if id == "v1" {
            scope.extend([Resource::MemoryValue, Resource::MemoryCandidates]);
        }
        let remote: &[&str] = match id {
            "forum-job" => &["/etc/web.conf", "/var/log/web.log"],
            "vex-setup" => &["/srv/www/health.txt"],
            "vex-production" => &["/home/kali/web-backup.conf", "/etc/web.conf"],
            "vex-incident" => &["/etc/web.conf", "/var/log/incident-preserved.log"],
            _ => &[],
        };
        scope.extend(remote.iter().map(|path| Resource::File {
            host: Some("10.20.4.15".into()),
            path: (*path).into(),
        }));
        if matches!(id, "vex-production" | "vex-incident") {
            scope.insert(Resource::Patched {
                host: "10.20.4.15".into(),
            });
        }
        Ok(scope.into_iter().collect())
    }
    /// Capture only declared resource changes made by terminal/GUI actions.
    pub fn track_action(&self, before: &WorldState, after: &mut WorldState) -> GameResult<()> {
        let mut owners = std::collections::BTreeMap::<Resource, Vec<String>>::new();
        for (mission, attempt) in &before.mission_runtime.attempts {
            if !after.mission_runtime.attempts.contains_key(mission) {
                continue;
            }
            for resource in &attempt.scope {
                owners
                    .entry(resource.clone())
                    .or_default()
                    .push(mission.clone());
            }
            for resource in mission_runtime::inherited_files(before, &attempt.id)
                .into_iter()
                .chain(mission_runtime::inherited_files(after, &attempt.id))
            {
                let list = owners.entry(resource).or_default();
                if !list.contains(mission) {
                    list.push(mission.clone());
                }
            }
        }
        for (resource, missions) in owners {
            if let Resource::File { host: None, path } = &resource {
                if after.packages.ownership.contains_key(path)
                    || before.packages.ownership.contains_key(path)
                    || after
                        .vfs
                        .nodes
                        .get(path)
                        .is_some_and(|n| n.metadata.contains_key("packageProjection"))
                {
                    continue;
                }
            }
            let old = resource.read(before);
            let new = resource.read(after);
            if old == new
                || before.mission_runtime.entry(&resource) != after.mission_runtime.entry(&resource)
            {
                continue;
            }
            if missions.len() != 1 {
                return Err(domain("resource belongs to multiple active missions; finish or abandon one attempt before editing it"));
            }
            mission_runtime::record_change(after, &missions[0], resource, old)?;
        }
        Ok(())
    }
    /// Migrate legacy saves using only mission-owned resources from their old
    /// boundary. Personal data is never copied out of the old WorldState.
    pub fn normalize_loaded(
        &self,
        world: &mut WorldState,
        mut baseline: impl FnMut(&str) -> GameResult<WorldState>,
    ) -> GameResult<()> {
        let active: Vec<_> = world
            .missions
            .iter()
            .filter(|(_, p)| p.status == "active")
            .map(|(id, _)| id.clone())
            .collect();
        for id in &active {
            if world.mission_runtime.attempts.contains_key(id) {
                continue;
            }
            if world.schema_version != 1 {
                return Err(domain(
                    "active mission is missing its attempt journal; save was not changed",
                ));
            }
            let original = baseline(id)?;
            let scope = self.scope(id)?;
            world.mission_runtime.begin(id, scope.clone());
            for resource in scope {
                let before = resource.read(&original);
                let after = resource.read(world);
                if before != after {
                    world.mission_runtime.record(id, resource, before, after)?;
                }
            }
            let definition = self
                .definitions
                .iter()
                .find(|m| m.id == *id)
                .ok_or_else(|| domain("legacy mission not found"))?;
            let progress = &world.missions[id];
            let effects: Vec<_> = definition
                .on_start
                .iter()
                .chain(
                    definition
                        .stages
                        .iter()
                        .take(progress.stage)
                        .flat_map(|s| &s.effects),
                )
                .collect();
            let mut messages = Vec::new();
            let mut money = 0;
            let mut reputation = 0;
            for effect in effects {
                if let Effect::Reward {
                    money: amount,
                    reputation: rep,
                } = effect
                {
                    money += amount;
                    reputation += rep;
                }
                if let Effect::Message { contact, text } = effect {
                    let text = text.replace("[NICKNAME]", &world.nickname);
                    messages.extend(
                        world
                            .messages
                            .iter()
                            .filter(|m| {
                                m.contact == *contact
                                    && m.text == text
                                    && !original.messages.iter().any(|old| old.id == m.id)
                            })
                            .map(|m| Resource::Message { id: m.id.clone() }),
                    );
                }
            }
            for resource in messages {
                let after = resource.read(world);
                world.mission_runtime.record(id, resource, None, after)?;
            }
            for (resource, amount) in [(Resource::Money, money), (Resource::Reputation, reputation)]
            {
                if amount != 0 {
                    let after = resource.read(world);
                    let value = after
                        .as_ref()
                        .and_then(serde_json::Value::as_i64)
                        .ok_or_else(|| domain("invalid legacy reward"))?;
                    world.mission_runtime.record(
                        id,
                        resource,
                        Some(serde_json::json!(value - amount)),
                        after,
                    )?;
                }
            }
        }
        for id in active {
            mission_runtime::discard(world, &id)?;
        }
        world.settings.remove("missionBaseline");
        world.schema_version = 2;
        Ok(())
    }
    pub fn load() -> GameResult<Self> {
        let mut definitions: Vec<MissionDefinition> = serde_json::from_str(include_str!(
            "../../content/missions/session_1/vertical-slice.json"
        ))?;
        definitions.extend(serde_json::from_str::<Vec<MissionDefinition>>(
            include_str!("../../content/missions/session_2/orion.json"),
        )?);
        let mut ids = BTreeSet::new();
        for mission in &definitions {
            if !ids.insert(&mission.id) || mission.stages.is_empty() {
                return Err(domain("invalid mission content"));
            }
            for stage in &mission.stages {
                if stage.conditions.is_empty() {
                    return Err(domain("stage needs a completion condition"));
                }
                let unique: BTreeSet<_> = stage.choices.iter().map(|c| &c.id).collect();
                if unique.len() != stage.choices.len() {
                    return Err(domain("duplicate choice id"));
                }
            }
        }
        Ok(Self { definitions })
    }
    pub fn available(&self, world: &WorldState, mission: &MissionDefinition) -> bool {
        !world.missions.values().any(|p| p.status == "active") && self.unlocked(world, mission)
    }
    pub fn unlocked(&self, world: &WorldState, mission: &MissionDefinition) -> bool {
        !world.missions.contains_key(&mission.id)
            && mission.requirements.iter().all(|c| c.matches(world))
    }
    pub fn start(&self, world: &mut WorldState, id: &str) -> GameResult<CheckpointEvent> {
        let mission = self
            .definitions
            .iter()
            .find(|m| m.id == id)
            .ok_or_else(|| domain("mission not found"))?;
        if world.missions.values().any(|p| p.status == "active") {
            return Err(domain("Já existe uma missão em andamento. Conclua ou abandone a tentativa atual antes de iniciar outra."));
        }
        if !self.available(world, mission) {
            return Err(domain("mission unavailable"));
        }
        let event = CheckpointEvent {
            kind: "mission_start",
            mission: id.into(),
            label: format!("Início · {}", mission.title),
            state: world.clone(),
        };
        world.mission_runtime.begin(id, self.scope(id)?);
        world.missions.insert(
            id.into(),
            MissionProgress {
                status: "active".into(),
                stage: 0,
                attempts: world.mission_runtime.attempts[id].number,
            },
        );
        for effect in &mission.on_start {
            effect.apply_scoped(world, id)?;
        }
        if id == "signal-no-ar" {
            let case: crate::investigation::SignalCase =
                serde_json::from_str(include_str!("../../content/cases/orion.json"))?;
            let resources = [
                Resource::file(crate::investigation::SIGNAL_PATH),
                Resource::Wifi {
                    bssid: case.network.bssid,
                },
            ];
            let before: Vec<_> = resources.iter().map(|r| r.read(world)).collect();
            crate::investigation::seed_orion(world)?;
            for (resource, old) in resources.into_iter().zip(before) {
                mission_runtime::record_change(world, id, resource, old)?;
            }
        }
        world.events.push(format!("mission_start:{id}"));
        Ok(event)
    }
    pub fn choose(
        &self,
        world: &mut WorldState,
        id: &str,
        choice_id: &str,
    ) -> GameResult<Option<CheckpointEvent>> {
        let progress = world
            .missions
            .get(id)
            .ok_or_else(|| domain("mission not active"))?;
        if progress.status != "active" {
            return Err(domain("mission not active"));
        }
        let stage = self
            .definitions
            .iter()
            .find(|m| m.id == id)
            .and_then(|m| m.stages.get(progress.stage))
            .ok_or_else(|| domain("stage not found"))?;
        let choice = stage
            .choices
            .iter()
            .find(|c| c.id == choice_id)
            .ok_or_else(|| domain("choice unavailable"))?;
        let event = choice.irreversible.then(|| CheckpointEvent {
            kind: "decision",
            mission: id.into(),
            label: format!("Antes de: {}", choice.label),
            state: world.clone(),
        });
        for effect in &choice.effects {
            effect.apply_scoped(world, id)?;
        }
        Ok(event)
    }
    pub fn abort(&self, world: &mut WorldState, id: &str) -> GameResult<()> {
        let progress = world
            .missions
            .get_mut(id)
            .ok_or_else(|| domain("mission not active"))?;
        if progress.status != "active" {
            return Err(domain("mission not active"));
        }
        mission_runtime::discard(world, id)
    }
    pub fn evaluate(&self, world: &mut WorldState) -> GameResult<Vec<CheckpointEvent>> {
        let mut events = Vec::new();
        for _ in 0..64 {
            let mut changed = false;
            for mission in &self.definitions {
                if self.available(world, mission)
                    && !world
                        .mission_runtime
                        .attempt_counts
                        .contains_key(&mission.id)
                    && !mission.start_triggers.is_empty()
                    && mission.start_triggers.iter().all(|c| c.matches(world))
                {
                    events.push(self.start(world, &mission.id)?);
                    changed = true;
                }
                let Some(progress) = world.missions.get(&mission.id) else {
                    continue;
                };
                if progress.status != "active" {
                    continue;
                }
                let Some(stage) = mission.stages.get(progress.stage) else {
                    return Err(domain("invalid mission progress"));
                };
                if !stage.conditions.iter().all(|c| c.matches(world)) {
                    continue;
                }
                for effect in &stage.effects {
                    effect.apply_scoped(world, &mission.id)?;
                }
                if let Some(progress) = world.missions.get_mut(&mission.id) {
                    progress.stage += 1;
                    if progress.stage == mission.stages.len() {
                        progress.status = "completed".into();
                        for effect in &mission.outcomes {
                            effect.apply(world)?;
                        }
                        mission_runtime::commit(world, &mission.id);
                        world.events.push(format!("mission_end:{}", mission.id));
                        events.push(CheckpointEvent {
                            kind: "mission_end",
                            mission: mission.id.clone(),
                            label: format!("Fim · {}", mission.title),
                            state: world.clone(),
                        });
                    }
                }
                changed = true;
            }
            if !changed {
                return Ok(events);
            }
        }
        Err(domain("mission evaluation limit exceeded"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn requirements_branching_and_rewards_are_authoritative() {
        let engine = MissionEngine::load().expect("content");
        let mut world = WorldState::new("neo", "lifeos").expect("world");
        assert!(engine.start(&mut world, "girl").is_err());
        engine.start(&mut world, "first-boot").expect("start");
        world
            .vfs
            .mkdir("/home/kali/Documents/notes.txt", "kali")
            .expect("directory");
        engine.evaluate(&mut world).expect("evaluate directory");
        assert!(!world.flags.contains("FIRST_BOOT_COMPLETE"));
        world
            .vfs
            .remove("/home/kali/Documents/notes.txt", "kali", false)
            .expect("remove directory");
        world
            .vfs
            .write("/home/kali/Documents/notes.txt", "pronto", "kali")
            .expect("write");
        engine.evaluate(&mut world).expect("evaluate");
        assert!(world.flags.contains("FIRST_BOOT_COMPLETE"));
        let reputation = world.reputation;
        engine.evaluate(&mut world).expect("repeat");
        assert_eq!(world.reputation, reputation);
        assert!(engine.start(&mut world, "first-boot").is_err());
        assert!(engine.choose(&mut world, "girl", "share").is_err());
    }
}
