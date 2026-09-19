//! Local, data-driven web. No socket, HTTP client, HTML interpreter or host file access.
pub mod community;
#[cfg(debug_assertions)]
pub mod desktop_qa;
#[cfg(debug_assertions)]
pub mod devtools;
pub mod discovery;
pub mod events;
pub mod materialize;
pub mod model;
mod popularity;
pub use events::sync as sync_events;
#[cfg(test)]
mod community_tests;
#[cfg(test)]
mod event_tests;
#[cfg(test)]
mod tests;
use crate::{
    error::GameResult,
    search::documents::{DocumentType, SearchDocument, Visibility},
    vfs::domain,
    world::WorldState,
};
use model::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::OnceLock,
};

pub const EPOCH: u64 = 1_789_257_600;
pub const PAGE_SIZE: usize = 18;
pub const HISTORY_LIMIT: usize = 200;

pub struct Repository {
    pub pack: Pack,
    pub documents: BTreeMap<String, usize>,
    routes: BTreeMap<String, usize>,
    brands: BTreeMap<String, usize>,
    domains: BTreeMap<String, usize>,
}
pub fn repository() -> &'static Repository {
    static REPOSITORY: OnceLock<Repository> = OnceLock::new();
    REPOSITORY.get_or_init(|| {
        let pack: Pack = serde_json::from_str(include_str!("../../../content/web/web-core.json"))
            .expect("validated virtual web pack");
        Repository {
            documents: pack
                .documents
                .iter()
                .enumerate()
                .map(|(i, d)| (d.id.clone(), i))
                .collect(),
            routes: pack
                .documents
                .iter()
                .enumerate()
                .map(|(i, d)| (format!("{}{}", d.brand_id, d.path), i))
                .collect(),
            brands: pack
                .brands
                .iter()
                .enumerate()
                .map(|(i, b)| (b.id.clone(), i))
                .collect(),
            domains: pack
                .domains
                .iter()
                .enumerate()
                .map(|(i, d)| (d.host.clone(), i))
                .collect(),
            pack,
        }
    })
}
impl Repository {
    pub fn document(&self, id: &str) -> Option<&Document> {
        self.documents.get(id).map(|i| &self.pack.documents[*i])
    }
    pub fn brand(&self, id: &str) -> &Brand {
        &self.pack.brands[self.brands[id]]
    }
    pub fn domain(&self, host: &str) -> Option<&DomainRegistration> {
        self.domains.get(host).map(|i| &self.pack.domains[*i])
    }
    pub fn url(&self, doc: &Document) -> String {
        format!(
            "https://www.{}{}",
            self.brand(&doc.brand_id).domain,
            doc.path
        )
    }
    pub fn card(&self, world: &WorldState, base: &Document) -> Card {
        let doc = events::project(world, base);
        Card {
            listing_status: community::listing_status(world, &doc),
            rating: {
                let ratings: Vec<_> = doc.comments.iter().filter_map(|c| c.rating).collect();
                (!ratings.is_empty()).then(|| {
                    ratings.iter().map(|r| u64::from(*r)).sum::<u64>() as f64 / ratings.len() as f64
                })
            },
            id: doc.id.clone(),
            url: self.url(&doc),
            title: doc.title.clone(),
            summary: doc.summary.clone(),
            category: doc.category.clone(),
            author: doc.author.clone(),
            visual: doc.visual.clone(),
            price_cents: match doc.detail {
                Detail::Product { price_cents, .. } | Detail::Listing { price_cents, .. } => {
                    Some(price_cents)
                }
                _ => None,
            },
            stock: match &doc.detail {
                Detail::Product { stock, .. } => Some(stock.clone()),
                _ => None,
            },
        }
    }
}
pub fn now(world: &WorldState) -> u64 {
    EPOCH.saturating_add(world.playtime_seconds)
}
pub fn validate_state(state: &WebState) -> GameResult<()> {
    community::validate(&state.community)?;
    if state
        .pack_versions
        .get("web-core")
        .is_some_and(|version| *version > repository().pack.version)
    {
        return Err(domain(
            "Este save exige uma versão mais recente do pacote web-core.",
        ));
    }
    if state.history.len() > HISTORY_LIMIT
        || state.cart.len() > 100
        || state.comments.values().any(|c| c.len() > 50)
        || state.comments.values().map(Vec::len).sum::<usize>() > 500
        || state.cart.values().any(|n| *n == 0 || *n > 9)
    {
        return Err(domain("Estado da internet virtual inválido."));
    }
    Ok(())
}
pub fn visible(world: &WorldState, doc: &Document) -> bool {
    events::published_at(world, doc).is_some_and(|date| date <= now(world))
        && match &doc.detail {
            Detail::SocialPost { profile_id, .. } => repository()
                .document(profile_id)
                .is_some_and(|profile| visible(world, profile)),
            _ => true,
        }
        && !events::removed(world, &doc.id)
        && doc.required_flags.iter().all(|f| world.flags.contains(f))
        && !matches!(world.search.documents.get(&doc.id), Some(None))
        && world
            .search
            .documents
            .get(&doc.id)
            .and_then(Option::as_ref)
            .is_none_or(|d| {
                d.visibility == Visibility::Public
                    && d.required_flags.iter().all(|f| world.flags.contains(f))
                    && d.required_missions.iter().all(|id| {
                        world
                            .missions
                            .get(id)
                            .is_some_and(|m| m.status == "active" || m.status == "completed")
                    })
            })
}
pub fn indexed_documents() -> Vec<SearchDocument> {
    let repo = repository();
    repo.pack
        .documents
        .iter()
        .map(|doc| {
            let mut keywords = doc.keywords.clone();
            for id in &doc.entities {
                if let Some(entity) = repo.pack.entities.iter().find(|e| &e.id == id) {
                    keywords.push(entity.name.clone());
                    keywords.extend(entity.aliases.clone());
                }
            }
            let mut content = doc.summary.clone();
            for block in &doc.blocks {
                match block {
                    Block::Paragraph { text }
                    | Block::Heading { text }
                    | Block::Code { text, .. }
                    | Block::Quote { text, .. } => {
                        content.push(' ');
                        content.push_str(text);
                    }
                    Block::List { items, .. } => {
                        content.push(' ');
                        content.push_str(&items.join(" "));
                    }
                    _ => {}
                }
            }
            SearchDocument {
                id: doc.id.clone(),
                url: repo.url(doc),
                title: doc.title.clone(),
                description: doc.summary.clone(),
                content,
                keywords,
                domain: repo.brand(&doc.brand_id).domain.clone(),
                kind: match repo.brand(&doc.brand_id).platform {
                    Platform::Forum => DocumentType::Forum,
                    Platform::Commerce | Platform::Classifieds => DocumentType::Market,
                    Platform::Social => DocumentType::Profile,
                    Platform::Corporate => DocumentType::Company,
                    Platform::Blog => DocumentType::Blog,
                    Platform::Editorial => DocumentType::News,
                    _ => DocumentType::Document,
                },
                published_at: None,
                popularity: popularity::score(doc),
                authority: match repo.brand(&doc.brand_id).platform {
                    Platform::Encyclopedia | Platform::Corporate => 90,
                    Platform::Editorial | Platform::Education => 80,
                    Platform::Recipe | Platform::Commerce => 65,
                    Platform::Blog | Platform::Video => 50,
                    Platform::Forum | Platform::Social | Platform::Classifieds => 35,
                    Platform::Archive => 60,
                },
                mission_tags: Vec::new(),
                visibility: Visibility::Public,
                required_flags: doc.required_flags.clone(),
                required_missions: Vec::new(),
                suggestions: vec![doc.title.clone()],
                image_id: (doc.path != "/").then(|| format!("web-{}", doc.visual)),
                offer: None,
            }
        })
        .collect()
}

fn decode(value: &str) -> GameResult<String> {
    let mut bytes = Vec::new();
    let raw = value.as_bytes();
    let mut i = 0;
    while i < raw.len() {
        match raw[i] {
            b'%' if i + 2 < raw.len() => {
                let hex =
                    std::str::from_utf8(&raw[i + 1..i + 3]).map_err(|_| domain("URL inválida."))?;
                bytes.push(u8::from_str_radix(hex, 16).map_err(|_| domain("URL inválida."))?);
                i += 3;
            }
            b'%' => return Err(domain("URL inválida.")),
            b'+' => {
                bytes.push(b' ');
                i += 1;
            }
            b => {
                bytes.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8(bytes).map_err(|_| domain("URL inválida."))
}

pub fn resolve(world: &WorldState, address: &str) -> GameResult<Option<Page>> {
    let key = crate::search::virtual_url_key(address)?;
    let end = key.find(['/', '?', '#']).unwrap_or(key.len());
    let repo = repository();
    let Some(registration) = repo.domain(&key[..end]) else {
        return Ok(None);
    };
    let brand = repo.brand(&registration.brand_id);
    if !world.network.connected && brand.platform != Platform::Encyclopedia {
        return Err(domain("Sem conexão com a internet virtual."));
    }
    let suffix = key[end..].split('#').next().unwrap_or("");
    let (path, query) = suffix.split_once('?').unwrap_or((suffix, ""));
    let path = if path.is_empty() { "/" } else { path };
    let mut parameters = BTreeMap::new();
    for part in query.split('&').filter(|p| !p.is_empty()) {
        let (key, value) = part.split_once('=').unwrap_or((part, ""));
        parameters.insert(decode(key)?, decode(value)?);
    }
    let query = parameters.get("q").cloned().unwrap_or_default();
    let category = parameters.get("category").cloned().unwrap_or_default();
    let offset = parameters
        .get("offset")
        .and_then(|n| n.parse::<usize>().ok())
        .unwrap_or(0)
        .min(1_000_000);
    let canonical_url = format!(
        "https://www.{}{}",
        brand.domain,
        if suffix.is_empty() { "/" } else { suffix }
    );
    let all: Vec<_> = repo
        .pack
        .documents
        .iter()
        .filter(|d| d.brand_id == brand.id && d.path != "/" && visible(world, d))
        .collect();
    let document = repo
        .routes
        .get(&format!("{}{path}", brand.id))
        .map(|i| &repo.pack.documents[*i])
        .filter(|d| visible(world, d));
    let is_listing = path == "/" || path == "/search";
    let dynamic = if !is_listing && document.is_none() {
        crate::search::VirtualSearchEngine::new(world)
            .page(address)
            .map(|d| Document {
                materializer: None,
                id: d.id.clone(),
                brand_id: brand.id.clone(),
                path: path.into(),
                title: d.title.clone(),
                summary: d.description.clone(),
                category: "Publicações".into(),
                author: brand.name.clone(),
                published_at: EPOCH - 86400,
                updated_at: None,
                entities: Vec::new(),
                keywords: d.keywords.clone(),
                blocks: d
                    .content
                    .split("\n\n")
                    .map(|text| Block::Paragraph { text: text.into() })
                    .collect(),
                detail: Detail::Article { reading_minutes: 3 },
                comments: Vec::new(),
                links: Vec::new(),
                visual: "office".into(),
                required_flags: d.required_flags.clone(),
            })
    } else {
        None
    };
    let document = document.or(dynamic.as_ref());
    let status = if !is_listing && document.is_none() {
        404
    } else {
        200
    };
    let normalized = crate::search::documents::normalize(&query);
    let mut listing: Vec<_> = all
        .iter()
        .filter(|d| {
            if let Detail::Listing {
                location,
                condition,
                ..
            } = &d.detail
            {
                return parameters
                    .get("location")
                    .is_none_or(|v| v.is_empty() || v == location)
                    && parameters
                        .get("condition")
                        .is_none_or(|v| v.is_empty() || v == condition)
                    && parameters.get("status").is_none_or(|v| {
                        v.is_empty() || community::listing_status(world, d).as_ref() == Some(v)
                    });
            }
            true
        })
        .filter(|d| category.is_empty() || d.category == category)
        .filter(|d| {
            if normalized.is_empty() {
                return true;
            }
            let text = crate::search::documents::normalize(&format!(
                "{} {} {}",
                d.title,
                d.summary,
                d.keywords.join(" ")
            ));
            normalized
                .split_whitespace()
                .all(|word| text.contains(word))
        })
        .collect();
    listing.sort_by_key(|d| {
        (
            std::cmp::Reverse(d.published_at),
            if matches!(d.detail, Detail::Profile { .. } | Detail::Lesson { .. }) {
                2
            } else if d.path == "/reviews/nexphone-x2" || d.path == "/produtos/nexphone-x2" {
                0
            } else {
                1
            },
        )
    });
    let mut related: Vec<_> = document
        .map(|doc| all.iter().copied().filter(|d| d.id != doc.id).collect())
        .unwrap_or_default();
    related.sort_by_key(|d| {
        (
            std::cmp::Reverse(
                document
                    .map(|doc| {
                        d.entities
                            .iter()
                            .filter(|e| doc.entities.contains(e))
                            .count()
                    })
                    .unwrap_or(0),
            ),
            std::cmp::Reverse(popularity::score(d)),
            d.id.clone(),
        )
    });
    let offer_history = document
        .map(|d| events::offer_history(world, d))
        .unwrap_or_default();
    let mut document = document.map(|d| events::project(world, d));
    if let Some(doc) = &mut document {
        doc.blocks = materialize::blocks(doc);
        if let Some(Some(override_doc)) = world.search.documents.get(&doc.id) {
            doc.title = override_doc.title.clone();
            doc.summary = override_doc.description.clone();
            doc.blocks = vec![Block::Paragraph {
                text: override_doc.content.clone(),
            }];
        }
        if let Some(comments) = world.web.comments.get(&doc.id) {
            doc.comments.extend(comments.clone());
        }
        doc.comments
            .retain(|c| c.published_at >= doc.published_at && c.published_at <= now(world));
    }
    let mut page = Page {
        archive_url: repo
            .routes
            .get(&format!("{}{path}", brand.id))
            .map(|i| &repo.pack.documents[*i])
            .and_then(|d| discovery::archive_url(world, d)),
        filters: parameters
            .iter()
            .filter(|(k, _)| ["location", "condition", "status"].contains(&k.as_str()))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
        ads: discovery::placements(
            world,
            &format!(
                "{} {}",
                query,
                document
                    .as_ref()
                    .map(|d| d.title.as_str())
                    .unwrap_or(&brand.name)
            ),
            &format!("{:?}", brand.platform).to_ascii_uppercase(),
        ),
        capture: None,
        community: community::page(world, brand, document.as_ref()),
        brand: brand.clone(),
        canonical_url,
        status,
        liked: document
            .as_ref()
            .is_some_and(|d| world.web.likes.contains(&d.id)),
        completed: document
            .as_ref()
            .is_some_and(|d| world.web.completed_lessons.contains(&d.id)),
        cards: if is_listing {
            listing
                .iter()
                .skip(offset)
                .take(PAGE_SIZE)
                .map(|d| repo.card(world, d))
                .collect()
        } else {
            Vec::new()
        },
        related: related
            .iter()
            .take(4)
            .map(|d| repo.card(world, d))
            .collect(),
        document,
        offer_history,
        categories: all
            .iter()
            .map(|d| d.category.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
        total: listing.len(),
        offset,
        query,
        category,
        cart_count: world.web.cart.values().sum(),
        history: Vec::new(),
        cart: world
            .web
            .cart
            .iter()
            .map(|(id, quantity)| {
                let card = repo
                    .document(id)
                    .filter(|d| visible(world, d))
                    .map(|d| repo.card(world, d))
                    .unwrap_or_else(|| Card {
                        listing_status: None,
                        rating: None,
                        id: id.clone(),
                        url: String::new(),
                        title: "Produto indisponível".into(),
                        summary: String::new(),
                        category: String::new(),
                        author: String::new(),
                        visual: "book".into(),
                        price_cents: None,
                        stock: None,
                    });
                let available = card.stock.as_deref().is_some_and(events::purchasable);
                CartItem {
                    card,
                    quantity: *quantity,
                    available,
                }
            })
            .collect(),
    };
    discovery::archive(world, &mut page);
    Ok(Some(page))
}

pub fn remember(world: &mut WorldState, url: &str, title: &str, favicon: &str) {
    world.web.history.retain(|entry| entry.url != url);
    world.web.history.insert(
        0,
        HistoryEntry {
            url: url.into(),
            title: title.into(),
            favicon: favicon.into(),
            timestamp: now(world),
        },
    );
    world.web.history.truncate(HISTORY_LIMIT);
}

pub fn interact(world: &mut WorldState, id: &str, action: &str, text: &str) -> GameResult<()> {
    if !world.network.connected {
        return Err(domain("Sem conexão com a internet virtual."));
    }
    if action == "adClick" {
        discovery::click(world, id)?;
        return Ok(());
    }
    // A withdrawn or newly gated item must still be removable from a saved cart.
    if action == "removeCart" && world.web.cart.remove(id).is_some() {
        world.web.revision = world.web.revision.saturating_add(1);
        return Ok(());
    }
    let doc = repository()
        .document(id)
        .filter(|d| visible(world, d))
        .ok_or_else(|| domain("Publicação indisponível."))?;
    let doc = events::project(world, doc);
    if community::interact(world, &doc, action, text)? {
        world.web.revision = world.web.revision.saturating_add(1);
        return Ok(());
    }
    match action {
        "like"
            if matches!(
                doc.detail,
                Detail::Thread { .. }
                    | Detail::Video { .. }
                    | Detail::Article { .. }
                    | Detail::SocialPost { .. }
            ) =>
        {
            if !world.web.likes.remove(id) {
                world.web.likes.insert(id.into());
            }
        }
        "complete" if matches!(doc.detail, Detail::Lesson { .. }) => {
            world.web.completed_lessons.insert(id.into());
        }
        "cart" => {
            if let Detail::Product { stock, .. } = &doc.detail {
                if !events::purchasable(stock) {
                    return Err(domain("Produto indisponível."));
                }
                if world.web.cart.len() >= 100 && !world.web.cart.contains_key(id) {
                    return Err(domain("O carrinho está cheio."));
                }
                let count = world.web.cart.entry(id.into()).or_default();
                *count = count.saturating_add(1).min(9);
            } else {
                return Err(domain("Este item não é um produto."));
            }
        }
        "removeCart" => {
            world.web.cart.remove(id);
        }
        "comment"
            if !matches!(
                doc.detail,
                Detail::Thread { locked: true, .. }
                    | Detail::Profile { .. }
                    | Detail::SocialProfile { .. }
                    | Detail::Reference { .. }
            ) =>
        {
            let text = text.trim();
            if text.is_empty()
                || text.chars().count() > 1000
                || text.chars().any(|c| c.is_control() && c != '\n')
            {
                return Err(domain("Escreva um comentário com até 1.000 caracteres."));
            }
            if world.web.comments.values().map(Vec::len).sum::<usize>() >= 500 {
                return Err(domain("Limite de comentários desta campanha atingido."));
            }
            let comment = Comment {
                id: uuid::Uuid::new_v4().to_string(),
                author: world.nickname.clone(),
                text: text.into(),
                published_at: now(world),
                rating: None,
            };
            let comments = world.web.comments.entry(id.into()).or_default();
            if comments.len() >= 50 {
                return Err(domain("Limite de comentários nesta publicação atingido."));
            }
            comments.push(comment);
        }
        _ => return Err(domain("Ação indisponível nesta página.")),
    }
    world.web.revision = world.web.revision.saturating_add(1);
    Ok(())
}
