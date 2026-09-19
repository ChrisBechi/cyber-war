use super::*;
use crate::{search::VirtualSearchEngine, service::GameService};
use rusqlite::Connection;

fn world() -> WorldState {
    let mut world = WorldState::new("lia", "pc").unwrap();
    world.network.connected = true;
    world
}
fn page(world: &WorldState, url: &str) -> Page {
    resolve(world, url).unwrap().unwrap()
}
fn search(world: &WorldState, query: &str) -> crate::search::SearchResponse {
    VirtualSearchEngine::new(world)
        .search(query, "all", None, 0)
        .unwrap()
}

#[test]
fn every_brand_can_be_found_by_name_and_has_a_real_home() {
    let w = world();
    for brand in &repository().pack.brands {
        let found = search(&w, &brand.name);
        assert!(
            found.documents.iter().take(3).any(|result| repository()
                .document(&result.id)
                .is_some_and(|d| d.brand_id == brand.id)),
            "brand absent from top 3: {}",
            brand.name
        );
        assert_eq!(page(&w, &brand.domain).status, 200);
    }
}

#[test]
fn all_pack_documents_resolve_and_every_visible_link_stays_in_the_virtual_world() {
    let w = world();
    let repo = repository();
    assert!(repo.pack.brands.len() >= 13);
    assert!(repo.pack.documents.len() > 900);
    let mut seen = BTreeSet::new();
    for doc in &repo.pack.documents {
        let url = repo.url(doc);
        assert!(seen.insert(url.clone()), "duplicate {url}");
        let p = page(&w, &url);
        assert_eq!(p.status, if visible(&w, doc) { 200 } else { 404 }, "{url}");
        if !visible(&w, doc) {
            assert!(p.document.is_none());
            continue;
        }
        for link in &doc.links {
            assert_eq!(
                page(&w, &link.url).status,
                200,
                "{} -> {}",
                doc.id,
                link.url
            );
        }
        assert!(p.cards.len() <= PAGE_SIZE);
        assert!(p.related.len() <= 4);
    }
    for unsafe_url in [
        "https://real.example",
        "javascript:alert(1)",
        "file:///C:/Windows",
        "https://shopnow.com@evil.test",
        "https://shopnow.com:443",
    ] {
        assert!(!matches!(resolve(&w, unsafe_url), Ok(Some(_))));
    }
    assert!(resolve(&w, "https://educamais.com/search?q=%FF").is_err());
}

#[test]
fn common_queries_typos_intent_and_zero_results_are_coherent() {
    let w = world();
    for query in [
        "pato de borracha",
        "javascript",
        "placa de vídeo",
        "bolo de cenoura",
        "celular",
        "wifi",
        "filme de terror",
        "curso inglês",
        "carro usado",
        "inteligência artificial",
        "roteador",
        "cachorro",
        "receita pizza",
        "linux",
        "Nexora",
        "cachorro pode comer banana",
        "história do rádio",
        "hotel barato",
        "cadeira gamer",
    ] {
        let result = search(&w, query);
        assert!(result.total > 0, "{query}");
        for doc in &result.documents {
            if repository().document(&doc.id).is_some() {
                assert_eq!(page(&w, &doc.url).status, 200);
            }
        }
    }
    let corrected = search(&w, "pato de boraxa");
    assert_eq!(corrected.correction.as_deref(), Some("pato de borracha"));
    assert!(corrected.total > 0);
    assert_eq!(search(&w, "asdkjasdhqwe").total, 0);
    assert_eq!(
        search(&w, "placa de vídeo").documents[0].domain,
        "wipedia.org"
    );
    assert_eq!(
        search(&w, "vídeo placa de vídeo").documents[0].domain,
        "viewtube.com"
    );
    assert_eq!(
        search(&w, "comprar pato de borracha").documents[0].domain,
        "shopnow.com"
    );
    assert_eq!(
        search(&w, "curso javascript").documents[0].domain,
        "educamais.com"
    );
    assert_eq!(
        search(&w, "receita pizza").documents[0].domain,
        "cozinhafacil.com"
    );
    assert_eq!(
        search(&w, "Nexora").documents[0].url,
        "https://www.nexora.com/"
    );
    for (mode, expected) in [("shopping", "shopnow.com"), ("videos", "viewtube.com")] {
        let result = VirtualSearchEngine::new(&w)
            .search("pato de borracha", mode, None, 0)
            .unwrap();
        assert!(result.total > 0);
        assert!(result.documents.iter().all(|d| d.domain == expected));
    }
}

#[test]
fn deep_navigation_connects_product_seller_course_teacher_and_corporate_sources() {
    let w = world();
    let product = page(&w, "shopnow.com/produto/pato-de-borracha")
        .document
        .unwrap();
    if let Detail::Product { seller, .. } = product.detail {
        assert!(matches!(
            page(&w, &seller.url).document.unwrap().detail,
            Detail::Profile { .. }
        ));
    } else {
        panic!("product");
    }
    let course = page(&w, "educamais.com/cursos/javascript")
        .document
        .unwrap();
    if let Detail::Course {
        teacher, lessons, ..
    } = course.detail
    {
        assert_eq!(page(&w, &teacher.url).status, 200);
        assert_eq!(lessons.len(), 8);
        for lesson in lessons {
            assert!(matches!(
                page(&w, &lesson.url).document.unwrap().detail,
                Detail::Lesson { .. }
            ));
        }
    } else {
        panic!("course");
    }
    let corporate = page(&w, "nexora.com/produtos/nexphone-x2")
        .document
        .unwrap();
    assert!(corporate
        .links
        .iter()
        .any(|l| l.url.contains("techbyte.com")));
    assert!(corporate.links.iter().any(|l| l.url.contains("/docs/")));
    let redirect = page(&w, "nexora.tech/produtos/nexphone-x2");
    assert_eq!(
        redirect.canonical_url,
        "https://www.nexora.com/produtos/nexphone-x2"
    );
    for brand in &repository().pack.brands {
        let missing = page(&w, &format!("{}/missing", brand.domain));
        assert_eq!(missing.status, 404);
        assert_eq!(missing.brand.id, brand.id);
    }
}

#[test]
fn narrative_publication_and_search_cache_follow_flag_changes_and_rollback() {
    let mut w = world();
    let url = "b1.tech/noticias/revisao-de-acessos";
    assert_eq!(page(&w, url).status, 404);
    let before = search(&w, "revisão acessos");
    assert!(!before
        .documents
        .iter()
        .any(|d| d.url.ends_with("revisao-de-acessos")));
    w.flags.insert("SESSION_1_COMPLETE".into());
    sync_events(&mut w);
    assert!(w.web.events.contains("ORION_ACCESS_REVIEW"));
    assert_eq!(page(&w, url).status, 200);
    assert!(search(&w, "revisão acessos")
        .documents
        .iter()
        .any(|d| d.url.ends_with("revisao-de-acessos")));
    w.flags.remove("SESSION_1_COMPLETE");
    sync_events(&mut w);
    assert!(w.web.events.is_empty());
    assert_eq!(page(&w, url).status, 404);
    assert!(!search(&w, "revisão acessos")
        .documents
        .iter()
        .any(|d| d.url.ends_with("revisao-de-acessos")));
    for i in 0..100 {
        search(&w, &format!("nonsense{i}"));
    }
    assert!(w.search.cache.lock().unwrap().len() <= 64);
}

#[test]
fn mutations_and_bounded_history_are_saved_as_deltas_in_five_independent_slots() {
    let mut game = GameService::new(Connection::open_in_memory().unwrap()).unwrap();
    let id = "web-shopnow-produto-pato-de-borracha";
    for slot in 1..=5 {
        game.new_game(slot, &format!("user{slot}"), "pc", false)
            .unwrap();
        game.mutate(
            |_, w, _| {
                w.network.connected = true;
                for _ in 0..slot {
                    interact(w, id, "cart", "")?;
                }
                interact(
                    w,
                    "web-redditor-t-rubber-duck",
                    "comment",
                    &format!("Comentário do slot {slot}"),
                )?;
                interact(w, "web-educa-cursos-javascript-aula-1", "complete", "")?;
                crate::browser::navigate(w, "shopnow.com/produto/pato-de-borracha")?;
                Ok(())
            },
            false,
        )
        .unwrap();
        game.end_session().unwrap();
    }
    for slot in 1..=5 {
        game.load(slot, false).unwrap();
        let w = game.world().unwrap();
        assert_eq!(w.web.cart[id], slot as u32);
        assert_eq!(w.web.history.len(), 1);
        assert_eq!(w.web.comments.len(), 1);
        assert_eq!(w.web.completed_lessons.len(), 1);
        assert!(serde_json::to_string(&w.web).unwrap().len() < 4000);
        let p = page(w, "shopnow.com/produto/pato-de-borracha");
        assert_eq!(p.cart_count, slot as u32);
        assert_eq!(p.document.unwrap().title, "Pato de borracha clássico");
        game.end_session().unwrap();
    }
    let mut w = world();
    for n in 0..1000 {
        remember(&mut w, &format!("wipedia.org/?q={n}"), "Wipédia", "W");
    }
    assert_eq!(w.web.history.len(), HISTORY_LIMIT);
    let mut old = serde_json::to_value(&w).unwrap();
    old.as_object_mut().unwrap().remove("web");
    let old: WorldState = serde_json::from_value(old).unwrap();
    assert!(old.web.history.is_empty());
    assert_eq!(old.web.pack_versions["web-core"], repository().pack.version);
}

#[test]
fn interactions_validate_document_kind_visibility_stock_and_text() {
    let mut w = world();
    assert!(interact(&mut w, "web-wipedia-wiki-javascript", "cart", "").is_err());
    assert!(interact(&mut w, "web-b1-noticias-revisao-de-acessos", "like", "").is_err());
    assert!(interact(&mut w, "web-redditor-t-rubber-duck", "comment", "").is_err());
    assert!(interact(
        &mut w,
        "web-redditor-t-rubber-duck",
        "comment",
        &"x".repeat(1001)
    )
    .is_err());
    let id = "web-redditor-t-rubber-duck";
    interact(&mut w, id, "like", "").unwrap();
    assert!(w.web.likes.contains(id));
    interact(&mut w, id, "like", "").unwrap();
    assert!(!w.web.likes.contains(id));
    let product = repository()
        .pack
        .documents
        .iter()
        .find(|d| matches!(&d.detail,Detail::Product{stock,..}if stock=="OUT_OF_STOCK"))
        .unwrap();
    assert!(interact(&mut w, &product.id, "cart", "").is_err());
    w.network.connected = false;
    assert!(interact(&mut w, id, "like", "").is_err());
    assert_eq!(page(&w, "wipedia.org/wiki/linux").status, 200);
}

#[test]
fn measured_search_and_navigation_smoke() {
    let w = world();
    search(&w, "javascript");
    let mut search_ms = Vec::new();
    let mut page_ms = Vec::new();
    for i in 0..250 {
        let query = &[
            "pato de borracha",
            "curso javascript",
            "wifi",
            "cabo",
            "receita pizza",
            "asdkjasdhqwe",
        ][i % 6];
        let start = std::time::Instant::now();
        search(&w, query);
        search_ms.push(start.elapsed().as_micros());
        let start = std::time::Instant::now();
        page(&w, "shopnow.com/produto/nexphone-x2");
        page_ms.push(start.elapsed().as_micros());
    }
    search_ms.sort();
    page_ms.sort();
    println!(
        "search P50/P95/P99 µs: {}/{}/{}; page: {}/{}/{}",
        search_ms[125], search_ms[237], search_ms[247], page_ms[125], page_ms[237], page_ms[247]
    );
    assert!(search_ms[247] < 3_000_000);
    assert!(page_ms[247] < 1_000_000);
}

#[test]
#[ignore = "100,000 searches and 10,000 routes; run explicitly for content milestones"]
fn web_stress_and_coverage_report() {
    let mut w = world();
    let queries = [
        "pato de borracha",
        "javascript",
        "placa de vídeo",
        "bolo de cenoura",
        "celular",
        "wifi",
        "filme de terror",
        "curso inglês",
        "carro usado",
        "inteligência artificial",
        "roteador",
        "cachorro",
        "receita pizza",
        "linux",
        "Nexora",
        "pato de boraxa",
        "asdkjasdhqwe",
    ];
    let mut coverage = Vec::new();
    for query in queries {
        let result = search(&w, query);
        coverage.push(serde_json::json!({"query":query,"total":result.total,"top":result.documents.first().map(|d|&d.url),"correction":result.correction}));
    }
    let mut searches = Vec::new();
    let mut pages = Vec::new();
    for i in 0..100_000 {
        if i % 100 == 0 {
            w.playtime_seconds = ((i / 100) % 5 * 600) as u64;
            if !w.flags.remove("SESSION_1_COMPLETE") {
                w.flags.insert("SESSION_1_COMPLETE".into());
            }
            sync_events(&mut w);
        }
        let query = queries[i % queries.len()];
        let start = std::time::Instant::now();
        let result = search(&w, query);
        searches.push(start.elapsed().as_micros());
        let ids: BTreeSet<_> = result.documents.iter().map(|d| &d.id).collect();
        assert_eq!(ids.len(), result.documents.len());
        assert!(result
            .documents
            .iter()
            .all(|d| VirtualSearchEngine::new(&w).visible(d)));
        if i % 10 != 0 {
            continue;
        }
        let document = &repository().pack.documents[i % repository().pack.documents.len()];
        let url = repository().url(document);
        let start = std::time::Instant::now();
        let p = page(&w, &url);
        pages.push(start.elapsed().as_micros());
        assert_eq!(p.status, if visible(&w, document) { 200 } else { 404 });
        remember(&mut w, &url, &document.title, "");
    }
    searches.sort();
    pages.sort();
    let percentiles = |samples: &Vec<u128>| serde_json::json!({"p50Ms":samples[samples.len()/2] as f64/1000.,"p95Ms":samples[samples.len()*95/100] as f64/1000.,"p99Ms":samples[samples.len()*99/100] as f64/1000.});
    let report = serde_json::json!({"profile":"debug; current development machine; engine only, excluding IPC/SQLite/render","queries":coverage,"searchRuns":searches.len(),"pageRuns":pages.len(),"search":percentiles(&searches),"page":percentiles(&pages),"cacheEntries":w.search.cache.lock().unwrap().len(),"materializationCache":materialize::cache_len(),"historyEntries":w.web.history.len(),"deltaBytes":serde_json::to_vec(&w.web).unwrap().len()});
    let directory =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../artifacts/web-review");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(
        directory.join("coverage.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    println!("{report}");
    assert!(searches[searches.len() * 99 / 100] < 3_000_000);
    assert!(pages[9900] < 1_000_000);
    assert!(w.search.cache.lock().unwrap().len() <= 64);
    assert!(w.web.history.len() <= HISTORY_LIMIT);
}

#[test]
#[ignore = "writes visual QA fixtures into ignored artifacts; run explicitly"]
fn export_web_review() {
    let w = world();
    let directory =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../artifacts/web-review");
    std::fs::create_dir_all(&directory).unwrap();
    for brand in &repository().pack.brands {
        let p = page(&w, &brand.domain);
        std::fs::write(
            directory.join(format!("{}.json", brand.id)),
            serde_json::to_vec_pretty(&p).unwrap(),
        )
        .unwrap();
    }
    for (name, url) in [
        ("product", "shopnow.com/produto/pato-de-borracha"),
        ("recipe", "cozinhafacil.com/receitas/bolo-de-cenoura"),
        ("course", "educamais.com/cursos/javascript"),
        ("thread", "redditor.com/t/rubber-duck"),
        ("article", "techbyte.com/reviews/nexphone-x2"),
        ("404", "nexora.com/missing"),
        ("social", "linkup.com"),
        ("social-profile", "linkup.com/pessoas/nara-campos"),
        ("social-post", "linkup.com/publicacoes/nara-campos-caderno"),
        ("classifieds", "feiralivre.com"),
        ("classified", "feiralivre.com/anuncios/radio-aurora"),
    ] {
        std::fs::write(
            directory.join(format!("{name}.json")),
            serde_json::to_vec_pretty(&page(&w, url)).unwrap(),
        )
        .unwrap();
    }
    let mut timeline = world();
    timeline.playtime_seconds = 1800;
    sync_events(&mut timeline);
    for (name, url) in [
        ("archive", "memoria.web"),
        (
            "capture",
            "memoria.web/snapshot/web-shopnow-ofertas-nexphone-x2",
        ),
    ] {
        std::fs::write(
            directory.join(format!("{name}.json")),
            serde_json::to_vec_pretty(&page(&timeline, url)).unwrap(),
        )
        .unwrap();
    }
    timeline.playtime_seconds = 0;
    sync_events(&mut timeline);
    interact(&mut timeline, "web-shopnow-produto-nexphone-x2", "cart", "").unwrap();
    for (name, second) in [
        ("offer-base", 0),
        ("offer-live", 600),
        ("offer-low", 1200),
        ("offer-empty", 1800),
        ("offer-restock", 2400),
    ] {
        timeline.playtime_seconds = second;
        sync_events(&mut timeline);
        std::fs::write(
            directory.join(format!("{name}.json")),
            serde_json::to_vec_pretty(&page(&timeline, "shopnow.com/produto/nexphone-x2")).unwrap(),
        )
        .unwrap();
    }
}
