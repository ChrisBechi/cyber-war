use super::documents::{tokens, SearchDocument};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::OnceLock,
};

/// Immutable postings are shared by every campaign. Saves store only narrative overrides.
pub struct SearchIndex {
    pub documents: BTreeMap<String, SearchDocument>,
    postings: BTreeMap<String, Vec<usize>>,
    ids: Vec<String>,
}
impl SearchIndex {
    pub fn new(documents: Vec<SearchDocument>) -> Self {
        let mut index = Self {
            documents: documents.into_iter().map(|d| (d.id.clone(), d)).collect(),
            postings: BTreeMap::new(),
            ids: Vec::new(),
        };
        for (ordinal, (id, doc)) in index.documents.iter().enumerate() {
            index.ids.push(id.clone());
            let normalized: BTreeSet<_> = tokens(doc).into_iter().collect();
            for token in normalized {
                if token.is_empty() {
                    continue;
                }
                index.postings.entry(token).or_default().push(ordinal);
            }
        }
        index
    }
    pub fn candidates(&self, query: &str) -> BTreeSet<String> {
        let mut postings = Vec::new();
        for token in query.split_whitespace() {
            let Some(hits) = self.postings.get(token) else {
                return BTreeSet::new();
            };
            postings.push(hits);
        }
        // Each posting is sorted by ordinal. Start with the rarest token and test membership
        // in the others; long document IDs are cloned only for final matches.
        postings.sort_by_key(|hits| hits.len());
        let Some(first) = postings.first() else {
            return self.ids.iter().cloned().collect();
        };
        first
            .iter()
            .filter(|id| {
                postings
                    .iter()
                    .skip(1)
                    .all(|hits| hits.binary_search(id).is_ok())
            })
            .map(|id| self.ids[*id].clone())
            .collect()
    }
}
pub fn base() -> &'static SearchIndex {
    static INDEX: OnceLock<SearchIndex> = OnceLock::new();
    INDEX.get_or_init(|| {
        let mut documents: Vec<SearchDocument> =
            serde_json::from_str(include_str!("../../../content/search/documents.json"))
                .expect("validated virtual search catalog");
        let web = crate::virtual_web::indexed_documents();
        let urls: BTreeSet<_> = web
            .iter()
            .filter_map(|d| super::virtual_url_key(&d.url).ok())
            .collect();
        documents.retain(|old| {
            super::virtual_url_key(&old.url)
                .ok()
                .is_none_or(|key| !urls.contains(&key))
        });
        documents.extend(web);
        SearchIndex::new(documents)
    })
}
