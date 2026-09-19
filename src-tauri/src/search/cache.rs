use super::{query::Intent, SearchResponse, VirtualSearchEngine};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::VecDeque;

#[derive(Debug, Default)]
pub struct SearchCache {
    entries: VecDeque<Entry>,
    hits: u64,
    misses: u64,
}
#[derive(Debug)]
struct Entry {
    key: String,
    ids: Vec<String>,
    total: usize,
    offset: usize,
    correction: Option<String>,
    intent: Intent,
}
pub fn key(engine: &VirtualSearchEngine<'_>, query: &str, mode: &str, offset: usize) -> String {
    let world = engine.world;
    #[derive(Serialize)]
    struct Version<'a> {
        flags: &'a std::collections::BTreeSet<String>,
        missions: &'a std::collections::BTreeMap<String, crate::world::MissionProgress>,
        documents: &'a std::collections::BTreeMap<String, Option<super::SearchDocument>>,
        ranking: &'a std::collections::BTreeMap<String, i32>,
        removed: &'a std::collections::BTreeSet<String>,
        event_times: &'a std::collections::BTreeMap<String, u64>,
        time: u64,
    }
    let version = Version {
        flags: &world.flags,
        missions: &world.missions,
        documents: &world.search.documents,
        ranking: &world.search.ranking,
        removed: &world.web.removed,
        event_times: &world.web.event_started_at,
        time: crate::virtual_web::now(world),
    };
    let fingerprint = Sha256::digest(serde_json::to_vec(&version).unwrap_or_default());
    format!(
        "{fingerprint:x}:{mode}:{offset}:{}",
        super::documents::normalize(query)
    )
}
impl SearchCache {
    pub fn get(
        &mut self,
        key: &str,
        engine: &VirtualSearchEngine<'_>,
        query: &str,
    ) -> Option<SearchResponse> {
        let Some(position) = self.entries.iter().position(|e| e.key == key) else {
            self.misses = self.misses.saturating_add(1);
            return None;
        };
        self.hits = self.hits.saturating_add(1);
        let entry = self.entries.remove(position)?;
        let documents: Vec<super::SearchDocument> = entry
            .ids
            .iter()
            .filter_map(|id| {
                engine
                    .world
                    .search
                    .documents
                    .get(id)
                    .and_then(Option::as_ref)
                    .or_else(|| super::index::base().documents.get(id))
            })
            .filter(|d| engine.visible(d))
            .map(|doc| engine.present(doc))
            .collect();
        let images = super::images::for_documents(engine, &documents);
        let response = SearchResponse {
            ads: if entry.offset == 0 && matches!(key.split(':').nth(1), Some("all" | "shopping")) {
                crate::virtual_web::discovery::ads(engine.world, query)
            } else {
                Vec::new()
            },
            query: query.into(),
            documents,
            total: entry.total,
            offset: entry.offset,
            images,
            correction: entry.correction.clone(),
            intent: entry.intent.clone(),
        };
        self.entries.push_back(entry);
        Some(response)
    }
    pub fn insert(&mut self, key: String, response: &SearchResponse) {
        self.entries.retain(|e| e.key != key);
        self.entries.push_back(Entry {
            key,
            ids: response.documents.iter().map(|d| d.id.clone()).collect(),
            total: response.total,
            offset: response.offset,
            correction: response.correction.clone(),
            intent: response.intent.clone(),
        });
        while self.entries.len() > 64 {
            self.entries.pop_front();
        }
    }
    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    #[cfg(debug_assertions)]
    pub fn diagnostics(&self) -> serde_json::Value {
        serde_json::json!({"entries":self.entries.len(),"limit":64,"hits":self.hits,"misses":self.misses})
    }
}
