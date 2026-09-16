//! Opt-in native QA fixture. Compiled only in debug builds, with an in-memory database.
use super::*;
use crate::service::GameService;

pub fn game() -> GameResult<GameService> {
    let mut game = GameService::new(rusqlite::Connection::open_in_memory()?)?;
    game.new_game(1, "archive-qa", "kali", false)?;
    game.end_session()?;
    game.start_session()?;
    game.mutate(
        |_, world, _| {
            world.network.connected = true;
            crate::packages::ipc::download(
                world,
                "https://mirror.kali.game/kali/pool/wireless-utils_1.0_amd64.deb",
                "/home/kali/Downloads/wireless-utils.deb",
                "kali",
            )?;
            for (url, name) in [
                (downloads::FIRMWARE_URL, "firmware.tar.gz"),
                (downloads::SOURCE_URL, "tool-2.1.tar.gz"),
            ] {
                downloads::download(world, url, &format!("/home/kali/Downloads/{name}"), "kali")?;
            }
            world.vfs.mkdir("/home/kali/qa", "kali")?;
            world.vfs.write(
                "/home/kali/qa/notes.txt",
                "Archive QA\nadmin connected\n",
                "kali",
            )?;
            world
                .vfs
                .write("/home/kali/qa/.hidden", "Hidden entry\n", "kali")?;
            ArchiveService::default().create_archive(
                world,
                "/home/kali/Downloads/private.zip",
                &["qa".into()],
                ArchiveFormat::Zip,
                true,
                Some("qa-secret"),
                "kali",
            )?;
            world.vfs.write(
                "/home/kali/qa/database.sql",
                "CREATE TABLE logs(id INTEGER);\n",
                "kali",
            )?;
            world
                .vfs
                .nodes
                .get_mut("/home/kali/qa/database.sql")
                .unwrap()
                .metadata
                .insert("logicalSize".into(), (1024u64 * 1024 * 1024).to_string());
            world
                .vfs
                .write("/home/kali/Downloads/broken.zip", "invalid archive", "kali")?;
            world.notify("Nexora", "Pacote de documentos disponível.");
            world
                .messages
                .last_mut()
                .unwrap()
                .attachments
                .push(downloads::ATTACHMENT_URL.into());
            Ok(())
        },
        false,
    )?;
    Ok(game)
}
