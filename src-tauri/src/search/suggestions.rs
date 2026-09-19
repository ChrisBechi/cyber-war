use super::{documents::normalize, VirtualSearchEngine};
use std::collections::BTreeSet;

pub fn suggest(engine: &VirtualSearchEngine<'_>, query: &str) -> Vec<String> {
    let prefix = normalize(query);
    if prefix.is_empty() {
        return vec![];
    }
    let mut values = BTreeSet::new();
    for doc in engine.documents().filter(|d| engine.visible(d)) {
        for value in doc.suggestions.iter().chain(std::iter::once(&doc.title)) {
            if normalize(value).starts_with(&prefix) {
                values.insert(value.clone());
            }
        }
    }
    for suggestion in engine.world.search.suggestions.values() {
        if suggestion
            .required_flags
            .iter()
            .all(|f| engine.world.flags.contains(f))
            && normalize(&suggestion.text).starts_with(&prefix)
        {
            values.insert(suggestion.text.clone());
        }
    }
    let mut unique = std::collections::BTreeMap::new();
    for value in values {
        unique.entry(normalize(&value)).or_insert(value);
    }
    // Narrative suggestions keep their authored text when a title normalizes
    // to the same value (for example, punctuation and case differences).
    for suggestion in engine.world.search.suggestions.values() {
        let key = normalize(&suggestion.text);
        if key.starts_with(&prefix)
            && suggestion
                .required_flags
                .iter()
                .all(|f| engine.world.flags.contains(f))
        {
            unique.insert(key, suggestion.text.clone());
        }
    }
    let mut values: Vec<_> = unique.into_values().collect();
    values.sort_by_key(|value| {
        let explicit = engine.world.search.suggestions.values().any(|suggestion| {
            normalize(&suggestion.text) == normalize(value)
                && suggestion
                    .required_flags
                    .iter()
                    .all(|f| engine.world.flags.contains(f))
        });
        (!explicit, value.chars().count(), normalize(value))
    });
    values.into_iter().take(8).collect()
}
