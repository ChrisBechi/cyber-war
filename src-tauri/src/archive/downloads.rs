//! One virtual download catalogue for browser, terminal, repository packages and attachments.
use super::*;
use crate::{
    vfs::{normalize, HOME},
    world::WorldState,
};

pub const FIRMWARE_URL: &str = "https://nexora.support/firmware_AXR550_v1.4.tar.gz";
pub const SOURCE_URL: &str = "https://repository.kali.local/tool-2.1.tar.gz";
pub const ATTACHMENT_URL: &str = "https://mail.goggle.local/attachments/documentos.zip";

type Package = (ArchiveFormat, Vec<(&'static str, &'static str, u16)>);
fn package(url: &str) -> Option<Package> {
    match url {
        FIRMWARE_URL => Some((
            ArchiveFormat::TarGzip,
            vec![
                (
                    "firmware/README",
                    "Nexora AXR550 firmware 1.4\nVirtual device package.\n",
                    0o644,
                ),
                (
                    "firmware/bin/update",
                    "#!/bin/sh\necho 'Virtual firmware update ready'\n",
                    0o755,
                ),
                (
                    "firmware/config/default.conf",
                    "model=AXR550\nversion=1.4\n",
                    0o644,
                ),
            ],
        )),
        SOURCE_URL => Some((
            ArchiveFormat::TarGzip,
            vec![
                (
                    "tool-2.1/README",
                    "CYBER WAR virtual source package 2.1\n",
                    0o644,
                ),
                (
                    "tool-2.1/LICENSE",
                    "This fictional example is provided for in-game use.\n",
                    0o644,
                ),
                ("tool-2.1/bin/tool", "#!/bin/sh\necho 'Tool 2.1'\n", 0o755),
                ("tool-2.1/config/tool.conf", "mode=virtual\n", 0o644),
            ],
        )),
        ATTACHMENT_URL => Some((
            ArchiveFormat::Zip,
            vec![
                (
                    "documentos/notes.txt",
                    "Documentos recebidos. Confira a versão do firmware no suporte Nexora.\n",
                    0o644,
                ),
                ("documentos/.reference", "AXR550-1.4\n", 0o644),
            ],
        )),
        _ => None,
    }
}
pub fn available(url: &str) -> bool {
    if crate::packages::ipc::download_entry(url).is_some() {
        return true;
    }
    package(url).is_some()
}
pub fn download(
    world: &mut WorldState,
    url: &str,
    destination: &str,
    actor: &str,
) -> GameResult<String> {
    if !world.network.connected {
        return Err(domain("network unreachable"));
    }
    if crate::packages::ipc::download_entry(url).is_some() {
        return crate::packages::ipc::download(world, url, destination, actor);
    }
    let (format, files) = package(url).ok_or_else(|| domain("virtual download not found"))?;
    let mut entries = std::collections::BTreeMap::new();
    for (path, text, mode) in files {
        let mut parent = crate::vfs::parent(path);
        while parent != "/" {
            entries
                .entry(parent.to_string())
                .or_insert_with(|| entry(parent, "directory", "", 0o755));
            parent = crate::vfs::parent(parent);
        }
        entries.insert(path.into(), entry(path, "file", text, mode));
    }
    let data = codec::encode(
        &entries.into_values().collect::<Vec<_>>(),
        format,
        None,
        &ArchiveSafetyLimits::default(),
    )?;
    let path = normalize(destination, &world.terminal.cwd)?;
    let path = world.fs()?.available_path(&path, false);
    let len = data.len() as u64;
    write_bytes(world, &path, data, actor, len)?;
    Ok(path)
}
fn entry(path: &str, kind: &str, text: &str, mode: u16) -> StoredEntry {
    StoredEntry {
        info: ArchiveEntry {
            path: path.into(),
            kind: kind.into(),
            original_size: text.len() as u64,
            compressed_size: None,
            permissions: mode,
            modified_at: 1,
            owner: Some("kali".into()),
            group: Some("kali".into()),
            crc: None,
            link_target: None,
            encrypted: false,
        },
        data: text.as_bytes().to_vec(),
        metadata: None,
    }
}
pub fn attachment(world: &mut WorldState, message_id: &str, index: usize) -> GameResult<String> {
    let url = world
        .messages
        .iter()
        .find(|m| m.id == message_id)
        .and_then(|m| m.attachments.get(index))
        .ok_or_else(|| domain("attachment not found"))?
        .clone();
    let name = url
        .rsplit('/')
        .next()
        .ok_or_else(|| domain("invalid attachment"))?;
    let path = format!("{HOME}/Downloads/{name}");
    let saved = std::mem::replace(
        &mut world.terminal,
        crate::terminal_sessions::fresh_session(),
    );
    let result = download(world, &url, &path, "kali");
    world.terminal = saved;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn repeated_downloads_and_attachments_return_the_actual_numbered_path() {
        let mut world = WorldState::new("neo", "pc").unwrap();
        world.network.connected = true;
        let first = download(
            &mut world,
            FIRMWARE_URL,
            "/home/kali/Downloads/firmware.tar.gz",
            "kali",
        )
        .unwrap();
        let original = world.vfs.nodes[&first].blob.clone();
        let second = download(&mut world, FIRMWARE_URL, &first, "kali").unwrap();
        assert_eq!(second, "/home/kali/Downloads/firmware (1).tar.gz");
        assert_eq!(world.vfs.nodes[&first].blob, original);
        world.notify("Nexora", "attachment");
        let message = world.messages.last_mut().unwrap();
        message.attachments.push(ATTACHMENT_URL.into());
        let id = message.id.clone();
        assert_eq!(
            attachment(&mut world, &id, 0).unwrap(),
            "/home/kali/Downloads/documentos.zip"
        );
        assert_eq!(
            attachment(&mut world, &id, 0).unwrap(),
            "/home/kali/Downloads/documentos (1).zip"
        );
    }
}
