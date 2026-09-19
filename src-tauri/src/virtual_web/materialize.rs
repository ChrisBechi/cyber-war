//! Compact descriptors become controlled blocks only on opening. Cache is process-local, bounded,
//! and keyed by the entire descriptor so a pack update cannot reuse obsolete text.
use super::model::{Block, Detail, Document};
use std::{
    collections::VecDeque,
    sync::{Mutex, OnceLock},
};
type Entries = VecDeque<(String, Vec<Block>)>;
static CACHE: OnceLock<Mutex<Entries>> = OnceLock::new();
pub fn blocks(doc: &Document) -> Vec<Block> {
    if doc.materializer.as_deref() != Some("product") {
        return doc.blocks.clone();
    }
    let key = serde_json::to_string(&(
        super::repository().pack.version,
        super::repository().pack.seed,
        &doc.id,
        &doc.detail,
        &doc.summary,
    ))
    .unwrap_or_default();
    let cache = CACHE.get_or_init(|| Mutex::new(VecDeque::new()));
    if let Ok(mut entries) = cache.lock() {
        if let Some(i) = entries.iter().position(|(id, _)| id == &key) {
            let entry = entries.remove(i).unwrap();
            let blocks = entry.1.clone();
            entries.push_back(entry);
            return blocks;
        }
    }
    let mut blocks = vec![Block::Paragraph {
        text: doc.summary.clone(),
    }];
    if let Detail::Product { specifications, .. } = &doc.detail {
        blocks.push(Block::Heading {
            text: "Confira antes de escolher".into(),
        });
        blocks.push(Block::Table {
            headings: vec!["Característica".into(), "Esta versão".into()],
            rows: specifications.clone(),
        });
        blocks.push(Block::Paragraph{text:"Compare o conector das duas pontas com as entradas dos seus equipamentos. Para transferência de arquivos, verifique também a velocidade declarada; encaixar na porta não garante a mesma capacidade de outro cabo.".into()});
    }
    if let Ok(mut entries) = cache.lock() {
        entries.push_back((key, blocks.clone()));
        while entries.len() > 64 {
            entries.pop_front();
        }
    }
    blocks
}
#[cfg(any(test, debug_assertions))]
pub fn cache_len() -> usize {
    CACHE
        .get()
        .and_then(|c| c.lock().ok())
        .map_or(0, |c| c.len())
}
#[cfg(test)]
mod tests {
    #[test]
    fn deterministic_descriptors_stay_bounded_without_save_copies() {
        let docs: Vec<_> = super::super::repository()
            .pack
            .documents
            .iter()
            .filter(|d| d.materializer.is_some())
            .collect();
        assert!(docs.len() > 100);
        let first = super::blocks(docs[0]);
        for d in docs.iter().take(200) {
            assert!(!super::blocks(d).is_empty());
        }
        assert!(super::cache_len() <= 64);
        assert_eq!(
            serde_json::to_string(&first).unwrap(),
            serde_json::to_string(&super::blocks(docs[0])).unwrap()
        );
        assert!(docs[0].blocks.is_empty());
    }
}
