//! Desktop actions only mutate the virtual filesystem. No native shell/URL execution.
use crate::{error::GameResult, software, vfs::domain, world::WorldState};
use serde::Deserialize;

pub const DIRECTORY: &str = "/home/kali/Desktop";

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ItemKind {
    Folder,
    File,
    Shell,
    Launcher,
    Url,
}

pub fn create(world: &mut WorldState, kind: ItemKind, name: &str, target: &str) -> GameResult<()> {
    if name.chars().any(char::is_control) {
        return Err(domain("Nome contém caracteres de controle."));
    }
    let name = name.trim();
    if name.is_empty()
        || name.len() > 200
        || name == "."
        || name == ".."
        || name
            .chars()
            .any(|c| c.is_control() || c == '/' || c == '\\')
    {
        return Err(domain(
            "Nome inválido: informe apenas o nome, sem barras ou controles.",
        ));
    }
    let suffix = match kind {
        ItemKind::Launcher | ItemKind::Url => ".desktop",
        ItemKind::Shell => ".sh",
        _ => "",
    };
    let path = format!(
        "{DIRECTORY}/{name}{}",
        if name.ends_with(suffix) { "" } else { suffix }
    );
    if world.vfs.nodes.contains_key(&path) {
        return Err(domain("EEXIST: já existe um item com esse nome."));
    }
    let mut metadata = std::collections::BTreeMap::new();
    let content = match kind {
        ItemKind::Folder => return world.vfs.mkdir(&path, "kali"),
        ItemKind::File => String::new(),
        ItemKind::Shell => "#!/bin/bash\n\n".into(),
        ItemKind::Launcher => {
            let app = software::get(target)?;
            metadata.insert("launcherId".into(), app.id.clone());
            format!(
                "[Desktop Entry]\nType=Application\nName={name}\nX-CyberWar-Application={}\n",
                app.id
            )
        }
        ItemKind::Url => {
            let url = target.trim();
            if url.is_empty()
                || url.len() > 2048
                || url.chars().any(char::is_whitespace)
                || (url.contains(':')
                    && !url.starts_with("https://")
                    && !url.starts_with("http://"))
            {
                return Err(domain(
                    "Use um endereço HTTP/HTTPS ou um endereço da internet virtual.",
                ));
            }
            metadata.insert("url".into(), url.into());
            format!("[Desktop Entry]\nType=Link\nName={name}\nURL={url}\n")
        }
    };
    world.vfs.write(&path, &content, "kali")?;
    let node = world
        .vfs
        .nodes
        .get_mut(&path)
        .ok_or_else(|| domain("Item não criado"))?;
    node.metadata.extend(metadata);
    if matches!(kind, ItemKind::Shell) {
        node.mode = 0o755;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vfs::VirtualFileSystem;
    #[test]
    fn desktop_items_and_sort_survive_clean_logout_and_reload() {
        let mut game =
            crate::service::GameService::new(rusqlite::Connection::open_in_memory().unwrap())
                .unwrap();
        game.new_game(1, "kali", "lifeos", false).unwrap();
        game.mutate(
            |_, world, _| {
                create(world, ItemKind::Folder, "Pessoal", "")?;
                create(world, ItemKind::File, "anotações.txt", "")?;
                create(world, ItemKind::Launcher, "Minha ferramenta", "kali-nmap")?;
                create(world, ItemKind::Url, "Biblioteca", "wipedia.org")?;
                world.settings.insert("desktopSort".into(), "name".into());
                Ok(())
            },
            false,
        )
        .unwrap();
        game.end_session().unwrap();
        game.load(1, false).unwrap();
        let world = game.world().unwrap();
        for name in [
            "Pessoal",
            "anotações.txt",
            "Minha ferramenta.desktop",
            "Biblioteca.desktop",
        ] {
            assert!(world.vfs.nodes.contains_key(&format!("{DIRECTORY}/{name}")));
        }
        assert_eq!(world.settings["desktopSort"], "name");
        assert_eq!(
            world.vfs.nodes[&format!("{DIRECTORY}/Biblioteca.desktop")].metadata["url"],
            "wipedia.org"
        );
        assert!(world.terminal_sessions.is_empty());
    }
    #[test]
    fn desktop_items_are_virtual_and_cannot_overwrite_or_escape() {
        let mut world = WorldState::new("kali", "lifeos").unwrap();
        world.vfs = VirtualFileSystem::default();
        create(&mut world, ItemKind::Folder, "Projetos", "").unwrap();
        create(&mut world, ItemKind::Shell, "teste", "").unwrap();
        assert_eq!(
            world
                .vfs
                .read(&format!("{DIRECTORY}/teste.sh"), "kali")
                .unwrap(),
            "#!/bin/bash\n\n"
        );
        assert_eq!(
            world.vfs.nodes[&format!("{DIRECTORY}/teste.sh")].mode,
            0o755
        );
        create(&mut world, ItemKind::Launcher, "Meu terminal", "terminal").unwrap();
        assert_eq!(
            world.vfs.nodes[&format!("{DIRECTORY}/Meu terminal.desktop")].metadata["launcherId"],
            "terminal"
        );
        create(&mut world, ItemKind::Url, "Biblioteca", "wipedia.org").unwrap();
        for name in ["../fora", "a/b", "a\\b", ".", "", "ruim\n"] {
            assert!(
                create(&mut world, ItemKind::File, name, "").is_err(),
                "{name:?}"
            );
        }
        assert!(create(&mut world, ItemKind::File, "teste.sh", "").is_err());
        assert!(create(
            &mut world,
            ItemKind::Launcher,
            "invalido",
            "arbitrary-command"
        )
        .is_err());
        for url in [
            "javascript:alert(1)",
            "file:///C:/Windows",
            "data:text/html,x",
            "a\nURL=b",
        ] {
            assert!(create(&mut world, ItemKind::Url, "invalido", url).is_err());
        }
    }
}
