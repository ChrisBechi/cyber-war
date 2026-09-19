//! Only compiled into debug builds. Read-only inspection never changes visibility or saves.
use super::{repository, visible};
use crate::{
    error::GameResult,
    search::{index, VirtualSearchEngine},
    world::WorldState,
};
use serde_json::{json, Value};
pub fn inspect(world: &WorldState, query: &str, address: &str) -> GameResult<Value> {
    if query.chars().count() > 200 || address.len() > 512 {
        return Err(crate::vfs::domain("Consulta de diagnóstico inválida."));
    }
    let repo = repository();
    let started = std::time::Instant::now();
    let page = super::resolve(world, address)?;
    let elapsed = started.elapsed().as_micros();
    let cache_before = world
        .search
        .cache
        .lock()
        .ok()
        .map(|cache| cache.diagnostics());
    let search = VirtualSearchEngine::new(world).search(query, "all", None, 0)?;
    let cache_after = world
        .search
        .cache
        .lock()
        .ok()
        .map(|cache| cache.diagnostics());
    let scores:Vec<_>=search.documents.iter().filter_map(|d|index::base().documents.get(&d.id).map(|base|json!({"id":d.id,"title":d.title,"url":d.url,"components":crate::search::explain(world,base,query)}))).collect();
    let query = crate::search::documents::normalize(query);
    let matches:Vec<_>=repo.pack.documents.iter().filter(|d|query.is_empty()||crate::search::documents::normalize(&format!("{} {}",d.id,d.title)).contains(&query)).take(100).map(|d|json!({"id":d.id,"title":d.title,"url":repo.url(d),"visible":visible(world,d),"source":d.materializer.as_deref().unwrap_or("authored"),"entities":d.entities})).collect();
    Ok(
        json!({"currentUrl":address,"pageStatus":page.as_ref().map(|p|p.status),"documentId":page.as_ref().and_then(|p|p.document.as_ref().map(|d|&d.id)),"seed":world.web.seed,"pack":repo.pack.version,"worldSeconds":world.playtime_seconds,"resolveMicros":elapsed,"materializationCache":super::materialize::cache_len(),"cacheBefore":cache_before,"cacheAfter":cache_after,"brands":repo.pack.brands,"entities":repo.pack.entities,"events":repo.pack.events,"activeEvents":world.web.events,"documents":matches,"results":scores,"totalResults":search.total}),
    )
}
