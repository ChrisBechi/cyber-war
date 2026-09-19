//! Synthetic scale fixture: measures index cost, never pretends these are authored world pages.
use super::{documents::SearchDocument, index::SearchIndex};
#[test]
#[ignore = "explicit synthetic scale audit: 250k indexed documents and 100k distinct lookups"]
fn synthetic_index_scale_and_fuzz_report() {
    let base = super::index::base()
        .documents
        .values()
        .next()
        .unwrap()
        .clone();
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../artifacts/web-audit");
    std::fs::create_dir_all(&directory).unwrap();
    let mut rows = Vec::new();
    for count in [10_000, 50_000, 100_000, 250_000] {
        let docs: Vec<SearchDocument> = (0..count)
            .map(|i| {
                let mut d = base.clone();
                d.id = format!("fixture-{i}");
                d.url = format!("https://www.fixture.test/page/{i}");
                d.domain = "fixture.test".into();
                d.title = format!("Registro de bancada {i}");
                d.content = format!("ensaio grupo{} serial{i}", i % 1000);
                d.description = format!("Amostra sintética {i} do grupo {}", i % 1000);
                d.keywords = vec![format!("serial{i}"), format!("grupo{}", i % 1000)];
                d.suggestions.clear();
                d.image_id = None;
                d
            })
            .collect();
        let bytes = serde_json::to_vec(&docs).unwrap().len();
        let started = std::time::Instant::now();
        let index = SearchIndex::new(docs);
        let build_ms = started.elapsed().as_millis();
        let mut micros = Vec::with_capacity(25_000);
        for i in 0..25_000 {
            let expected = i % count;
            let query = format!("serial{expected}");
            let at = std::time::Instant::now();
            let hits = index.candidates(&query);
            micros.push(at.elapsed().as_micros());
            assert!(hits.contains(&format!("fixture-{expected}")));
            assert_eq!(hits.len(), 1);
            assert!(index.candidates(&format!("inexistente{i}")).is_empty());
        }
        micros.sort();
        rows.push(serde_json::json!({"documents":count,"serializedBytes":bytes,"indexBuildMs":build_ms,"lookups":25000,"negativeQueries":25000,"p50Micros":micros[12500],"p95Micros":micros[23750],"p99Micros":micros[24750]}));
        std::fs::write(directory.join("scale.json"),serde_json::to_vec_pretty(&serde_json::json!({"fixture":"synthetic; index-only; excludes IPC, SQLite and rendering","profiles":"current development host only","runs":rows})).unwrap()).unwrap();
    }
}
#[test]
#[ignore = "writes and measures a disposable synthetic content file up to 500k records"]
fn storage_size_growth_to_half_million_documents() {
    use std::io::{BufWriter, Write};
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../artifacts/web-audit");
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join(format!("storage-{}.tmp", uuid::Uuid::new_v4()));
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .unwrap();
    let mut writer = BufWriter::new(file);
    let mut rows = Vec::new();
    let started = std::time::Instant::now();
    for i in 1..=500_000 {
        serde_json::to_writer(&mut writer,&serde_json::json!({"id":format!("synthetic-{i}"),"title":format!("Registro de armazenamento {i}"),"body":"Texto controlado de bancada para medir o crescimento físico do pacote local.","url":format!("https://www.fixture.test/{i}")})).unwrap();
        writer.write_all(b"\n").unwrap();
        if [10_000, 50_000, 100_000, 250_000, 500_000].contains(&i) {
            writer.flush().unwrap();
            rows.push(serde_json::json!({"documents":i,"fileBytes":std::fs::metadata(&path).unwrap().len(),"elapsedMs":started.elapsed().as_millis()}));
        }
    }
    drop(writer);
    std::fs::remove_file(&path).unwrap();
    std::fs::write(directory.join("storage.json"),serde_json::to_vec_pretty(&serde_json::json!({"format":"synthetic JSONL content pack; static content is not stored in save SQLite","runs":rows})).unwrap()).unwrap();
}
