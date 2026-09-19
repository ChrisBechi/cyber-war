use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DocumentType {
    WebPage,
    News,
    Forum,
    Profile,
    Company,
    Image,
    Market,
    Blog,
    Document,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Visibility {
    Public,
    Hidden,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchDocument {
    pub id: String,
    pub url: String,
    pub title: String,
    pub description: String,
    pub content: String,
    pub keywords: Vec<String>,
    pub domain: String,
    #[serde(rename = "type")]
    pub kind: DocumentType,
    #[serde(default)]
    pub published_at: Option<String>,
    pub popularity: u32,
    pub authority: u32,
    #[serde(default)]
    pub mission_tags: Vec<String>,
    pub visibility: Visibility,
    #[serde(default)]
    pub required_flags: Vec<String>,
    #[serde(default)]
    pub required_missions: Vec<String>,
    #[serde(default)]
    pub suggestions: Vec<String>,
    #[serde(default)]
    pub image_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offer: Option<crate::virtual_web::model::OfferVersion>,
}

pub fn normalize(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| match c {
            'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
            'é' | 'ê' | 'è' | 'ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ó' | 'ô' | 'õ' | 'ò' | 'ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            c if c.is_alphanumeric() => c,
            _ => ' ',
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn tokens(doc: &SearchDocument) -> Vec<String> {
    super::query::terms(&format!(
        "{} {} {} {} {}",
        doc.title,
        doc.description,
        doc.content,
        doc.domain,
        doc.keywords.join(" ")
    ))
    .split_whitespace()
    .map(str::to_owned)
    .collect()
}
