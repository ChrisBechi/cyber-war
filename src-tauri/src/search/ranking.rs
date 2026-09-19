use super::documents::{normalize, SearchDocument};
use crate::world::WorldState;
#[derive(Default, serde::Serialize)]
pub struct Components {
    pub authority: i64,
    pub popularity: i64,
    pub title: i64,
    pub keywords: i64,
    pub description: i64,
    pub phrase: i64,
    pub facets: i64,
    pub navigation: i64,
    pub mission: i64,
    pub narrative_override: i64,
}
impl Components {
    pub fn total(&self) -> i64 {
        self.authority
            + self.popularity
            + self.title
            + self.keywords
            + self.description
            + self.phrase
            + self.facets
            + self.navigation
            + self.mission
            + self.narrative_override
    }
}

pub fn score(doc: &SearchDocument, query: &str, world: &WorldState) -> i64 {
    breakdown(doc, query, world).total()
}
pub fn breakdown(doc: &SearchDocument, query: &str, world: &WorldState) -> Components {
    let title = normalize(&doc.title);
    let keywords = normalize(&doc.keywords.join(" "));
    let description = normalize(&doc.description);
    let mut parts = Components {
        authority: i64::from(doc.authority.min(100) * 2),
        popularity: i64::from(doc.popularity.min(100)),
        narrative_override: i64::from(*world.search.ranking.get(&doc.id).unwrap_or(&0)),
        ..Default::default()
    };
    for term in query.split_whitespace() {
        parts.title += if title.split_whitespace().any(|t| t == term) {
            1000
        } else {
            0
        };
        parts.keywords += if keywords.split_whitespace().any(|t| t == term) {
            600
        } else {
            0
        };
        parts.description += if description.split_whitespace().any(|t| t == term) {
            200
        } else {
            0
        };
    }
    if !query.is_empty() && title.contains(query) {
        parts.phrase += 1500;
    }
    if !query.is_empty() && super::query::terms(&doc.title) == query {
        parts.phrase += 2200;
    }
    if let Some(web) = crate::virtual_web::repository().document(&doc.id) {
        use crate::virtual_web::model::Detail;
        // Product facets belong below an exact product or an explanatory page.
        if doc.domain == "shopnow.com"
            && !query.contains("cabo")
            && web.entities.iter().any(|e| e == "usb")
        {
            parts.facets -= 500;
        }
        if matches!(web.detail, Detail::Lesson { .. }) {
            parts.facets -= 200;
        }
    }
    // A bare organization/domain query should prefer its landing page.
    let host = doc.domain.trim_start_matches("www.");
    if query == normalize(host.split('.').next().unwrap_or(""))
        && super::virtual_url_key(&doc.url).ok().as_deref() == Some(host)
    {
        parts.navigation += 1000;
    }
    parts.mission += doc
        .mission_tags
        .iter()
        .filter(|tag| {
            world
                .missions
                .get(*tag)
                .is_some_and(|m| m.status == "active")
        })
        .count() as i64
        * 400;
    parts
}
