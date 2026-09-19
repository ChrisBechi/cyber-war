//! Deterministic projections of the campaign clock and mission flags.
//! Pack definitions are immutable; saves contain only active event receipts.
use super::{model::*, now, repository, EPOCH};
use crate::world::WorldState;
use std::collections::{BTreeMap, BTreeSet};

pub fn active(world: &WorldState, event: &EventDefinition) -> bool {
    world.playtime_seconds >= event.after_seconds
        && event
            .required_flag
            .as_ref()
            .is_none_or(|flag| world.flags.contains(flag))
}

fn timestamp(world: &WorldState, event: &EventDefinition) -> u64 {
    let scheduled = EPOCH.saturating_add(event.after_seconds);
    if event.required_flag.is_none() {
        scheduled
    } else {
        world
            .web
            .event_started_at
            .get(&event.id)
            .copied()
            .unwrap_or_else(|| now(world))
            .min(now(world))
            .max(scheduled)
    }
}

fn ordered(world: &WorldState) -> Vec<&EventDefinition> {
    let mut events: Vec<_> = repository()
        .pack
        .events
        .iter()
        .filter(|e| active(world, e))
        .collect();
    // An event activated later wins. ID breaks simultaneous ties consistently.
    events.sort_by_key(|e| (timestamp(world, e), &e.id));
    events
}

pub fn published_at(world: &WorldState, doc: &Document) -> Option<u64> {
    let publisher = repository()
        .pack
        .events
        .iter()
        .find(|e| e.publish.contains(&doc.id));
    match publisher {
        Some(event) if active(world, event) => Some(timestamp(world, event).max(doc.published_at)),
        Some(_) => None,
        None => Some(doc.published_at),
    }
}

pub fn removed(world: &WorldState, id: &str) -> bool {
    repository()
        .pack
        .events
        .iter()
        .any(|e| active(world, e) && e.remove.iter().any(|d| d == id))
}

pub fn offer_history(world: &WorldState, doc: &Document) -> Vec<OfferVersion> {
    let Detail::Product {
        price_cents, stock, ..
    } = &doc.detail
    else {
        return Vec::new();
    };
    let mut price = *price_cents;
    let mut availability = stock.clone();
    let mut versions = vec![OfferVersion {
        timestamp: doc.published_at,
        price_cents: price,
        stock: availability.clone(),
    }];
    for event in ordered(world) {
        for effect in event.offers.iter().filter(|e| e.document_id == doc.id) {
            let next_price = effect.price_cents.unwrap_or(price);
            let next_stock = effect.stock.as_ref().unwrap_or(&availability).clone();
            if next_price != price || next_stock != availability {
                price = next_price;
                availability = next_stock;
                versions.push(OfferVersion {
                    timestamp: timestamp(world, event),
                    price_cents: price,
                    stock: availability.clone(),
                });
            }
        }
    }
    versions
}

pub fn project(world: &WorldState, base: &Document) -> Document {
    let mut doc = base.clone();
    let published = published_at(world, base).unwrap_or(base.published_at);
    let shift = published.saturating_sub(base.published_at);
    doc.published_at = published;
    for comment in &mut doc.comments {
        comment.published_at = comment.published_at.saturating_add(shift);
    }
    doc.comments.retain(|c| c.published_at <= now(world));
    let history = offer_history(world, base);
    if let Some(latest) = history.last() {
        if let Detail::Product {
            price_cents, stock, ..
        } = &mut doc.detail
        {
            *price_cents = latest.price_cents;
            *stock = latest.stock.clone();
            if history.len() > 1 {
                doc.updated_at = Some(latest.timestamp);
            }
        }
    }
    doc
}

pub fn purchasable(stock: &str) -> bool {
    matches!(stock, "IN_STOCK" | "LOW_STOCK")
}

/// Next visible boundary in campaign seconds. Polling does no SQL work until it is due.
pub fn next_change(world: &WorldState) -> Option<u64> {
    let mut next = repository()
        .pack
        .events
        .iter()
        .filter(|e| {
            e.after_seconds > world.playtime_seconds
                && e.required_flag
                    .as_ref()
                    .is_none_or(|f| world.flags.contains(f))
        })
        .map(|e| e.after_seconds)
        .min();
    let mut consider = |at: u64| {
        let second = at.saturating_sub(EPOCH);
        if second > world.playtime_seconds && next.is_none_or(|current| second < current) {
            next = Some(second);
        }
    };
    for ad in &repository().pack.ads {
        consider(EPOCH.saturating_add(ad.starts_at));
        consider(EPOCH.saturating_add(ad.ends_at));
    }
    for doc in &repository().pack.documents {
        if !doc.required_flags.iter().all(|f| world.flags.contains(f)) || removed(world, &doc.id) {
            continue;
        }
        if let Some(published) = published_at(world, doc) {
            consider(published);
            let shift = published.saturating_sub(doc.published_at);
            for comment in &doc.comments {
                consider(comment.published_at.saturating_add(shift));
            }
        }
    }
    next
}

pub fn sync(world: &mut WorldState) {
    let receipts: BTreeMap<_, _> = repository()
        .pack
        .events
        .iter()
        .filter(|e| active(world, e))
        .map(|e| (e.id.clone(), timestamp(world, e)))
        .collect();
    let events: BTreeSet<_> = receipts.keys().cloned().collect();
    let removed = repository()
        .pack
        .events
        .iter()
        .filter(|e| events.contains(&e.id))
        .flat_map(|e| e.remove.iter().cloned())
        .collect();
    if events != world.web.events
        || receipts != world.web.event_started_at
        || removed != world.web.removed
    {
        world.web.events = events;
        world.web.event_started_at = receipts;
        world.web.removed = removed;
        world.web.revision = world.web.revision.saturating_add(1);
    }
    world
        .web
        .pack_versions
        .entry(repository().pack.id.clone())
        .and_modify(|version| *version = (*version).max(repository().pack.version))
        .or_insert(repository().pack.version);
}
