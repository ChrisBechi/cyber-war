//! Local discovery service. No HTTP client, OS filesystem, DNS, OAuth or microphone access.
pub mod accounts;
mod cache;
pub mod documents;
pub mod images;
pub mod index;
pub mod mission_modifiers;
mod query;
mod ranking;
mod suggestions;
#[cfg(debug_assertions)]
pub fn explain(
    world: &crate::world::WorldState,
    doc: &documents::SearchDocument,
    query: &str,
) -> serde_json::Value {
    let terms = query::terms(query);
    let intent = query::intent(query);
    let parts = ranking::breakdown(doc, &terms, world);
    let base = parts.total();
    let intent_score = query::boost(doc, &intent);
    serde_json::json!({"components":parts,"intent":intent_score,"total":base+intent_score,"terms":terms,"intentType":intent})
}
#[cfg(test)]
mod quality_tests;
#[cfg(test)]
mod retrieval_tests;
#[cfg(test)]
mod scale_tests;
#[cfg(test)]
mod tests;

use crate::{error::GameResult, vfs::domain, world::WorldState};
use documents::{normalize, DocumentType, SearchDocument, Visibility};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SearchState {
    #[serde(skip)]
    pub cache: std::sync::Arc<std::sync::Mutex<cache::SearchCache>>,
    pub documents: BTreeMap<String, Option<SearchDocument>>,
    pub ranking: BTreeMap<String, i32>,
    pub suggestions: BTreeMap<String, mission_modifiers::SearchSuggestion>,
    pub images: BTreeMap<String, images::ImageRecord>,
    pub accounts: BTreeMap<String, accounts::VirtualGoggleAccount>,
    pub session: Option<String>,
    pub history_enabled: bool,
    pub history: Vec<String>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    pub ads: Vec<crate::virtual_web::discovery::Ad>,
    pub query: String,
    pub documents: Vec<SearchDocument>,
    pub total: usize,
    pub offset: usize,
    pub images: Vec<images::ImageRecord>,
    pub correction: Option<String>,
    pub intent: query::Intent,
}
pub struct VirtualSearchEngine<'a> {
    pub world: &'a WorldState,
}
impl<'a> VirtualSearchEngine<'a> {
    pub fn new(world: &'a WorldState) -> Self {
        Self { world }
    }
    pub fn documents(&self) -> impl Iterator<Item = &SearchDocument> {
        index::base()
            .documents
            .values()
            .filter(|d| !self.world.search.documents.contains_key(&d.id))
            .chain(
                self.world
                    .search
                    .documents
                    .values()
                    .filter_map(Option::as_ref),
            )
    }
    pub fn visible(&self, doc: &SearchDocument) -> bool {
        doc.visibility == Visibility::Public
            && crate::virtual_web::repository()
                .document(&doc.id)
                .is_none_or(|d| crate::virtual_web::visible(self.world, d))
            && doc
                .required_flags
                .iter()
                .all(|f| self.world.flags.contains(f))
            && doc.required_missions.iter().all(|id| {
                self.world
                    .missions
                    .get(id)
                    .is_some_and(|m| m.status == "active" || m.status == "completed")
            })
    }
    pub fn present(&self, doc: &SearchDocument) -> SearchDocument {
        let mut result = doc.clone();
        result.offer = crate::virtual_web::repository()
            .document(&doc.id)
            .and_then(|base| crate::virtual_web::events::offer_history(self.world, base).pop());
        result
    }
    pub fn search(
        &self,
        query: &str,
        mode: &str,
        source: Option<&str>,
        offset: usize,
    ) -> GameResult<SearchResponse> {
        if !self.world.network.connected {
            return Err(domain("Sem conexão com a internet virtual."));
        }
        if query.chars().count() > 200
            || source.is_some_and(|s| s.len() > 512)
            || !["all", "images", "videos", "news", "shopping"].contains(&mode)
        {
            return Err(domain("Pesquisa inválida."));
        }
        let started = std::time::Instant::now();
        let cache_key = source
            .is_none()
            .then(|| cache::key(self, query, mode, offset));
        if let Some(key) = &cache_key {
            if let Ok(mut cache) = self.world.search.cache.lock() {
                if let Some(response) = cache.get(key, self, query) {
                    return Ok(response);
                }
            }
        }
        let original = normalize(query);
        let mut normalized = query::terms(query);
        if normalized.is_empty() && !original.is_empty() {
            normalized = original.clone();
        }
        let correction = if !original.is_empty() && index::base().candidates(&normalized).is_empty()
        {
            query::correction(query)
        } else {
            None
        };
        if let Some(corrected) = &correction {
            normalized = query::terms(corrected);
        }
        let candidates = index::base().candidates(&normalized);
        let reverse = source.map(|s| images::reverse(self, s)).transpose()?;
        let selected: Vec<_> = if normalized.is_empty() || reverse.is_some() {
            self.documents().collect()
        } else {
            candidates
                .iter()
                .filter(|id| !self.world.search.documents.contains_key(*id))
                .filter_map(|id| index::base().documents.get(id))
                .chain(
                    self.world
                        .search
                        .documents
                        .values()
                        .filter_map(Option::as_ref),
                )
                .collect()
        };
        let mut documents: Vec<_> = selected
            .into_iter()
            .filter(|doc| {
                if !self.visible(doc) || (mode == "images" && doc.image_id.is_none()) {
                    return false;
                }
                if mode == "news" && doc.kind != DocumentType::News
                    || mode == "shopping" && doc.kind != DocumentType::Market
                    || mode == "videos"
                        && !crate::virtual_web::repository()
                            .document(&doc.id)
                            .is_some_and(|d| {
                                matches!(d.detail, crate::virtual_web::model::Detail::Video { .. })
                            })
                {
                    return false;
                }
                if let Some(ids) = &reverse {
                    return ids.contains(&doc.id);
                }
                if normalized.is_empty() {
                    return mode == "images";
                }
                if self.world.search.documents.contains_key(&doc.id) {
                    let tokens: BTreeSet<_> = documents::tokens(doc).into_iter().collect();
                    normalized
                        .split_whitespace()
                        .all(|term| tokens.contains(term))
                } else {
                    candidates.contains(&doc.id)
                }
            })
            .collect();
        let intent = query::intent(query);
        documents.sort_by_cached_key(|document| {
            (
                std::cmp::Reverse(
                    ranking::score(document, &normalized, self.world)
                        + query::boost(document, &intent),
                ),
                document.id.clone(),
            )
        });
        let total = documents.len();
        let documents: Vec<SearchDocument> = documents
            .into_iter()
            .skip(offset)
            .take(20)
            .map(|doc| self.present(doc))
            .collect();
        if started.elapsed() >= std::time::Duration::from_secs(5) {
            return Err(domain(
                "A pesquisa demorou mais que o esperado. Tente novamente.",
            ));
        }
        let images = images::for_documents(self, &documents);
        let response = SearchResponse {
            ads: if offset == 0 && ["all", "shopping"].contains(&mode) {
                crate::virtual_web::discovery::ads(self.world, query)
            } else {
                Vec::new()
            },
            query: query.into(),
            documents,
            total,
            offset,
            images,
            correction,
            intent,
        };
        if let Some(key) = cache_key {
            if let Ok(mut cache) = self.world.search.cache.lock() {
                cache.insert(key, &response);
            }
        }
        Ok(response)
    }
    pub fn suggest(&self, query: &str) -> GameResult<Vec<String>> {
        if query.chars().count() > 200 {
            return Err(domain("Pesquisa muito longa."));
        }
        if !self.world.network.connected {
            return Err(domain("Sem conexão com a internet virtual."));
        }
        Ok(suggestions::suggest(self, query))
    }
    pub fn page(&self, address: &str) -> Option<&SearchDocument> {
        let key = virtual_url_key(address).ok()?;
        self.documents()
            .find(|d| self.visible(d) && virtual_url_key(&d.url).ok().as_ref() == Some(&key))
    }
}
/// Parses only a virtual identifier. This never opens a URL.
pub fn virtual_url_key(address: &str) -> GameResult<String> {
    if address.len() > 512
        || address
            .chars()
            .any(|c| c.is_whitespace() || c.is_control() || c == '\\')
    {
        return Err(domain("Endereço virtual inválido."));
    }
    let address = address
        .strip_prefix("https://")
        .or_else(|| address.strip_prefix("http://"))
        .unwrap_or(address);
    let end = address.find(['/', '?', '#']).unwrap_or(address.len());
    let host = address[..end].to_ascii_lowercase();
    let host = host.strip_prefix("www.").unwrap_or(&host);
    if !host.contains('.')
        || !host
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b".-".contains(&b))
    {
        return Err(domain("Endereço virtual inválido."));
    }
    Ok(format!("{host}{}", address[end..].trim_end_matches('/')))
}
pub fn validate_document(doc: &SearchDocument) -> GameResult<()> {
    let key = virtual_url_key(&doc.url)?;
    if doc.id.is_empty()
        || doc.id.len() > 100
        || doc.title.is_empty()
        || doc.title.len() > 300
        || doc.content.len() > 100_000
        || key
            .split('/')
            .next()
            .unwrap_or("")
            .trim_start_matches("www.")
            != doc.domain.trim_start_matches("www.")
    {
        return Err(domain("Documento de pesquisa inválido."));
    }
    Ok(())
}
pub fn remember(world: &mut WorldState, query: &str) {
    if world.search.history_enabled && !query.trim().is_empty() {
        world.search.history.retain(|q| q != query);
        world.search.history.insert(0, query.into());
        world.search.history.truncate(100);
    }
}
