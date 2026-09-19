use super::*;
use crate::{search::VirtualSearchEngine, service::GameService};
use rusqlite::Connection;
fn world() -> WorldState {
    let mut w = WorldState::new("lia", "pc").unwrap();
    w.network.connected = true;
    w
}
fn page(w: &WorldState, url: &str) -> Page {
    resolve(w, url).unwrap().unwrap()
}
const PROFILE: &str = "web-linkup-pessoas-nara-campos";
const LISTING: &str = "web-feiralivre-anuncios-radio-aurora";

#[test]
fn social_graph_feed_notifications_and_messages_are_local_and_bounded() {
    let mut w = world();
    assert!(page(&w, "linkup.com").community.unwrap().feed.is_empty());
    interact(&mut w, PROFILE, "follow", "").unwrap();
    let feed = page(&w, "linkup.com").community.unwrap();
    assert_eq!(feed.feed.len(), 1);
    assert_eq!(feed.notifications.len(), 1);
    interact(&mut w, &feed.notifications[0].id, "readNotification", "").unwrap();
    assert!(page(&w, "linkup.com").community.unwrap().notifications[0].read);
    interact(
        &mut w,
        "web-linkup-home",
        "post",
        "Meu caderno <script>não executa</script>",
    )
    .unwrap();
    assert_eq!(w.web.community.posts.len(), 1);
    interact(
        &mut w,
        PROFILE,
        "message",
        "Onde encontro os projetos públicos?",
    )
    .unwrap();
    assert_eq!(w.web.community.messages.len(), 2);
    assert!(w.web.community.messages[1].incoming);
    let post_id = feed.feed[0].id.clone();
    interact(&mut w, &post_id, "like", "").unwrap();
    assert!(w.web.likes.contains(&post_id));
    w.search.documents.insert(PROFILE.into(), None);
    let hidden = page(&w, "linkup.com").community.unwrap();
    assert!(hidden.feed.is_empty());
    assert!(hidden.notifications.is_empty());
    assert!(hidden.messages.is_empty());
    assert!(interact(&mut w, &post_id, "like", "").is_err());
    w.search.documents.remove(PROFILE);
    assert!(page(&w, "feiralivre.com")
        .community
        .unwrap()
        .messages
        .is_empty());
    assert!(interact(&mut w, PROFILE, "post", &"a".repeat(1001)).is_err());
    interact(&mut w, PROFILE, "follow", "").unwrap();
    assert!(page(&w, "linkup.com").community.unwrap().feed.is_empty());
    w.network.connected = false;
    assert!(interact(&mut w, PROFILE, "message", "Teste").is_err());
}
#[test]
fn classifieds_enforce_negotiation_transitions_and_keep_independent_slots() {
    let mut w = world();
    let other = world();
    assert!(interact(&mut w, LISTING, "sold", "").is_err());
    interact(&mut w, LISTING, "offer", "100").unwrap();
    assert!(w.web.community.listings[LISTING].offer_cents.is_none());
    interact(&mut w, LISTING, "offer", "8000").unwrap();
    assert_eq!(w.web.community.listings[LISTING].offer_cents, Some(8000));
    interact(&mut w, LISTING, "reserve", "").unwrap();
    let reserved = page(
        &w,
        "feiralivre.com/search?status=RESERVED&location=Porto%20Claro",
    );
    assert_eq!(reserved.cards.len(), 1);
    assert_eq!(
        reserved.cards[0].listing_status.as_deref(),
        Some("RESERVED")
    );
    assert!(!page(&w, "feiralivre.com/search?status=AVAILABLE")
        .cards
        .iter()
        .any(|d| d.id == LISTING));
    assert!(interact(&mut w, LISTING, "offer", "8500").is_err());
    interact(&mut w, LISTING, "release", "").unwrap();
    interact(&mut w, LISTING, "reserve", "").unwrap();
    interact(&mut w, LISTING, "sold", "").unwrap();
    assert!(interact(&mut w, LISTING, "reserve", "").is_err());
    assert_eq!(
        page(&other, "feiralivre.com/anuncios/radio-aurora")
            .community
            .unwrap()
            .listing
            .unwrap()
            .status,
        "AVAILABLE"
    );
}
#[test]
fn paid_results_expire_exhaust_budget_and_never_escape_gates_or_change_organic_order() {
    let mut w = world();
    assert!(discovery::ads(&w, "celular").is_empty());
    w.playtime_seconds = 600;
    sync_events(&mut w);
    let original = VirtualSearchEngine::new(&w)
        .search("celular", "all", None, 0)
        .unwrap();
    assert_eq!(original.ads.len(), 1);
    assert!(discovery::placements(&w, "celular", "ARCHIVE").is_empty());
    assert_eq!(discovery::placements(&w, "celular", "EDITORIAL").len(), 1);
    for _ in 0..200 {
        discovery::click(&mut w, "x2-lancamento").unwrap();
    }
    assert!(discovery::click(&mut w, "x2-lancamento").is_err());
    let cached = VirtualSearchEngine::new(&w)
        .search("celular", "all", None, 0)
        .unwrap();
    assert!(cached.ads.is_empty());
    assert_eq!(
        original.documents.iter().map(|d| &d.id).collect::<Vec<_>>(),
        cached.documents.iter().map(|d| &d.id).collect::<Vec<_>>()
    );
    w.web.ad_clicks.clear();
    w.playtime_seconds = 1800;
    sync_events(&mut w);
    assert!(discovery::ads(&w, "celular").is_empty());
}
#[test]
fn archive_preserves_old_content_only_after_removal_and_rollback_revokes_it() {
    let mut w = world();
    let url = "memoria.web/snapshot/web-shopnow-ofertas-nexphone-x2";
    assert_eq!(page(&w, url).status, 404);
    w.playtime_seconds = 1200;
    sync_events(&mut w);
    assert_eq!(page(&w, url).status, 404);
    w.playtime_seconds = 1800;
    sync_events(&mut w);
    let captured = page(&w, url);
    assert_eq!(captured.status, 200);
    assert_eq!(captured.capture.unwrap().timestamp, EPOCH + 1799);
    assert_eq!(page(&w, "shopnow.com/ofertas/nexphone-x2").status, 404);
    assert_eq!(page(&w, "memoria.web").cards.len(), 1);
    w.search
        .documents
        .insert("web-shopnow-ofertas-nexphone-x2".into(), None);
    assert_eq!(page(&w, url).status, 404);
    w.search.documents.clear();
    w.playtime_seconds = 600;
    sync_events(&mut w);
    assert_eq!(page(&w, url).status, 404);
}
#[test]
fn new_community_state_survives_sqlite_save_and_old_saves_default_cleanly() {
    let mut game = GameService::new(Connection::open_in_memory().unwrap()).unwrap();
    game.new_game(1, "lia", "pc", false).unwrap();
    game.mutate(
        |_, w, _| {
            w.network.connected = true;
            interact(w, PROFILE, "follow", "")?;
            interact(w, LISTING, "reserve", "")?;
            interact(w, "web-linkup-home", "post", "Uma descoberta para guardar")
        },
        false,
    )
    .unwrap();
    game.end_session().unwrap();
    game.load(1, false).unwrap();
    assert!(game
        .world()
        .unwrap()
        .web
        .community
        .following
        .contains(PROFILE));
    assert_eq!(game.world().unwrap().web.community.posts.len(), 1);
    assert_eq!(
        game.world().unwrap().web.community.listings[LISTING].status,
        "RESERVED"
    );
    let mut old = serde_json::to_value(world()).unwrap();
    old["web"].as_object_mut().unwrap().remove("community");
    let restored: WorldState = serde_json::from_value(old).unwrap();
    assert!(restored.web.community.posts.is_empty());
}

#[test]
fn all_community_deltas_survive_disk_reopen_in_five_slots() {
    let path = std::env::temp_dir().join(format!("cyber-web-{}.db", uuid::Uuid::new_v4()));
    {
        let mut game = GameService::new(Connection::open(&path).unwrap()).unwrap();
        for slot in 1..=5 {
            game.new_game(slot, &format!("player{slot}"), "pc", false)
                .unwrap();
            game.mutate(
                |_, w, _| {
                    w.network.connected = true;
                    w.playtime_seconds = 600;
                    sync_events(w);
                    interact(w, PROFILE, "follow", "")?;
                    for _ in 0..slot {
                        interact(
                            w,
                            "web-linkup-home",
                            "post",
                            &format!("Registro do slot {slot}"),
                        )?;
                        discovery::click(w, "x2-lancamento")?;
                    }
                    interact(w, LISTING, "reserve", "")?;
                    if slot % 2 == 0 {
                        interact(w, LISTING, "sold", "")?;
                    }
                    interact(w, LISTING, "message", "Conferir situação")
                },
                false,
            )
            .unwrap();
            game.end_session().unwrap();
        }
    }
    {
        let mut game = GameService::new(Connection::open(&path).unwrap()).unwrap();
        for slot in 1..=5 {
            game.load(slot, false).unwrap();
            let w = game.world().unwrap();
            assert_eq!(w.web.community.posts.len(), slot as usize);
            assert_eq!(w.web.ad_clicks["x2-lancamento"], slot as u64);
            assert!(w
                .web
                .community
                .posts
                .iter()
                .all(|p| p.text.ends_with(&slot.to_string())));
            assert_eq!(
                w.web.community.listings[LISTING].status,
                if slot % 2 == 0 { "SOLD" } else { "RESERVED" }
            );
            assert!(w
                .web
                .community
                .messages
                .last()
                .unwrap()
                .text
                .contains(if slot % 2 == 0 {
                    "vendido"
                } else {
                    "reservado"
                }));
            let before = serde_json::to_string(&w.web).unwrap();
            assert!(game
                .mutate(
                    |_, w, _| {
                        interact(w, "web-linkup-home", "post", "Must roll back")?;
                        Err::<(), _>(crate::vfs::domain("injected transaction failure"))
                    },
                    false
                )
                .is_err());
            assert_eq!(
                before,
                serde_json::to_string(&game.world().unwrap().web).unwrap()
            );
            game.end_session().unwrap();
        }
    }
    // Only the unique test database created above is removed.
    std::fs::remove_file(path).unwrap();
}

#[test]
#[ignore = "exports an isolated, playable SQLite fixture for native desktop QA"]
fn export_desktop_web_qa() {
    let directory =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../artifacts/desktop-web-qa");
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("game-hacker.db");
    assert!(
        !path.exists(),
        "QA fixture already exists; preserve it or select a fresh directory"
    );
    let mut game = GameService::new(Connection::open(path).unwrap()).unwrap();
    game.new_game(1, "webqa", "qa-pc", false).unwrap();
    game.mutate(
        |_, w, _| {
            w.network.connected = true;
            w.flags.extend([
                "V1_COMPLETE".into(),
                "WIFI_COMPLETE".into(),
                "SESSION_1_COMPLETE".into(),
                "GAME_DOWNLOADED".into(),
            ]);
            w.missions.clear();
            w.playtime_seconds = 700;
            sync_events(w);
            Ok(())
        },
        false,
    )
    .unwrap();
    game.end_session().unwrap();
}
