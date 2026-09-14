//! Case instruments operate exclusively on the campaign's virtual network and files.
use crate::{
    error::GameResult,
    network::VirtualWifi,
    vfs::{domain, normalize, HOME},
    world::WorldState,
};
use serde::{Deserialize, Serialize};

pub const SIGNAL_PATH: &str = "/home/kali/projects/orion.signal.json";
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Packet {
    pub number: u32,
    pub time: String,
    pub source: String,
    pub destination: String,
    pub protocol: String,
    pub stream: u32,
    pub text: String,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignalCase {
    pub network: VirtualWifi,
    pub organization: String,
    pub capture_path: String,
    pub packets: Vec<Packet>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignalEvidence {
    pub organization: String,
    pub capture_path: Option<String>,
    pub detail: String,
}
pub fn seed_orion(world: &mut WorldState) -> GameResult<()> {
    let text = include_str!("../../content/cases/orion.json");
    let case: SignalCase = serde_json::from_str(text)?;
    if !world
        .network
        .wifi
        .iter()
        .any(|wifi| wifi.bssid == case.network.bssid)
    {
        world.network.wifi.push(case.network);
    }
    world.vfs.seed(SIGNAL_PATH, "file", text, "kali");
    Ok(())
}
pub fn scan(world: &WorldState) -> GameResult<Vec<VirtualWifi>> {
    if !world.network.connected {
        return Err(domain("Interface virtual desconectada"));
    }
    Ok(world.network.wifi.clone())
}
pub fn inspect(world: &mut WorldState, bssid: &str) -> GameResult<SignalEvidence> {
    let network = scan(world)?
        .into_iter()
        .find(|ap| ap.bssid == bssid)
        .ok_or_else(|| domain("Sinal não encontrado na rede virtual"))?;
    if let Ok(text) = world.vfs.read(SIGNAL_PATH, "kali") {
        let case: SignalCase = serde_json::from_str(&text)?;
        if case.network.bssid == bssid {
            let content = serde_json::to_string_pretty(&case.packets)?;
            let path = normalize(&case.capture_path, HOME)?;
            if world
                .vfs
                .nodes
                .get(&path)
                .is_some_and(|node| node.content != content)
            {
                return Err(domain("O caminho da captura já contém outro arquivo"));
            }
            world.vfs.write(&path, &content, "kali")?;
            world.flags.insert("ORION_SIGNAL_FOUND".into());
            return Ok(SignalEvidence { organization: case.organization, capture_path: Some(path), detail: "Estação corporativa · fluxo 4 preservado\nConversa interna encontrada na evidência do sinal.".into() });
        }
    }
    Ok(SignalEvidence {
        organization: network.ssid,
        capture_path: None,
        detail: "Nenhuma comunicação preservada neste sinal.".into(),
    })
}
pub fn packets(world: &WorldState, path: &str) -> GameResult<Vec<Packet>> {
    let path = normalize(path, HOME)?;
    let text = world.vfs.read(&path, "kali")?;
    let packets: Vec<Packet> = serde_json::from_str(&text)
        .map_err(|_| domain("Este arquivo não contém uma captura virtual válida"))?;
    if packets.len() > 2000 || packets.iter().any(|packet| packet.text.len() > 4096) {
        return Err(domain("Captura excede o limite do instrumento"));
    }
    Ok(packets)
}
pub fn follow_stream(world: &mut WorldState, path: &str, stream: u32) -> GameResult<Vec<Packet>> {
    let selected: Vec<_> = packets(world, path)?
        .into_iter()
        .filter(|packet| packet.stream == stream)
        .collect();
    if selected.is_empty() {
        return Err(domain("Fluxo não encontrado nesta captura"));
    }
    if world.flags.contains("ORION_SIGNAL_FOUND")
        && world
            .missions
            .get("em-claro")
            .is_some_and(|m| m.status == "active")
    {
        let case: SignalCase = serde_json::from_str(&world.vfs.read(SIGNAL_PATH, "kali")?)?;
        if normalize(path, HOME)? == case.capture_path
            && selected.iter().any(|p| p.text == "US$ 2.4B")
        {
            world.flags.insert("ORION_STREAM_READ".into());
        }
    }
    Ok(selected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{save, service::GameService};
    #[test]
    fn orion_case_is_playable_persistent_and_stays_inside_the_virtual_world() {
        let mut game = GameService::new(rusqlite::Connection::open_in_memory().unwrap()).unwrap();
        game.new_game(1, "kali", "game-hacker", false).unwrap();
        let networks = scan(game.world().unwrap()).unwrap();
        assert_eq!(networks.len(), 3);
        assert!(networks.iter().any(|network| network.ssid == "Vizinho_5G"));
        assert!(packets(game.world().unwrap(), "C:\\Windows\\system.ini").is_err());
        game.mutate(
            |_, w, _| {
                w.vfs
                    .write("/home/kali/Documents/notes.txt", "ready", "kali")
            },
            false,
        )
        .unwrap();
        game.mutate(
            |engine, w, events| {
                w.flags.insert("SESSION_1_COMPLETE".into());
                events.push(engine.start(w, "signal-no-ar")?);
                Ok(())
            },
            false,
        )
        .unwrap();
        assert_eq!(scan(game.world().unwrap()).unwrap().len(), 4);
        assert!(game
            .mutate(|_, w, _| inspect(w, "unknown-host"), false)
            .is_err());
        let evidence = game
            .mutate(|_, w, _| inspect(w, "02:00:00:00:20:04"), false)
            .unwrap();
        assert!(game.world().unwrap().flags.contains("SIGNAL_COMPLETE"));
        game.mutate(
            |engine, w, events| {
                events.push(engine.start(w, "em-claro")?);
                Ok(())
            },
            false,
        )
        .unwrap();
        let path = evidence.capture_path.unwrap();
        let stream = game
            .mutate(|_, w, _| follow_stream(w, &path, 4), false)
            .unwrap();
        assert_eq!(stream.len(), 4);
        assert!(game.world().unwrap().flags.contains("EM_CLARO_COMPLETE"));
        assert!(game
            .world()
            .unwrap()
            .messages
            .iter()
            .any(|m| m.contact == "NULL" && m.text == "Te peguei."));
        game.load(1, false).unwrap();
        assert_eq!(packets(game.world().unwrap(), &path).unwrap().len(), 4);
        assert_eq!(
            save::list(&game.connection)
                .unwrap()
                .iter()
                .filter(|slot| slot.occupied)
                .count(),
            1
        );
    }
}
