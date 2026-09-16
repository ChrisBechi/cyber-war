//! Official default-edition launcher metadata and virtual inspection workspaces.
//! No package installation, shell invocation, DNS lookup or real network fallback.
use crate::{
    error::GameResult,
    vfs::{domain, normalize, HOME},
    world::WorldState,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::sync::LazyLock;

#[derive(Debug, Deserialize)]
pub struct Software {
    pub id: String,
    pub name: String,
    pub description: String,
    pub package: String,
    pub commands: Vec<String>,
    pub source: String,
    pub operation: String,
    pub resource: Option<String>,
}
#[derive(Deserialize)]
pub struct Catalog {
    pub entries: Vec<Software>,
    pub favorites: Vec<String>,
}
pub static CATALOG: LazyLock<Catalog> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../content/software/kali-default.json"))
        .expect("validated Kali catalog")
});

static IDS: LazyLock<BTreeMap<&'static str, usize>> = LazyLock::new(|| {
    CATALOG
        .entries
        .iter()
        .enumerate()
        .map(|(i, e)| (e.id.as_str(), i))
        .collect()
});
static COMMAND_INDEX: LazyLock<BTreeMap<&'static str, usize>> = LazyLock::new(|| {
    let mut index = BTreeMap::new();
    for (i, entry) in CATALOG.entries.iter().enumerate() {
        for name in std::iter::once(&entry.id)
            .chain(std::iter::once(&entry.name))
            .chain(entry.commands.iter())
        {
            // Preserve the existing first-entry precedence. Tooling validates
            // every overlap against its explicit collision resolution policy.
            index.entry(name.as_str()).or_insert(i);
        }
    }
    index
});
pub fn get(id: &str) -> GameResult<&'static Software> {
    IDS.get(id)
        .map(|i| &CATALOG.entries[*i])
        .ok_or_else(|| domain("Aplicativo fora do conjunto padrão do Kali"))
}

pub fn by_command(command: &str) -> Option<&'static Software> {
    COMMAND_INDEX.get(command).map(|i| &CATALOG.entries[*i])
}

fn list(world: &WorldState, key: &str, fallback: &[String]) -> Vec<String> {
    world
        .settings
        .get(key)
        .and_then(|value| serde_json::from_str::<Vec<String>>(value).ok())
        .unwrap_or_else(|| fallback.to_vec())
}

pub fn open(world: &mut WorldState, id: &str) -> GameResult<()> {
    let tool = get(id)?;
    available(world, tool)?;
    let mut recent = list(world, "launcherRecent", &[]);
    recent.retain(|item| item != id && get(item).is_ok());
    recent.insert(0, id.into());
    recent.truncate(30);
    world
        .settings
        .insert("launcherRecent".into(), serde_json::to_string(&recent)?);
    // Privilege and cwd belong to the terminal instance created by terminal_open,
    // not to the shared fallback context or another open terminal.
    Ok(())
}

pub fn favorite(world: &mut WorldState, id: &str) -> GameResult<()> {
    get(id)?;
    let mut favorites = list(world, "launcherFavorites", &CATALOG.favorites);
    if favorites.iter().any(|item| item == id) {
        favorites.retain(|item| item != id);
    } else if favorites.len() < 64 {
        favorites.push(id.into());
    } else {
        return Err(domain("Limite de 64 favoritos"));
    }
    world.settings.insert(
        "launcherFavorites".into(),
        serde_json::to_string(&favorites)?,
    );
    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolReport {
    pub title: String,
    pub body: String,
    pub saved_path: Option<String>,
}

fn available(world: &WorldState, tool: &Software) -> GameResult<()> {
    if world.packages.managed.contains(&tool.package)
        && world
            .packages
            .installed
            .get(&tool.package)
            .is_none_or(|p| p.status != crate::packages::model::Status::Installed)
    {
        return Err(domain(format!("Package {} is not installed", tool.package)));
    }
    Ok(())
}

pub fn inspect(world: &WorldState, id: &str, target: &str, actor: &str) -> GameResult<ToolReport> {
    let tool = get(id)?;
    available(world, tool)?;
    if target.len() > 4096 || target.chars().any(char::is_control) {
        return Err(domain("Alvo virtual inválido"));
    }
    if tool.resource.is_some() {
        return Err(domain(
            "Esta entrada é uma referência, não um instrumento de inspeção",
        ));
    }
    let details = match tool.operation.as_str() {
        "host" => {
            let host = world.network.host(target.trim())?;
            let ports: String = host
                .services
                .iter()
                .map(|service| {
                    format!(
                        "{}/tcp  {:8}  {} {}\n",
                        service.port,
                        if !host.firewall.contains(&service.port) {
                            "filtered"
                        } else if service.running {
                            "open"
                        } else {
                            "closed"
                        },
                        service.name,
                        service.version
                    )
                })
                .collect();
            format!(
                "HOST {} ({})\n\nPORT     STATE     SERVICE\n{ports}",
                host.hostname, host.address
            )
        }
        "web" => format!(
            "Resposta HTTP do serviço virtual\nAlvo: {target}\n\n{}",
            world.network.request(target.trim())?
        ),
        "file" => {
            let path = normalize(target, HOME)?;
            let content = world.vfs.read(&path, actor)?;
            format!(
                "Arquivo local: {path}\nTamanho: {} bytes\nSHA-256: {:x}\n\nConteúdo textual\n{}",
                content.len(),
                Sha256::digest(content.as_bytes()),
                content.chars().take(12000).collect::<String>()
            )
        }
        "wifi" => {
            if !world.network.connected {
                return Err(domain("Interface virtual desconectada"));
            }
            world
                .network
                .wifi
                .iter()
                .map(|ap| {
                    format!(
                        "{}  {}  canal {}  {} dBm  {}\n",
                        ap.ssid, ap.bssid, ap.channel, ap.signal, ap.encryption
                    )
                })
                .collect()
        }
        "system" => {
            let processes: String = world
                .processes
                .iter()
                .map(|p| {
                    format!(
                        "{}  {}  {}  {}\n",
                        p.pid,
                        p.user,
                        if p.running { "running" } else { "stopped" },
                        p.name
                    )
                })
                .collect();
            format!("Sistema: {} / LifeOS\nUsuário: {actor}\nRede virtual: {}\n\nPID  USER  STATE  PROCESS\n{processes}", world.hostname, if world.network.connected { "conectada" } else { "desconectada" })
        }
        _ => return Err(domain("Operação virtual desconhecida")),
    };
    Ok(ToolReport { title: tool.name.clone(), body: format!("{} · laboratório LifeOS\nInspeção virtual: {}\n\n{details}\nEste relatório usa o mundo do jogo. Não reproduz todos os módulos do software original.\n", tool.name, tool.operation), saved_path: None })
}

pub fn run(
    world: &mut WorldState,
    id: &str,
    target: &str,
    save_path: Option<&str>,
) -> GameResult<ToolReport> {
    let mut report = inspect(world, id, target, "kali")?;
    if let Some(path) = save_path {
        let path = normalize(path, HOME)?;
        if world.vfs.nodes.contains_key(&path) {
            return Err(domain("O relatório já existe; escolha outro nome"));
        }
        world.vfs.write(&path, &report.body, "kali")?;
        report.saved_path = Some(path);
    }
    Ok(report)
}

pub fn command(
    world: &WorldState,
    tool: &Software,
    args: &[String],
    actor: &str,
) -> GameResult<String> {
    if args.is_empty() || args == ["--help"] || args == ["-h"] {
        return Ok(format!("{} — {}\nPacote padrão: {}\n\nLifeOS: workspace de inspeção virtual ({})\nUso: {} --lab [ALVO_VIRTUAL]\nInspeciona o estado atual sem executar o binário original.\nReferência: {}\n", tool.name, tool.description, tool.package, tool.operation, tool.name, tool.source));
    }
    if args.first().map(String::as_str) != Some("--lab") || args.len() > 2 {
        return Err(domain(
            "Use --help ou --lab [ALVO_VIRTUAL]; opções do binário original não são emuladas",
        ));
    }
    Ok(inspect(
        world,
        &tool.id,
        args.get(1).map(String::as_str).unwrap_or(""),
        actor,
    )?
    .body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    #[test]
    fn catalog_is_default_only_and_has_unique_launchers() {
        let ids: BTreeSet<_> = CATALOG.entries.iter().map(|e| &e.id).collect();
        assert_eq!(ids.len(), CATALOG.entries.len());
        assert!(get("kali-nmap").is_ok());
        assert!(get("kali-burpsuite").is_ok());
        assert!(get("kali-ghidra").is_err());
        for id in &CATALOG.favorites {
            assert!(get(id).is_ok());
        }
        for command in CATALOG.entries.iter().flat_map(|e| &e.commands) {
            assert!(
                !command.starts_with("Includes "),
                "invalid imported command: {command}"
            );
        }
    }
    #[test]
    fn inspection_uses_world_permissions_and_never_external_targets() {
        let mut world = WorldState::new("kali", "lifeos").unwrap();
        let report = run(
            &mut world,
            "kali-nmap",
            "vex.local",
            Some("Documents/scan.txt"),
        )
        .unwrap();
        assert!(report.body.contains("22/tcp"));
        assert!(world
            .vfs
            .read("/home/kali/Documents/scan.txt", "kali")
            .unwrap()
            .contains("vex"));
        for target in [
            "google.com",
            "127.0.0.1",
            "192.168.1.1",
            "file:///etc/passwd",
        ] {
            assert!(run(&mut world, "kali-nmap", target, None).is_err());
        }
        assert!(run(
            &mut world,
            "kali-nmap",
            "vex.local",
            Some("/root/report.txt")
        )
        .is_err());
        assert!(run(
            &mut world,
            "kali-nmap",
            "vex.local",
            Some("Documents/scan.txt")
        )
        .is_err());
        let file_tool = CATALOG
            .entries
            .iter()
            .find(|e| e.operation == "file")
            .unwrap();
        assert!(inspect(&world, &file_tool.id, "/root/private.txt", "kali").is_err());
        world.network.connected = false;
        assert!(run(&mut world, "kali-nmap", "vex.local", None).is_err());
    }
    #[test]
    fn launcher_preferences_survive_serialization_and_do_not_duplicate() {
        let mut world = WorldState::new("kali", "lifeos").unwrap();
        open(&mut world, "terminal").unwrap();
        open(&mut world, "kali-nmap").unwrap();
        open(&mut world, "terminal").unwrap();
        favorite(&mut world, "kali-nmap").unwrap();
        let loaded: WorldState =
            serde_json::from_str(&serde_json::to_string(&world).unwrap()).unwrap();
        assert_eq!(
            list(&loaded, "launcherRecent", &[]),
            ["terminal", "kali-nmap"]
        );
        assert!(list(&loaded, "launcherFavorites", &[]).contains(&"kali-nmap".into()));
        assert!(open(&mut world, "not-installed").is_err());
        open(&mut world, "root-terminal").unwrap();
        assert_eq!(world.terminal.user, "kali");
    }
}
