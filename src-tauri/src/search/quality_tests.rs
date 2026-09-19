use super::*;
use serde::Deserialize;
#[derive(Deserialize)]
struct Query {
    query: String,
    entity: Option<String>,
    #[serde(default)]
    empty: bool,
}
#[test]
fn relevance_corpus_generates_auditable_metrics_without_hiding_misses() {
    let corpus: Vec<Query> =
        serde_json::from_str(include_str!("../../../content/search/quality-corpus.json")).unwrap();
    let mut world = WorldState::new("lia", "pc").unwrap();
    world.network.connected = true;
    let mut rows = Vec::new();
    let mut misses = 0;
    for q in corpus {
        if let Some(id) = &q.entity {
            assert!(
                crate::virtual_web::repository()
                    .pack
                    .entities
                    .iter()
                    .any(|e| &e.id == id),
                "invalid judgment entity {id}"
            );
        }
        let result = VirtualSearchEngine::new(&world)
            .search(&q.query, "all", None, 0)
            .unwrap();
        let relevant: Vec<_> = result
            .documents
            .iter()
            .map(|d| {
                q.entity.as_ref().is_some_and(|entity| {
                    crate::virtual_web::repository()
                        .document(&d.id)
                        .is_some_and(|d| d.entities.contains(entity))
                })
            })
            .collect();
        let precision =
            |k: usize| relevant.iter().take(k).filter(|r| **r).count() as f64 / k as f64;
        let success = if q.empty {
            result.total == 0
        } else {
            relevant.iter().take(10).any(|r| *r)
        };
        if !success {
            misses += 1;
        }
        rows.push(serde_json::json!({"query":q.query,"expectedEntity":q.entity,"expectedEmpty":q.empty,"total":result.total,"precisionAt1":precision(1),"precisionAt3":precision(3),"precisionAt10":precision(10),"success":success,"top":result.documents.iter().take(10).map(|d|&d.url).collect::<Vec<_>>()}));
    }
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../artifacts/web-audit");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(directory.join("quality.json"),serde_json::to_vec_pretty(&serde_json::json!({"judgments":"authored entity-level seed corpus; not a 10k human-reviewed corpus","queries":rows.len(),"misses":misses,"results":rows})).unwrap()).unwrap();
    assert_eq!(misses, 0, "Inspect artifacts/web-audit/quality.json");
}
#[test]
#[ignore = "10,000 distinct generated adversarial inputs, separate from relevance judgments"]
fn distinct_query_fuzz_never_panics_leaks_or_returns_duplicate_urls() {
    let mut world = WorldState::new("lia", "pc").unwrap();
    world.network.connected = true;
    let mut unique = BTreeSet::new();
    let mut latencies = Vec::new();
    for i in 0..10_000 {
        let query = match i % 5 {
            0 => format!("inexistente{i} /\u{202e} <script>"),
            1 => format!("javascript placa de video {i}"),
            2 => format!("%@{} rua inexistente", i),
            3 => format!("pato de borra{}", i),
            _ => format!("\"nexora\" orçamento {i}"),
        };
        assert!(unique.insert(query.clone()));
        let at = std::time::Instant::now();
        let result = VirtualSearchEngine::new(&world)
            .search(&query, "all", None, 0)
            .unwrap();
        latencies.push(at.elapsed().as_micros());
        let mut urls = BTreeSet::new();
        for d in result.documents {
            assert!(urls.insert(d.url.clone()));
            assert!(VirtualSearchEngine::new(&world).visible(&d));
        }
    }
    latencies.sort();
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../artifacts/web-audit");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("fuzz.json"),serde_json::to_vec_pretty(&serde_json::json!({"uniqueInputs":unique.len(),"purpose":"robustness, not relevance","p50Micros":latencies[5000],"p95Micros":latencies[9500],"p99Micros":latencies[9900]})).unwrap()).unwrap();
}
