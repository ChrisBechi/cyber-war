use super::{
    documents::{SearchDocument, Visibility},
    index, validate_document,
};
use crate::{error::GameResult, vfs::domain, world::WorldState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchSuggestion {
    pub text: String,
    #[serde(default)]
    pub required_flags: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum SearchEffect {
    #[serde(rename = "ADD_SEARCH_DOCUMENT")]
    AddDocument { document: Box<SearchDocument> },
    #[serde(rename = "REMOVE_SEARCH_DOCUMENT")]
    RemoveDocument { id: String },
    #[serde(rename = "CHANGE_SEARCH_VISIBILITY")]
    ChangeVisibility { id: String, visibility: Visibility },
    #[serde(rename = "ADD_SEARCH_SUGGESTION")]
    AddSuggestion {
        id: String,
        suggestion: SearchSuggestion,
    },
    #[serde(rename = "CHANGE_SEARCH_RANKING")]
    ChangeRanking { id: String, boost: i32 },
}
impl SearchEffect {
    pub fn resource(&self) -> crate::mission_runtime::Resource {
        use crate::mission_runtime::Resource;
        match self {
            Self::AddSuggestion { id, .. } => Resource::SearchSuggestion { id: id.clone() },
            Self::ChangeRanking { id, .. } => Resource::SearchRanking { id: id.clone() },
            Self::AddDocument { document } => Resource::SearchDocument {
                id: document.id.clone(),
            },
            Self::RemoveDocument { id } | Self::ChangeVisibility { id, .. } => {
                Resource::SearchDocument { id: id.clone() }
            }
        }
    }
    pub fn apply(&self, world: &mut WorldState) -> GameResult<()> {
        match self {
            Self::AddDocument { document } => {
                validate_document(document)?;
                world
                    .search
                    .documents
                    .insert(document.id.clone(), Some(document.as_ref().clone()));
            }
            Self::RemoveDocument { id } => {
                world.search.documents.insert(id.clone(), None);
            }
            Self::ChangeVisibility { id, visibility } => {
                let mut doc = world
                    .search
                    .documents
                    .get(id)
                    .cloned()
                    .unwrap_or_else(|| index::base().documents.get(id).cloned())
                    .ok_or_else(|| domain("Documento não indexado."))?;
                doc.visibility = visibility.clone();
                world.search.documents.insert(id.clone(), Some(doc));
            }
            Self::AddSuggestion { id, suggestion } => {
                if suggestion.text.trim().is_empty() || suggestion.text.len() > 200 {
                    return Err(domain("Sugestão inválida."));
                }
                world
                    .search
                    .suggestions
                    .insert(id.clone(), suggestion.clone());
            }
            Self::ChangeRanking { id, boost } => {
                world
                    .search
                    .ranking
                    .insert(id.clone(), (*boost).clamp(-100_000, 100_000));
            }
        }
        Ok(())
    }
}
