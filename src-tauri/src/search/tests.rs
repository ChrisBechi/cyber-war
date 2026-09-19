use super::*;
use crate::{
    mission::{Condition, Effect, MissionDefinition, MissionEngine, Stage},
    mission_runtime,
    service::GameService,
};
use mission_modifiers::SearchEffect;
use rusqlite::Connection;

fn world() -> WorldState {
    let mut w = WorldState::new("neo", "pc").unwrap();
    w.network.connected = true;
    w
}
fn results(w: &WorldState, query: &str) -> SearchResponse {
    VirtualSearchEngine::new(w)
        .search(query, "all", None, 0)
        .unwrap()
}

#[test]
fn catalog_is_valid_and_index_can_intersect_thousands_of_documents() {
    for doc in index::base().documents.values() {
        validate_document(doc).unwrap();
    }
    let docs: Vec<_> = (0..5000)
        .map(|n| {
            let mut d = index::base().documents["orion"].clone();
            d.id = format!("doc-{n:04}");
            d.keywords.push(format!("needle{n}"));
            d
        })
        .collect();
    let index = index::SearchIndex::new(docs);
    assert_eq!(
        index.candidates("orion needle3456"),
        BTreeSet::from(["doc-3456".into()])
    );
    assert!(index.candidates("orion missing").is_empty());
    assert_eq!(
        index.candidates("needle3456 orion"),
        index.candidates("orion needle3456")
    );
    assert_eq!(
        index.candidates("orion orion needle3456"),
        index.candidates("orion needle3456")
    );
    assert_eq!(index.candidates("").len(), 5000);
}

#[test]
fn relevance_accents_and_lucky_order_are_deterministic() {
    let w = world();
    assert_eq!(results(&w, "Orion").documents[0].id, "orion");
    assert_eq!(results(&w, "orion aquisição").documents[0].id, "orion");
    assert_eq!(results(&w, "orion aquisicao").documents[0].id, "orion");
    assert_eq!(results(&w, "zxqv987semcorrespondencia").total, 0);
    assert_eq!(results(&w, "").total, 0);
    let ids = || {
        results(&w, "orion")
            .documents
            .into_iter()
            .map(|d| d.id)
            .collect::<Vec<_>>()
    };
    assert_eq!(ids(), ids());
    assert_eq!(
        results(&w, "Orion & tecnologia").query,
        "Orion & tecnologia"
    );
}

#[test]
fn story_gates_filter_search_suggestions_images_and_direct_navigation() {
    let mut w = world();
    assert_eq!(results(&w, "NULL").total, 0);
    assert_eq!(results(&w, "fakebook").total, 0);
    assert!(!VirtualSearchEngine::new(&w)
        .suggest("orion")
        .unwrap()
        .contains(&"orion vazamento".into()));
    let unavailable = crate::browser::navigate(&mut w, "https://www.b1.tech/orion").unwrap();
    assert_eq!(
        serde_json::to_value(unavailable).unwrap()["virtualWeb"]["status"],
        404
    );
    w.flags.insert("SESSION_1_COMPLETE".into());
    assert_eq!(results(&w, "NULL").total, 2);
    assert!(VirtualSearchEngine::new(&w)
        .suggest("orion")
        .unwrap()
        .contains(&"orion vazamento".into()));
    let available = crate::browser::navigate(&mut w, "https://www.b1.tech/orion").unwrap();
    assert_eq!(
        serde_json::to_value(available).unwrap()["virtualWeb"]["status"],
        200
    );
    w.missions.insert(
        "girl".into(),
        crate::world::MissionProgress {
            status: "active".into(),
            stage: 0,
            attempts: 1,
        },
    );
    assert_eq!(results(&w, "fakebook").total, 1);
    assert!(VirtualSearchEngine::new(&w)
        .suggest("fake")
        .unwrap()
        .contains(&"fakebook perfil".into()));
    w.network.connected = false;
    assert!(VirtualSearchEngine::new(&w)
        .search("orion", "all", None, 0)
        .is_err());
}

#[test]
fn mission_modifiers_add_remove_hide_suggest_and_rerank() {
    let mut w = world();
    let mut document = index::base().documents["orion"].clone();
    document.id = "investigation".into();
    document.title = "Orion — investigação".into();
    SearchEffect::AddDocument {
        document: Box::new(document),
    }
    .apply(&mut w)
    .unwrap();
    assert_eq!(results(&w, "investigação").total, 1);
    SearchEffect::ChangeVisibility {
        id: "investigation".into(),
        visibility: Visibility::Hidden,
    }
    .apply(&mut w)
    .unwrap();
    assert_eq!(results(&w, "investigação").total, 0);
    SearchEffect::ChangeVisibility {
        id: "investigation".into(),
        visibility: Visibility::Public,
    }
    .apply(&mut w)
    .unwrap();
    SearchEffect::ChangeRanking {
        id: "investigation".into(),
        boost: 50_000,
    }
    .apply(&mut w)
    .unwrap();
    assert_eq!(results(&w, "orion").documents[0].id, "investigation");
    SearchEffect::AddSuggestion {
        id: "new".into(),
        suggestion: mission_modifiers::SearchSuggestion {
            text: "orion investigação".into(),
            required_flags: vec![],
        },
    }
    .apply(&mut w)
    .unwrap();
    assert!(VirtualSearchEngine::new(&w)
        .suggest("orion")
        .unwrap()
        .contains(&"orion investigação".into()));
    SearchEffect::RemoveDocument {
        id: "investigation".into(),
    }
    .apply(&mut w)
    .unwrap();
    assert_eq!(results(&w, "investigação").total, 0);
}

#[test]
fn actual_mission_engine_journals_search_changes_without_reverting_account_settings() {
    let mut w = world();
    let engine = MissionEngine {
        definitions: vec![MissionDefinition {
            id: "search-story".into(),
            title: "Search story".into(),
            description: "test".into(),
            contact: "VEX".into(),
            session: 0,
            requirements: vec![],
            start_triggers: vec![],
            on_start: vec![
                Effect::Search {
                    effect: SearchEffect::RemoveDocument { id: "orion".into() },
                },
                Effect::Search {
                    effect: SearchEffect::ChangeRanking {
                        id: "orion-campus".into(),
                        boost: 50_000,
                    },
                },
            ],
            stages: vec![Stage {
                objective: "wait".into(),
                hint: "".into(),
                conditions: vec![Condition::Flag { key: "DONE".into() }],
                choices: vec![],
                effects: vec![],
            }],
            outcomes: vec![],
        }],
    };
    engine.start(&mut w, "search-story").unwrap();
    assert_eq!(results(&w, "orion").documents[0].id, "orion-campus");
    accounts::authenticate(&mut w, "neo@goggle.com", "virtual", Some("Neo")).unwrap();
    w.search.history_enabled = true;
    mission_runtime::discard(&mut w, "search-story").unwrap();
    assert_eq!(results(&w, "orion").documents[0].id, "orion");
    assert!(accounts::session(&w).account.is_some());
    assert!(w.search.history_enabled);
}

#[test]
fn reverse_search_matches_virtual_urls_and_vfs_content_after_a_rename() {
    let mut w = world();
    let source = "https://www.orion.com/media/campus.svg";
    let response = VirtualSearchEngine::new(&w)
        .search("", "all", Some(source), 0)
        .unwrap();
    assert!(response.documents.iter().any(|d| d.id == "orion"));
    let path = "/home/kali/Pictures/unknown.svg";
    w.vfs
        .write(
            path,
            include_str!("../../../public/assets/goggle/orion-campus.svg"),
            "kali",
        )
        .unwrap();
    let from_vfs = VirtualSearchEngine::new(&w)
        .search("", "all", Some(path), 0)
        .unwrap();
    assert_eq!(response.total, from_vfs.total);
    assert!(images::files(&w).contains(&path.into()));
    let encoded = {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode(include_bytes!(
            "../../../public/assets/goggle/orion-campus.svg"
        ))
    };
    crate::binary::import(
        &mut w,
        "/home/kali/Pictures/copy.svg",
        &encoded,
        "image/svg+xml",
        None,
        "kali",
    )
    .unwrap();
    assert_eq!(
        VirtualSearchEngine::new(&w)
            .search("", "all", Some("/home/kali/Pictures/copy.svg"), 0)
            .unwrap()
            .total,
        response.total
    );
    SearchEffect::ChangeVisibility {
        id: "orion-image".into(),
        visibility: Visibility::Hidden,
    }
    .apply(&mut w)
    .unwrap();
    assert_eq!(
        VirtualSearchEngine::new(&w)
            .search("orion", "images", None, 0)
            .unwrap()
            .total,
        0
    );
    assert!(!VirtualSearchEngine::new(&w)
        .search("", "all", Some(path), 0)
        .unwrap()
        .documents
        .iter()
        .any(|d| d.id == "orion-image"));
}

#[test]
fn sandbox_rejects_host_paths_unregistered_urls_and_active_schemes() {
    let mut w = world();
    for source in [
        "C:\\Users\\chris\\photo.jpg",
        "file:///C:/photo.jpg",
        "//server/share/a.jpg",
        "/home/kali/../private.jpg",
        "https://www.google.com/photo.jpg",
        "https://example.com/img.jpg",
        "javascript:alert(1)",
        "data:image/png;base64,abcd",
        "https://orion.com@evil.com/media/campus.svg",
    ] {
        assert!(
            VirtualSearchEngine::new(&w)
                .search("", "all", Some(source), 0)
                .is_err(),
            "{source}"
        );
    }
    assert!(crate::browser::navigate(&mut w, "https://google.com").is_err());
    assert!(crate::browser::navigate(&mut w, "https://goggle.com.evil.test").is_err());
    assert!(VirtualSearchEngine::new(&w)
        .search("https://google.com", "all", None, 0)
        .unwrap()
        .documents
        .is_empty());
    for address in [
        "goggle.com",
        "www.goggle.com",
        "https://www.goggle.com",
        "https://www.goggle.com/search?q=Orion%20Tecnologia",
        "https://www.goggle.com/about",
        "https://www.goggle.com/privacy",
    ] {
        assert!(
            crate::browser::navigate(&mut w, address).is_ok(),
            "{address}"
        );
    }
}

#[test]
fn accounts_history_and_narrative_overrides_survive_a_save_and_old_saves_migrate() {
    let mut service = GameService::new(Connection::open_in_memory().unwrap()).unwrap();
    service.new_game(1, "neo", "pc", false).unwrap();
    service
        .mutate(
            |_, w, _| {
                accounts::authenticate(w, "neo@goggle.com", "virtual", Some("Neo"))?;
                w.search.history_enabled = true;
                remember(w, "Orion");
                SearchEffect::RemoveDocument { id: "orion".into() }.apply(w)
            },
            false,
        )
        .unwrap();
    service.end_session().unwrap();
    service.load(1, false).unwrap();
    let w = service.world().unwrap();
    assert_eq!(
        accounts::session(w).account.unwrap().email,
        "neo@goggle.com"
    );
    assert_eq!(w.search.history, vec!["Orion"]);
    assert!(w.search.documents["orion"].is_none());
    let mut snapshot = serde_json::to_value(w).unwrap();
    snapshot.as_object_mut().unwrap().remove("search");
    let migrated: WorldState = serde_json::from_value(snapshot).unwrap();
    assert!(migrated.search.accounts.is_empty());
    assert!(!migrated.search.history_enabled);
    let mut w = world();
    assert!(accounts::authenticate(&mut w, "neo@gmail.com", "virtual", Some("Neo")).is_err());
    accounts::authenticate(&mut w, "neo@goggle.com", "virtual", Some("Neo")).unwrap();
    w.search.session = None;
    assert!(accounts::session(&w).account.is_none());
    assert!(accounts::authenticate(&mut w, "neo@goggle.com", "wrong", None).is_err());
    accounts::authenticate(&mut w, "neo@goggle.com", "virtual", None).unwrap();
    assert!(!serde_json::to_string(&accounts::session(&w))
        .unwrap()
        .contains("passwordVirtual"));
    remember(&mut w, "not saved");
    assert!(w.search.history.is_empty());
    w.search.history_enabled = true;
    remember(&mut w, "saved");
    w.search.history.clear();
    assert!(accounts::session(&w).history.is_empty());
}

#[test]
fn paginated_results_do_not_repeat_and_ties_use_stable_ids() {
    let mut w = world();
    for n in 0..45 {
        let mut doc = index::base().documents["orion"].clone();
        doc.id = format!("extra-{n:02}");
        SearchEffect::AddDocument {
            document: Box::new(doc),
        }
        .apply(&mut w)
        .unwrap();
    }
    let first = results(&w, "orion");
    let second = VirtualSearchEngine::new(&w)
        .search("orion", "all", None, 20)
        .unwrap();
    assert_eq!(first.documents.len(), 20);
    assert_eq!(second.documents.len(), 20);
    assert_eq!(first.total, second.total);
    assert!(first
        .documents
        .iter()
        .all(|d| second.documents.iter().all(|e| e.id != d.id)));
}
