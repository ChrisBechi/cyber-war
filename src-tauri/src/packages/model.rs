use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Relation {
    pub name: String,
    #[serde(default)]
    pub op: String,
    #[serde(default)]
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Payload {
    pub path: String,
    pub kind: String,
    pub content: String,
    pub mode: u16,
    pub logical_size: u64,
    #[serde(default)]
    pub conffile: bool,
    #[serde(default)]
    pub binding: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum Action {
    EnsureDirectory { path: String },
    DefaultFile { path: String, content: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Definition {
    pub name: String,
    pub version: String,
    pub architecture: String,
    pub description: String,
    pub section: String,
    pub maintainer: String,
    pub homepage: String,
    pub priority: String,
    pub essential: bool,
    pub download_size: u64,
    pub keywords: Vec<String>,
    pub depends: Vec<Vec<Relation>>,
    pub recommends: Vec<Vec<Relation>>,
    pub suggests: Vec<Vec<Relation>>,
    pub conflicts: Vec<Relation>,
    pub provides: Vec<Relation>,
    pub replaces: Vec<Relation>,
    pub files: Vec<Payload>,
    pub actions: Vec<Action>,
}
impl Definition {
    pub fn key(&self) -> String {
        format!(
            "{}_{}_{}",
            self.name,
            self.version.replace(':', "%3A"),
            self.architecture
        )
    }
    pub fn installed_size(&self) -> u64 {
        self.files.iter().map(|f| f.logical_size).sum()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Status {
    Unpacked,
    HalfConfigured,
    Installed,
    ConfigFiles,
}
impl Status {
    pub fn label(self) -> &'static str {
        match self {
            Self::Unpacked => "unpacked",
            Self::HalfConfigured => "half-configured",
            Self::Installed => "installed",
            Self::ConfigFiles => "config-files",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Installed {
    pub definition: Definition,
    pub status: Status,
    pub automatic: bool,
    pub held: bool,
    pub problems: Vec<String>,
    pub origin: String,
    pub installed_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ownership {
    pub owners: BTreeSet<String>,
    pub kind: String,
    pub shipped_hash: String,
    pub conffile: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexEntry {
    pub package: Definition,
    pub repository: String,
    pub suite: String,
    pub component: String,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
// Older saves may omit repository runtime fields. Apply the same defaults as
// an unmodified repository, without replacing any explicitly saved value.
#[serde(default, rename_all = "camelCase")]
pub struct RepositoryState {
    pub available: bool,
    pub trusted: bool,
    pub release: u32,
}
impl Default for RepositoryState {
    fn default() -> Self {
        Self {
            available: true,
            trusted: true,
            release: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub sequence: u64,
    pub transaction: String,
    pub kind: String,
    pub package: Option<String>,
    pub version: Option<String>,
    pub repository: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub id: String,
    pub operation: String,
    pub phase: String,
    #[serde(default)]
    pub phases: Vec<String>,
    pub packages: Vec<String>,
    pub at: u64,
    pub exit_code: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PackageState {
    pub initialized: bool,
    #[serde(default)]
    pub managed: BTreeSet<String>,
    pub revision: u64,
    pub installed: BTreeMap<String, Installed>,
    pub ownership: BTreeMap<String, Ownership>,
    pub indexes: BTreeMap<String, Vec<IndexEntry>>,
    pub repositories: BTreeMap<String, RepositoryState>,
    pub sources: BTreeSet<String>,
    pub events: Vec<Event>,
    pub sequence: u64,
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Clone)]
pub struct Plan {
    pub id: String,
    pub fingerprint: String,
    pub operation: String,
    pub requested: BTreeSet<String>,
    pub install: Vec<IndexEntry>,
    pub remove: Vec<String>,
    pub purge: bool,
    pub dpkg: bool,
    pub download_size: u64,
    pub peak_size: u64,
}

#[derive(Debug, Clone)]
pub struct Pending {
    pub plan: Plan,
    pub actor: String,
}
