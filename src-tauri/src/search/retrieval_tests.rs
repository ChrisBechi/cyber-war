use super::*;
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Case {
    query: String,
    expected_ids: Vec<String>,
}
#[test]
#[ignore = "run node scripts/build-web-query-corpus.mjs first; 10k mechanically judged retrieval cases"]
fn corpus_retrieval_report_records_precision_recall_latency_and_misses() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../artifacts/web-audit");
    let cases: Vec<Case> = serde_json::from_slice(
        &std::fs::read(dir.join("query-corpus.json"))
            .expect("Generate corpus before this explicit audit"),
    )
    .unwrap();
    let mut w = WorldState::new("lia", "pc").unwrap();
    w.network.connected = true;
    let mut totals = [0.; 3];
    let mut empty = 0;
    let mut hits = 0;
    let mut missing = Vec::new();
    let mut latency = Vec::new();
    let mut unique = BTreeSet::new();
    for case in &cases {
        assert!(unique.insert(&case.query));
        let at = std::time::Instant::now();
        let response = VirtualSearchEngine::new(&w)
            .search(&case.query, "all", None, 0)
            .unwrap();
        latency.push(at.elapsed().as_micros());
        let mut urls = BTreeSet::new();
        for d in &response.documents {
            assert!(urls.insert(&d.url));
            assert!(VirtualSearchEngine::new(&w).visible(d));
        }
        let relevant = |d: &&SearchDocument| case.expected_ids.contains(&d.id);
        for (i, k) in [1, 3, 10].iter().enumerate() {
            totals[i] +=
                response.documents.iter().take(*k).filter(relevant).count() as f64 / (*k as f64);
        }
        if response.total == 0 {
            empty += 1;
        }
        if response
            .documents
            .iter()
            .any(|d| case.expected_ids.contains(&d.id))
        {
            hits += 1;
        } else if missing.len() < 100 {
            missing.push(serde_json::json!({"query":case.query,"expectedIds":case.expected_ids,"top":response.documents.first().map(|d|&d.id)}));
        }
    }
    latency.sort();
    let n = cases.len();
    assert!(n >= 10000);
    let report = serde_json::json!({"queries":n,"judgments":"quoted passages; expected source documents; incomplete judgments undercount other relevant results","precisionAt1":totals[0]/n as f64,"precisionAt3":totals[1]/n as f64,"precisionAt10":totals[2]/n as f64,"sourceHitAt20":hits as f64/n as f64,"zeroResults":empty,"p50Micros":latency[n/2],"p95Micros":latency[n*95/100],"p99Micros":latency[n*99/100],"sampleMisses":missing});
    std::fs::write(
        dir.join("retrieval.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    assert!(latency[n * 99 / 100] < 3_000_000);
}
