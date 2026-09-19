//! Paid placements and historical captures use the same visibility rules as organic pages.
use super::{events, model::*, now, repository, visible, EPOCH};
use crate::{error::GameResult, vfs::domain, world::WorldState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Campaign {
    pub id: String,
    pub advertiser: String,
    pub document_id: String,
    pub title: String,
    pub description: String,
    pub keywords: Vec<String>,
    #[serde(default)]
    pub targeting: Vec<String>,
    #[serde(default)]
    pub quality: u8,
    #[serde(default)]
    pub fraud_risk: u8,
    pub budget_cents: u64,
    pub click_cost_cents: u64,
    pub starts_at: u64,
    pub ends_at: u64,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ad {
    pub id: String,
    pub advertiser: String,
    pub title: String,
    pub description: String,
    pub url: String,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Capture {
    pub timestamp: u64,
    pub original_url: String,
}

fn eligible(world: &WorldState, c: &Campaign) -> bool {
    c.starts_at <= world.playtime_seconds
        && world.playtime_seconds < c.ends_at
        && c.click_cost_cents > 0
        && world
            .web
            .ad_clicks
            .get(&c.id)
            .copied()
            .unwrap_or(0)
            .saturating_add(1)
            .saturating_mul(c.click_cost_cents)
            <= c.budget_cents
        && repository()
            .document(&c.document_id)
            .is_some_and(|d| visible(world, d))
}
pub fn ads(world: &WorldState, query: &str) -> Vec<Ad> {
    placements(world, query, "SEARCH")
}
pub fn placements(world: &WorldState, query: &str, surface: &str) -> Vec<Ad> {
    let normalized = crate::search::documents::normalize(query);
    if normalized.is_empty() {
        return Vec::new();
    }
    let mut eligible: Vec<_> = repository()
        .pack
        .ads
        .iter()
        .filter(|c| eligible(world, c))
        .filter(|c| c.targeting.is_empty() || c.targeting.iter().any(|target| target == surface))
        .filter(|c| {
            c.keywords
                .iter()
                .any(|k| normalized.contains(&crate::search::documents::normalize(k)))
        })
        .collect();
    eligible.sort_by_key(|c| {
        (
            std::cmp::Reverse(i16::from(c.quality) - i16::from(c.fraud_risk) / 2),
            &c.id,
        )
    });
    eligible
        .into_iter()
        .take(2)
        .map(|c| Ad {
            id: c.id.clone(),
            advertiser: c.advertiser.clone(),
            title: c.title.clone(),
            description: c.description.clone(),
            url: repository().url(repository().document(&c.document_id).unwrap()),
        })
        .collect()
}
pub fn click(world: &mut WorldState, id: &str) -> GameResult<String> {
    if !world.network.connected {
        return Err(domain("Sem conexão com a internet virtual."));
    }
    let c = repository()
        .pack
        .ads
        .iter()
        .find(|c| c.id == id)
        .filter(|c| eligible(world, c))
        .ok_or_else(|| domain("Este anúncio não está mais disponível."))?;
    *world.web.ad_clicks.entry(c.id.clone()).or_default() += 1;
    world.web.revision = world.web.revision.saturating_add(1);
    Ok(repository().url(repository().document(&c.document_id).unwrap()))
}
fn capture(world: &WorldState, doc: &Document) -> Option<u64> {
    let publication = events::published_at(world, doc)?;
    if !doc.required_flags.iter().all(|f| world.flags.contains(f)) {
        return None;
    }
    // Search overrides may revoke access; an archive must not bypass that decision.
    if world.search.documents.contains_key(&doc.id) {
        return None;
    }
    repository()
        .pack
        .events
        .iter()
        .filter(|e| e.remove.contains(&doc.id) && events::active(world, e))
        .map(|e| {
            if e.required_flag.is_some() {
                world
                    .web
                    .event_started_at
                    .get(&e.id)
                    .copied()
                    .unwrap_or(now(world))
            } else {
                EPOCH.saturating_add(e.after_seconds)
            }
        })
        .filter_map(|removed| removed.checked_sub(1))
        .filter(|date| *date >= publication && *date <= now(world))
        .min()
}
pub fn archive_url(world: &WorldState, doc: &Document) -> Option<String> {
    capture(world, doc).map(|_| format!("https://www.memoria.web/snapshot/{}", doc.id))
}
pub fn archive(world: &WorldState, page: &mut Page) {
    if page.brand.platform != Platform::Archive {
        return;
    }
    let repo = repository();
    let prefix = format!("https://www.{}/snapshot/", page.brand.domain);
    if let Some(id) = page
        .canonical_url
        .strip_prefix(&prefix)
        .map(|s| s.split('?').next().unwrap_or(s))
    {
        if let Some((doc, time)) = repo
            .document(id)
            .and_then(|d| capture(world, d).map(|t| (d, t)))
        {
            let mut past = world.clone();
            past.playtime_seconds = time.saturating_sub(EPOCH);
            let mut snapshot = events::project(&past, doc);
            snapshot.blocks = super::materialize::blocks(doc);
            snapshot.path = format!("/snapshot/{id}");
            // Archived links retain their original URLs; do not expose current personal comments.
            page.document = Some(snapshot);
            page.status = 200;
            page.capture = Some(Capture {
                timestamp: time,
                original_url: repo.url(doc),
            });
            page.related.clear();
            page.offer_history.clear();
            page.ads.clear();
        }
    } else if page.status == 200 {
        let query = crate::search::documents::normalize(&page.query);
        let cards: Vec<_> = repo
            .pack
            .documents
            .iter()
            .filter_map(|d| capture(world, d).map(|time| (d, time)))
            .filter(|(d, _)| {
                query.split_whitespace().all(|term| {
                    crate::search::documents::normalize(&format!("{} {}", d.title, d.summary))
                        .contains(term)
                })
            })
            .map(|(d, time)| {
                let mut past = world.clone();
                past.playtime_seconds = time.saturating_sub(EPOCH);
                let mut c = repo.card(&past, d);
                c.url = format!("{prefix}{}", d.id);
                c
            })
            .collect();
        page.total = cards.len();
        page.cards = cards
            .into_iter()
            .skip(page.offset)
            .take(super::PAGE_SIZE)
            .collect();
    }
}
