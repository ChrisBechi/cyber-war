use super::*;
use crate::{search::VirtualSearchEngine, service::GameService};
use rusqlite::Connection;

const PRODUCT: &str = "web-shopnow-produto-nexphone-x2";
const URL: &str = "shopnow.com/produto/nexphone-x2";
const PROMOTION: &str = "web-shopnow-ofertas-nexphone-x2";
const THREAD: &str = "web-redditor-t-x2-ultimas-unidades";

fn world() -> WorldState {
    let mut world = WorldState::new("lia", "pc").unwrap();
    world.network.connected = true;
    world
}
fn page(world: &WorldState, url: &str) -> Page {
    resolve(world, url).unwrap().unwrap()
}
fn offer(page: &Page) -> (u64, &str) {
    match &page.document.as_ref().unwrap().detail {
        Detail::Product {
            price_cents, stock, ..
        } => (*price_cents, stock),
        _ => panic!("expected product"),
    }
}

#[test]
fn timeline_keeps_detail_list_cart_and_cached_search_consistent() {
    let mut w = world();
    interact(&mut w, PRODUCT, "cart", "").unwrap();
    for (second, price, stock, versions) in [
        (0, 189900, "IN_STOCK", 1),
        (599, 189900, "IN_STOCK", 1),
        (600, 179900, "IN_STOCK", 2),
        (1199, 179900, "IN_STOCK", 2),
        (1200, 179900, "LOW_STOCK", 3),
        (1799, 179900, "LOW_STOCK", 3),
        (1800, 189900, "OUT_OF_STOCK", 4),
        (2399, 189900, "OUT_OF_STOCK", 4),
        (2400, 184900, "IN_STOCK", 5),
    ] {
        w.playtime_seconds = second;
        sync_events(&mut w);
        let p = page(&w, URL);
        assert_eq!(offer(&p), (price, stock));
        assert_eq!(p.offer_history.len(), versions);
        assert_eq!(p.cart_count, 1);
        assert_eq!(p.cart[0].card.price_cents, Some(price));
        assert_eq!(p.cart[0].available, events::purchasable(stock));
        let listing = page(&w, "shopnow.com/search?q=NexPhone");
        let card = listing.cards.iter().find(|d| d.id == PRODUCT).unwrap();
        assert_eq!(card.price_cents, Some(price));
        assert_eq!(card.stock.as_deref(), Some(stock));
        for _ in 0..2 {
            let results = VirtualSearchEngine::new(&w)
                .search("NexPhone", "shopping", None, 0)
                .unwrap();
            let current = results
                .documents
                .iter()
                .find(|d| d.id == PRODUCT)
                .unwrap()
                .offer
                .as_ref()
                .unwrap();
            assert_eq!(current.price_cents, price);
            assert_eq!(current.stock, stock);
            assert_eq!(
                results.documents.iter().any(|d| d.id == PROMOTION),
                (600..1800).contains(&second)
            );
        }
        let before = serde_json::to_value(&w.web).unwrap();
        sync_events(&mut w);
        assert_eq!(
            serde_json::to_value(&w.web).unwrap(),
            before,
            "sync must be idempotent"
        );
    }
    assert_eq!(
        w.web.event_started_at["NEXPHONE_CAMPAIGN_OPEN"],
        EPOCH + 600
    );
    assert_eq!(w.web.event_started_at["NEXPHONE_RESTOCKED"], EPOCH + 2400);
    let base = repository().document(PRODUCT).unwrap();
    assert!(matches!(
        base.detail,
        Detail::Product {
            price_cents: 189900,
            ..
        }
    ));
}

#[test]
fn future_publications_comments_and_withdrawals_respect_every_discovery_surface() {
    let mut w = world();
    let repo = repository();
    for (second, expected) in [
        (0, false),
        (599, false),
        (600, true),
        (1799, true),
        (1800, false),
    ] {
        w.playtime_seconds = second;
        // Reads also remain correct when a clock/flag change hasn't been projected to receipts yet.
        let p = page(&w, &repo.url(repo.document(PROMOTION).unwrap()));
        assert_eq!(p.status, if expected { 200 } else { 404 });
        let engine = VirtualSearchEngine::new(&w);
        for mode in ["all", "images", "shopping"] {
            let result = engine
                .search("condição especial NexPhone", mode, None, 0)
                .unwrap();
            assert_eq!(result.documents.iter().any(|d| d.id == PROMOTION), expected);
            if !expected {
                assert!(result
                    .images
                    .iter()
                    .all(|i| !i.document_ids.iter().any(|id| id == PROMOTION)));
            }
        }
        assert_eq!(
            engine
                .suggest("Condição especial")
                .unwrap()
                .iter()
                .any(|s| s.contains("NexPhone")),
            expected
        );
    }
    w.playtime_seconds = 1200;
    let url = repo.url(repo.document(THREAD).unwrap());
    assert!(page(&w, &url).document.unwrap().comments.is_empty());
    w.playtime_seconds = 1260;
    assert_eq!(page(&w, &url).document.unwrap().comments.len(), 1);
    w.playtime_seconds = 1800;
    assert!(interact(&mut w, PRODUCT, "cart", "").is_err());
}

#[test]
fn flag_receipts_and_clock_rollback_restore_the_correct_version() {
    let mut w = world();
    w.playtime_seconds = 700;
    w.flags.insert("SESSION_1_COMPLETE".into());
    sync_events(&mut w);
    let p = page(&w, "b1.tech/noticias/revisao-de-acessos");
    assert_eq!(p.document.unwrap().published_at, EPOCH + 700);
    assert_eq!(w.web.event_started_at["ORION_ACCESS_REVIEW"], EPOCH + 700);
    w.playtime_seconds = 1500;
    sync_events(&mut w);
    assert_eq!(w.web.event_started_at["ORION_ACCESS_REVIEW"], EPOCH + 700);
    interact(
        &mut w,
        "web-redditor-t-registros-orion",
        "comment",
        "Resposta posterior.",
    )
    .unwrap();
    w.playtime_seconds = 710;
    sync_events(&mut w);
    assert!(page(&w, "redditor.com/t/registros-orion")
        .document
        .unwrap()
        .comments
        .is_empty());
    w.flags.remove("SESSION_1_COMPLETE");
    w.playtime_seconds = 0;
    sync_events(&mut w);
    assert_eq!(page(&w, "b1.tech/noticias/revisao-de-acessos").status, 404);
    assert!(w.web.events.is_empty());
    assert!(w.web.event_started_at.is_empty());
    assert_eq!(offer(&page(&w, URL)), (189900, "IN_STOCK"));
    assert_eq!(page(&w, URL).offer_history.len(), 1);
}

#[test]
fn next_boundary_tracks_publication_and_delayed_comments_without_polling_writes() {
    let mut w = world();
    assert_eq!(events::next_change(&w), Some(600));
    w.playtime_seconds = 1200;
    sync_events(&mut w);
    assert_eq!(events::next_change(&w), Some(1260));
    w.playtime_seconds = 1260;
    assert_eq!(events::next_change(&w), Some(1800));
    w.playtime_seconds = 2400;
    sync_events(&mut w);
    assert_eq!(events::next_change(&w), Some(864000));
}

#[test]
fn hidden_cart_rows_can_be_removed_without_leaking_the_document() {
    let mut w = world();
    interact(&mut w, PRODUCT, "cart", "").unwrap();
    w.search.documents.insert(PRODUCT.into(), None);
    let p = page(&w, "shopnow.com");
    assert_eq!(p.cart_count, 1);
    assert_eq!(p.cart.len(), 1);
    assert!(!p.cart[0].available);
    assert_eq!(p.cart[0].card.title, "Produto indisponível");
    assert!(p.cart[0].card.url.is_empty());
    interact(&mut w, PRODUCT, "removeCart", "").unwrap();
    assert!(w.web.cart.is_empty());
}

#[test]
fn event_receipts_and_live_offers_survive_sqlite_in_five_independent_slots() {
    let mut game = GameService::new(Connection::open_in_memory().unwrap()).unwrap();
    let stages = [
        (30, 189900, "IN_STOCK"),
        (630, 179900, "IN_STOCK"),
        (1230, 179900, "LOW_STOCK"),
        (1830, 189900, "OUT_OF_STOCK"),
        (2430, 184900, "IN_STOCK"),
    ];
    for (i, (second, _, _)) in stages.iter().enumerate() {
        game.new_game(i as i64 + 1, "lia", "pc", false).unwrap();
        game.mutate(
            |_, w, _| {
                w.network.connected = true;
                interact(w, PRODUCT, "cart", "")?;
                w.playtime_seconds = *second;
                Ok(())
            },
            true,
        )
        .unwrap();
        game.end_session().unwrap();
    }
    for (i, (_, price, stock)) in stages.iter().enumerate() {
        game.load(i as i64 + 1, false).unwrap();
        let w = game.world().unwrap();
        assert_eq!(offer(&page(w, URL)), (*price, *stock));
        assert_eq!(w.web.cart[PRODUCT], 1);
        assert_eq!(w.web.event_started_at.len(), i);
        assert!(serde_json::to_vec(&w.web).unwrap().len() < 2000);
        game.end_session().unwrap();
    }
}

#[test]
fn version_one_migrates_without_catalog_copies_and_future_versions_still_fail() {
    let mut old = serde_json::to_value(world()).unwrap();
    old["web"].as_object_mut().unwrap().remove("eventStartedAt");
    old["web"]["packVersions"]["web-core"] = serde_json::json!(1);
    let mut w: WorldState = serde_json::from_value(old).unwrap();
    sync_events(&mut w);
    assert_eq!(w.web.pack_versions["web-core"], 3);
    w.validate().unwrap();
    w.web.pack_versions.insert("web-core".into(), 4);
    sync_events(&mut w);
    assert!(validate_state(&w.web).is_err());
}
