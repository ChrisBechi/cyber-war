use super::documents::normalize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Intent {
    Navigational,
    Informational,
    Shopping,
    Recipe,
    Video,
    News,
    Technical,
    Educational,
    General,
}
pub fn boost(doc: &super::SearchDocument, intent: &Intent) -> i64 {
    use crate::virtual_web::model::Detail;
    let detail = crate::virtual_web::repository()
        .document(&doc.id)
        .map(|d| &d.detail);
    match (intent, detail) {
        (Intent::Shopping, Some(Detail::Product { .. }))
        | (Intent::Recipe, Some(Detail::Recipe { .. }))
        | (Intent::Educational, Some(Detail::Course { .. }))
        | (Intent::Video, Some(Detail::Video { .. })) => 3500,
        (Intent::Informational, Some(Detail::Reference { .. })) => 2000,
        (Intent::News, _) if doc.kind == super::DocumentType::News => 2000,
        _ => 0,
    }
}
pub fn intent(query: &str) -> Intent {
    let text = normalize(query);
    if ["comprar", "preco", "loja", "cabo"]
        .iter()
        .any(|t| text.split_whitespace().any(|s| s == *t))
    {
        Intent::Shopping
    } else if text.contains("receita") || text.contains("bolo") {
        Intent::Recipe
    } else if text.contains("curso") || text.contains("aula") {
        Intent::Educational
    } else if text
        .replace("placas de video", "")
        .replace("placa de video", "")
        .replace("placa video", "")
        .split_whitespace()
        .any(|term| matches!(term, "video" | "videos"))
    {
        Intent::Video
    } else if text.contains("noticia") {
        Intent::News
    } else if text.starts_with("o que") {
        Intent::Informational
    } else if text.contains("debugging") || text.contains("manual") {
        Intent::Technical
    } else if crate::virtual_web::repository()
        .pack
        .brands
        .iter()
        .any(|b| normalize(&b.name) == text || normalize(&b.domain) == text)
    {
        Intent::Navigational
    } else {
        Intent::General
    }
}
pub fn terms(query: &str) -> String {
    normalize(query)
        .replace("wi fi", "wifi")
        .split_whitespace()
        .filter(|w| {
            ![
                "o", "a", "os", "as", "um", "uma", "de", "do", "da", "dos", "das", "em", "no",
                "na", "e", "que", "qual", "como", "para", "por", "comprar", "receita", "curso",
                "video", "noticias",
            ]
            .contains(w)
        })
        .map(|w| match w {
            "celulares" => "celular",
            "roteadores" => "roteador",
            "cabos" => "cabo",
            "filmes" => "filme",
            "placas" => "placa",
            "cachorros" => "cachorro",
            _ => w,
        })
        .collect::<Vec<_>>()
        .join(" ")
}
fn distance(a: &str, b: &str) -> usize {
    let a: Vec<_> = a.chars().collect();
    let b: Vec<_> = b.chars().collect();
    let mut row: Vec<_> = (0..=b.len()).collect();
    for (i, x) in a.iter().enumerate() {
        let mut previous = row[0];
        row[0] = i + 1;
        for (j, y) in b.iter().enumerate() {
            let old = row[j + 1];
            row[j + 1] = (row[j] + 1)
                .min(old + 1)
                .min(previous + usize::from(x != y));
            previous = old;
        }
    }
    row[b.len()]
}
/// Only curated general-knowledge aliases participate in spelling correction.
pub fn correction(query: &str) -> Option<String> {
    let normalized = normalize(query);
    if normalized.len() < 5 || normalized.split_whitespace().count() > 6 {
        return None;
    }
    static ALIASES: std::sync::OnceLock<Vec<(String, String, usize)>> = std::sync::OnceLock::new();
    let aliases = ALIASES.get_or_init(|| {
        crate::virtual_web::repository()
            .pack
            .entities
            .iter()
            .flat_map(|e| std::iter::once(&e.name).chain(e.aliases.iter()))
            .map(|name| {
                let key = normalize(name);
                let length = key.chars().count();
                (name.clone(), key, length)
            })
            .collect()
    });
    let length = normalized.chars().count();
    let threshold = if normalized.len() > 10 { 3 } else { 1 };
    aliases
        .iter()
        .filter(|(_, candidate, candidate_length)| {
            length.abs_diff(*candidate_length) <= threshold
                && candidate.chars().next() == normalized.chars().next()
        })
        .filter_map(|(name, candidate, _)| {
            let delta = distance(&normalized, candidate);
            (delta > 0 && delta <= threshold).then_some((delta, name))
        })
        .min_by_key(|(delta, name)| (*delta, *name))
        .map(|(_, name)| name.to_lowercase())
}

#[cfg(test)]
mod tests {
    #[test]
    fn bounded_alias_candidates_preserve_curated_typos_and_reject_distant_names() {
        assert_eq!(
            super::correction("pato de boraxa").as_deref(),
            Some("pato de borracha")
        );
        assert!(super::correction("inexistente999 asteroide quadrado").is_none());
        assert_eq!(super::correction("Nexora"), None);
    }
}
