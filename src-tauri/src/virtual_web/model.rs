use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Link {
    pub label: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Platform {
    Editorial,
    Forum,
    Commerce,
    Encyclopedia,
    Recipe,
    Education,
    Corporate,
    Video,
    Blog,
    Social,
    Classifieds,
    Archive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Brand {
    #[serde(default)]
    pub identity: Option<BrandIdentity>,
    #[serde(default)]
    pub depth: Option<String>,
    pub id: String,
    pub name: String,
    pub domain: String,
    pub platform: Platform,
    pub layout: String,
    pub tagline: String,
    pub accent: String,
    pub mark: String,
    pub navigation: Vec<Link>,
    pub voice: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BrandIdentity {
    pub typography: String,
    pub header: String,
    pub cards: String,
    pub density: String,
    pub surface: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DomainRegistration {
    pub host: String,
    pub brand_id: String,
    pub redirect: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub aliases: Vec<String>,
    pub description: String,
    pub relations: Vec<Relation>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Relation {
    pub kind: String,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum Block {
    Paragraph {
        text: String,
    },
    Heading {
        text: String,
    },
    Quote {
        text: String,
        attribution: String,
    },
    List {
        items: Vec<String>,
        ordered: bool,
    },
    Table {
        headings: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    Code {
        text: String,
        language: String,
    },
    Link {
        label: String,
        url: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Comment {
    pub id: String,
    pub author: String,
    pub text: String,
    pub published_at: u64,
    pub rating: Option<u8>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum Detail {
    SocialProfile {
        bio: String,
        location: String,
        role: String,
        employer: Link,
        following: Vec<String>,
    },
    SocialPost {
        profile_id: String,
        likes: u32,
    },
    Listing {
        price_cents: u64,
        condition: String,
        location: String,
        seller: Link,
        status: String,
        minimum_cents: u64,
    },
    Article {
        reading_minutes: u32,
    },
    Product {
        price_cents: u64,
        stock: String,
        seller: Link,
        specifications: Vec<Vec<String>>,
    },
    Recipe {
        minutes: u32,
        servings: u32,
        difficulty: String,
        ingredients: Vec<String>,
        steps: Vec<String>,
    },
    Course {
        minutes: u32,
        level: String,
        teacher: Link,
        lessons: Vec<Link>,
    },
    Lesson {
        course: Link,
        position: u32,
        next: Option<Link>,
    },
    Thread {
        community: String,
        votes: u32,
        locked: bool,
    },
    Video {
        seconds: u32,
        views: u32,
        channel: Link,
        chapters: Vec<String>,
    },
    Profile {
        bio: String,
        location: String,
    },
    Reference {
        facts: Vec<Vec<String>>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Document {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub materializer: Option<String>,
    pub id: String,
    pub brand_id: String,
    pub path: String,
    pub title: String,
    pub summary: String,
    pub category: String,
    pub author: String,
    pub published_at: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<u64>,
    pub entities: Vec<String>,
    pub keywords: Vec<String>,
    pub blocks: Vec<Block>,
    pub detail: Detail,
    pub comments: Vec<Comment>,
    pub links: Vec<Link>,
    pub visual: String,
    #[serde(default)]
    pub required_flags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Pack {
    #[serde(default)]
    pub ads: Vec<super::discovery::Campaign>,
    pub id: String,
    pub version: u32,
    pub seed: u64,
    pub brands: Vec<Brand>,
    pub domains: Vec<DomainRegistration>,
    pub entities: Vec<Entity>,
    pub documents: Vec<Document>,
    pub events: Vec<EventDefinition>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EventDefinition {
    pub id: String,
    pub required_flag: Option<String>,
    #[serde(default)]
    pub after_seconds: u64,
    pub publish: Vec<String>,
    pub remove: Vec<String>,
    #[serde(default)]
    pub offers: Vec<OfferEffect>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OfferEffect {
    pub document_id: String,
    pub price_cents: Option<u64>,
    pub stock: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfferVersion {
    pub timestamp: u64,
    pub price_cents: u64,
    pub stock: String,
}

/// Immutable packs are shared by all slots. Only personal and narrative deltas are serialized.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct WebState {
    pub ad_clicks: BTreeMap<String, u64>,
    pub community: super::community::CommunityState,
    pub pack_versions: BTreeMap<String, u32>,
    pub seed: u64,
    pub revision: u64,
    pub events: BTreeSet<String>,
    pub event_started_at: BTreeMap<String, u64>,
    pub removed: BTreeSet<String>,
    pub likes: BTreeSet<String>,
    pub completed_lessons: BTreeSet<String>,
    pub cart: BTreeMap<String, u32>,
    pub comments: BTreeMap<String, Vec<Comment>>,
    pub history: Vec<HistoryEntry>,
}
impl Default for WebState {
    fn default() -> Self {
        Self {
            ad_clicks: BTreeMap::new(),
            community: Default::default(),
            pack_versions: BTreeMap::from([("web-core".into(), 3)]),
            seed: 2317,
            revision: 0,
            events: BTreeSet::new(),
            event_started_at: BTreeMap::new(),
            removed: BTreeSet::new(),
            likes: BTreeSet::new(),
            completed_lessons: BTreeSet::new(),
            cart: BTreeMap::new(),
            comments: BTreeMap::new(),
            history: Vec::new(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HistoryEntry {
    pub url: String,
    pub title: String,
    pub favicon: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    pub listing_status: Option<String>,
    pub rating: Option<f64>,
    pub id: String,
    pub url: String,
    pub title: String,
    pub summary: String,
    pub category: String,
    pub author: String,
    pub visual: String,
    pub price_cents: Option<u64>,
    pub stock: Option<String>,
}
#[derive(Debug, Clone, Serialize)]
pub struct CartItem {
    pub card: Card,
    pub quantity: u32,
    pub available: bool,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    pub archive_url: Option<String>,
    pub filters: BTreeMap<String, String>,
    pub ads: Vec<super::discovery::Ad>,
    pub capture: Option<super::discovery::Capture>,
    pub community: Option<super::community::CommunityPage>,
    pub brand: Brand,
    pub canonical_url: String,
    pub status: u16,
    pub document: Option<Document>,
    pub cards: Vec<Card>,
    pub related: Vec<Card>,
    pub categories: Vec<String>,
    pub total: usize,
    pub offset: usize,
    pub query: String,
    pub category: String,
    pub liked: bool,
    pub completed: bool,
    pub cart_count: u32,
    pub cart: Vec<CartItem>,
    pub offer_history: Vec<OfferVersion>,
    pub history: Vec<HistoryEntry>,
}
