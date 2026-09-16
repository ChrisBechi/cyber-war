use crate::{
    error::GameResult,
    network::VirtualNetwork,
    vfs::{domain, VirtualFileSystem, HOME},
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    #[serde(default)]
    pub attachments: Vec<String>,
    pub id: String,
    pub contact: String,
    pub text: String,
    pub read: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VirtualProcess {
    pub pid: u32,
    pub name: String,
    pub user: String,
    pub running: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalSession {
    #[serde(skip)]
    pub shell: crate::shell::Runtime,
    #[serde(skip)]
    pub presentation: crate::system_info::Presentation,
    #[serde(skip)]
    pub shell_depth: usize,
    #[serde(skip)]
    pub last_status: i32,
    #[serde(skip)]
    pub exported: BTreeSet<String>,
    pub cwd: String,
    pub user: String,
    pub host: Option<String>,
    #[serde(skip)]
    pub nano: Option<crate::nano::NanoSession>,
    #[serde(skip)]
    pub env: BTreeMap<String, String>,
    #[serde(skip)]
    pub history: Vec<String>,
    #[serde(skip)]
    pub foreground: Option<String>,
    #[serde(skip)]
    pub archive_pending: Option<crate::archive::cli::Pending>,
    #[serde(skip)]
    pub package_pending: Option<crate::packages::model::Pending>,
    #[serde(skip)]
    pub package_job: Option<u32>,
    #[serde(skip)]
    pub stdin: Option<String>,
    #[serde(skip)]
    pub io: crate::shell_pipeline::IoState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionProgress {
    pub status: String,
    pub stage: usize,
    pub attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldState {
    pub schema_version: u32,
    pub nickname: String,
    pub hostname: String,
    pub playtime_seconds: u64,
    pub session: u8,
    pub money: i64,
    pub reputation: i64,
    pub flags: BTreeSet<String>,
    pub techniques: BTreeSet<String>,
    pub inventory: BTreeSet<String>,
    pub decisions: BTreeMap<String, String>,
    pub missions: BTreeMap<String, MissionProgress>,
    #[serde(default)]
    pub mission_runtime: crate::mission_runtime::MissionRuntime,
    pub vfs: VirtualFileSystem,
    #[serde(skip)]
    pub blobs: crate::binary::BlobCache,
    #[serde(default)]
    pub archive_events: Vec<crate::archive::ArchiveEvent>,
    #[serde(skip)]
    pub archive_jobs: Vec<crate::archive::jobs::Job>,
    #[serde(default)]
    pub packages: crate::packages::model::PackageState,
    #[serde(skip)]
    pub package_lock: Option<String>,
    pub network: VirtualNetwork,
    #[serde(default)]
    pub domains: crate::domains::DomainState,
    pub terminal: TerminalSession,
    #[serde(skip)]
    pub terminal_sessions: BTreeMap<String, TerminalSession>,
    pub messages: Vec<Message>,
    pub contacts: BTreeSet<String>,
    pub processes: Vec<VirtualProcess>,
    pub evidence: Vec<String>,
    pub heat: u32,
    pub memory_value: i64,
    pub memory_candidates: Vec<u32>,
    pub settings: BTreeMap<String, String>,
    pub events: Vec<String>,
}

impl WorldState {
    pub fn new(nickname: &str, hostname: &str) -> GameResult<Self> {
        for value in [nickname, hostname] {
            if value.is_empty()
                || value.len() > 24
                || !value
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || "_-".contains(c))
            {
                return Err(domain("nome deve ter 1–24 letras ASCII, números, _ ou -"));
            }
        }
        let mut world = Self {
            schema_version: 2,
            nickname: nickname.into(),
            hostname: hostname.into(),
            playtime_seconds: 0,
            session: 0,
            money: 0,
            reputation: 0,
            flags: BTreeSet::new(),
            techniques: BTreeSet::new(),
            inventory: BTreeSet::new(),
            decisions: BTreeMap::new(),
            missions: BTreeMap::new(),
            mission_runtime: crate::mission_runtime::MissionRuntime::default(),
            vfs: VirtualFileSystem::default(),
            blobs: crate::binary::BlobCache::new(),
            archive_events: Vec::new(),
            archive_jobs: Vec::new(),
            packages: crate::packages::model::PackageState::default(),
            package_lock: None,
            network: VirtualNetwork::initial()?,
            domains: crate::domains::DomainState::seeded(),
            terminal: TerminalSession {
                shell: Default::default(),
                presentation: Default::default(),
                shell_depth: 0,
                last_status: 0,
                exported: BTreeSet::new(),
                cwd: HOME.into(),
                user: "kali".into(),
                host: None,
                nano: None,
                env: BTreeMap::new(),
                history: Vec::new(),
                foreground: None,
                archive_pending: None,
                package_pending: None,
                package_job: None,
                stdin: None,
                io: Default::default(),
            },
            terminal_sessions: BTreeMap::new(),
            messages: vec![Message {
                attachments: Vec::new(),
                id: "welcome".into(),
                contact: "Mãe".into(),
                text: "Filho, vou trabalhar. Tem comida na geladeira ❤️".into(),
                read: false,
            }],
            contacts: BTreeSet::from(["Mãe".into(), "Gregory".into()]),
            processes: vec![VirtualProcess {
                pid: 1,
                name: "lifeos-session".into(),
                user: "kali".into(),
                running: true,
            }],
            evidence: Vec::new(),
            heat: 0,
            memory_value: 100,
            memory_candidates: vec![4096, 8192, 12288],
            settings: BTreeMap::from([
                ("fontSize".into(), "14".into()),
                ("wallpaper".into(), "waves".into()),
                ("fileSort".into(), "name".into()),
                ("fileSortDirection".into(), "asc".into()),
                ("fileFoldersFirst".into(), "true".into()),
                ("autostartApps".into(), "[]".into()),
                ("servicesEnabled".into(), "[]".into()),
            ]),
            events: Vec::new(),
        };
        crate::packages::state::initialize(&mut world)?;
        Ok(world)
    }

    pub fn fs(&self) -> GameResult<&VirtualFileSystem> {
        match &self.terminal.host {
            Some(host) => self
                .network
                .hosts
                .get(host)
                .map(|h| &h.files)
                .ok_or_else(|| domain("session host missing")),
            None => Ok(&self.vfs),
        }
    }
    pub fn fs_mut(&mut self) -> GameResult<&mut VirtualFileSystem> {
        match &self.terminal.host {
            Some(host) => self
                .network
                .hosts
                .get_mut(host)
                .map(|h| &mut h.files)
                .ok_or_else(|| domain("session host missing")),
            None => Ok(&mut self.vfs),
        }
    }
    pub fn notify(&mut self, contact: &str, text: &str) {
        self.contacts.insert(contact.into());
        self.messages.push(Message {
            attachments: Vec::new(),
            id: uuid::Uuid::new_v4().to_string(),
            contact: contact.into(),
            text: text.replace("[NICKNAME]", &self.nickname),
            read: false,
        });
    }
    pub fn validate(&self) -> GameResult<()> {
        if ![1, 2].contains(&self.schema_version) {
            return Err(domain("unsupported save version"));
        }
        if self.schema_version == 2
            && self
                .missions
                .values()
                .filter(|p| p.status == "active")
                .count()
                > 1
        {
            return Err(domain("only one mission may be active"));
        }
        self.vfs.directory(HOME, "root")?;
        for fs in std::iter::once(&self.vfs).chain(self.network.hosts.values().map(|h| &h.files)) {
            for node in fs.nodes.values() {
                if let Some(blob) = &node.blob {
                    blob.validate()?;
                    if node.kind != "file" || !node.content.is_empty() {
                        return Err(domain("invalid binary file node"));
                    }
                }
            }
        }
        self.fs()?
            .directory(&self.terminal.cwd, &self.terminal.user)?;
        if self.money < 0 || self.reputation < 0 {
            return Err(domain("invalid progression"));
        }
        self.domains.validate()?;
        crate::packages::state::validate(self)?;
        Ok(())
    }
}
