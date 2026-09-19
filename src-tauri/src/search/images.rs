use super::VirtualSearchEngine;
use crate::{error::GameResult, vfs::domain, world::WorldState};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, sync::OnceLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageRecord {
    pub id: String,
    pub virtual_url: String,
    pub asset: String,
    pub hashes: Vec<String>,
    pub document_ids: Vec<String>,
    pub entities: Vec<String>,
}
pub fn catalog() -> &'static Vec<ImageRecord> {
    static IMAGES: OnceLock<Vec<ImageRecord>> = OnceLock::new();
    IMAGES.get_or_init(|| {
        let mut images = vec![
            ImageRecord {
                id: "orion-campus".into(),
                virtual_url: "https://www.orion.com/media/campus.svg".into(),
                asset: "orion-campus".into(),
                hashes: vec![format!(
                    "{:x}",
                    Sha256::digest(include_bytes!(
                        "../../../public/assets/goggle/orion-campus.svg"
                    ))
                )],
                document_ids: vec!["orion-image".into(), "orion".into(), "orion-campus".into()],
                entities: vec!["company:orion".into(), "place:orion-campus".into()],
            },
            ImageRecord {
                id: "archive-records".into(),
                virtual_url: "https://www.archive.org/media/archive.svg".into(),
                asset: "archive-records".into(),
                hashes: vec![format!(
                    "{:x}",
                    Sha256::digest(include_bytes!(
                        "../../../public/assets/goggle/archive-records.svg"
                    ))
                )],
                document_ids: vec!["archive-image".into(), "archive".into()],
                entities: vec!["collection:archive".into()],
            },
        ];
        let repo = crate::virtual_web::repository();
        let visuals: BTreeSet<_> = repo
            .pack
            .documents
            .iter()
            .filter(|d| d.path != "/")
            .map(|d| &d.visual)
            .collect();
        for visual in visuals {
            let id = format!("web-{visual}");
            images.push(ImageRecord {
                id: id.clone(),
                virtual_url: format!("https://www.goggle.com/media/{id}.svg"),
                asset: id,
                hashes: Vec::new(),
                document_ids: repo
                    .pack
                    .documents
                    .iter()
                    .filter(|d| &d.visual == visual && d.path != "/")
                    .map(|d| d.id.clone())
                    .collect(),
                entities: repo
                    .pack
                    .documents
                    .iter()
                    .filter(|d| &d.visual == visual)
                    .flat_map(|d| d.entities.clone())
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect(),
            });
        }
        images
    })
}
pub fn records(world: &WorldState) -> impl Iterator<Item = &ImageRecord> {
    catalog()
        .iter()
        .filter(|i| !world.search.images.contains_key(&i.id))
        .chain(world.search.images.values())
}
pub fn reverse(engine: &VirtualSearchEngine<'_>, source: &str) -> GameResult<Vec<String>> {
    let hash = if source.starts_with('/') && !source.starts_with("//") {
        if source.contains('\\') || source.contains(':') || source.split('/').any(|c| c == "..") {
            return Err(domain("Selecione um arquivo do VFS."));
        }
        let node = engine.world.vfs.readable(source, "kali")?;
        if node.kind != "file" {
            return Err(domain("Selecione uma imagem do VFS."));
        }
        match &node.blob {
            Some(blob) if blob.mime.starts_with("image/") => Some(blob.hash.clone()),
            None if source.ends_with(".svg") && node.content.trim_start().starts_with("<svg") => {
                Some(format!("{:x}", Sha256::digest(node.content.as_bytes())))
            }
            _ => return Err(domain("Este arquivo do VFS não é uma imagem.")),
        }
    } else {
        super::virtual_url_key(source)?;
        None
    };
    let key = if hash.is_none() {
        Some(super::virtual_url_key(source)?)
    } else {
        None
    };
    let matched: Vec<_> = records(engine.world)
        .filter(|image| {
            hash.as_ref().is_some_and(|h| image.hashes.contains(h))
                || key.as_ref().is_some_and(|k| {
                    super::virtual_url_key(&image.virtual_url).ok().as_ref() == Some(k)
                })
        })
        .collect();
    if hash.is_none() && matched.is_empty() {
        return Err(domain("URL de imagem não registrada na internet virtual."));
    }
    Ok(matched
        .iter()
        .flat_map(|i| i.document_ids.iter().cloned())
        .collect())
}
pub fn files(world: &WorldState) -> Vec<String> {
    world
        .vfs
        .nodes
        .iter()
        .filter(|(path, node)| {
            !crate::vfs::VirtualFileSystem::is_trash_path(path)
                && node.kind == "file"
                && (node
                    .blob
                    .as_ref()
                    .is_some_and(|b| b.mime.starts_with("image/"))
                    || (path.ends_with(".svg") && node.content.trim_start().starts_with("<svg")))
                && world.vfs.readable(path, "kali").is_ok()
        })
        .map(|(path, _)| path.clone())
        .collect()
}
pub fn for_documents(
    engine: &VirtualSearchEngine<'_>,
    documents: &[super::SearchDocument],
) -> Vec<ImageRecord> {
    let ids: BTreeSet<_> = documents
        .iter()
        .filter_map(|d| d.image_id.as_ref())
        .collect();
    records(engine.world)
        .filter(|image| ids.contains(&image.id))
        .cloned()
        .map(|mut image| {
            image
                .document_ids
                .retain(|id| documents.iter().any(|d| &d.id == id));
            if image.id.starts_with("web-") {
                image.entities = image
                    .document_ids
                    .iter()
                    .filter_map(|id| crate::virtual_web::repository().document(id))
                    .flat_map(|d| d.entities.clone())
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect();
            }
            image
        })
        .collect()
}
