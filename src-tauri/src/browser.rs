use crate::{domains, error::GameResult, vfs::domain, world::WorldState};
use serde::{Deserialize, Serialize};

pub const SECTOR_IX_INSTALLER: &str = r##"#!/usr/bin/env bash
# SECTOR IX — Protocolo Zero / instalador virtual do Cyber War
mkdir -p "$HOME/Games/sector-ix/bin" "$HOME/Games/sector-ix/assets" "$HOME/Games/sector-ix/config" "$HOME/Games/sector-ix/data" "$HOME/Games/sector-ix/docs" "$HOME/Games/sector-ix/logs" "$HOME/Games/sector-ix/mods" "$HOME/Games/sector-ix/runtime" "$HOME/Games/sector-ix/saves"
touch "$HOME/Games/sector-ix/bin/sector-ix" "$HOME/Games/sector-ix/bin/sector-ix-launcher" "$HOME/Games/sector-ix/assets/.keep" "$HOME/Games/sector-ix/config/settings.conf" "$HOME/Games/sector-ix/data/levels.dat" "$HOME/Games/sector-ix/docs/README.txt" "$HOME/Games/sector-ix/logs/sector-ix.log" "$HOME/Games/sector-ix/mods/.keep" "$HOME/Games/sector-ix/runtime/engine.bin" "$HOME/Games/sector-ix/SECTOR-IX.desktop"
echo '{"game":"SECTOR IX","version":"1.0.0","slot":1,"progress":0,"credits":0,"checkpoint":"start"}' > "$HOME/Games/sector-ix/saves/sector-ix.save"
chmod +x "$HOME/Games/sector-ix/bin/sector-ix"
echo "SECTOR IX instalado em $HOME/Games/sector-ix"
echo "Digite sector-ix para abrir o jogo interno."
"##;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Bookmark {
    pub title: String,
    pub address: String,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Preferences {
    pub home: String,
    pub zoom: u16,
    pub show_bookmarks: bool,
    pub bookmarks: Vec<Bookmark>,
}
pub fn save_preferences(world: &mut WorldState, preferences: Preferences) -> GameResult<()> {
    fn valid_address(address: &str) -> bool {
        if address.len() > 512
            || address
                .chars()
                .any(|c| c.is_control() || c.is_whitespace() || c == '\\')
        {
            return false;
        }
        let host = address
            .strip_prefix("https://")
            .or_else(|| address.strip_prefix("http://"))
            .unwrap_or(address)
            .split('/')
            .next()
            .unwrap_or("");
        let host = host.strip_prefix("www.").unwrap_or(host);
        [
            "wipedia.org",
            "archive.org",
            "fakebook.com",
            "b1.tech",
            "mercado.com.br",
            "meudominio.com.br",
            "vigilia.org",
            domains::BLACKWIRE_ONION_ADDRESS,
        ]
        .contains(&host)
            && host
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b".-".contains(&b))
    }
    if !valid_address(&preferences.home)
        || !(50..=200).contains(&preferences.zoom)
        || preferences.bookmarks.len() > 50
        || preferences.bookmarks.iter().any(|bookmark| {
            bookmark.title.trim().is_empty()
                || bookmark.title.len() > 100
                || bookmark.title.chars().any(char::is_control)
                || !valid_address(&bookmark.address)
        })
    {
        return Err(domain("Preferências inválidas: use sites do navegador do jogo, zoom de 50 a 200% e até 50 favoritos."));
    }
    world.settings.insert(
        "browserPreferences".into(),
        serde_json::to_string(&preferences)?,
    );
    Ok(())
}

#[cfg(test)]
mod preferences_tests {
    use super::*;
    use crate::service::GameService;
    use crate::world::{MissionProgress, WorldState};
    use rusqlite::Connection;
    fn preferences() -> Preferences {
        Preferences {
            home: "wipedia.org".into(),
            zoom: 120,
            show_bookmarks: true,
            bookmarks: vec![Bookmark {
                title: "Arquivo".into(),
                address: "https://archive.org".into(),
            }],
        }
    }
    #[test]
    fn preferences_survive_a_clean_session_and_reject_active_or_host_urls() {
        let mut game = GameService::new(Connection::open_in_memory().unwrap()).unwrap();
        game.new_game(1, "neo", "pc", false).unwrap();
        game.mutate(|_, w, _| save_preferences(w, preferences()), false)
            .unwrap();
        game.end_session().unwrap();
        game.load(1, false).unwrap();
        let saved = game.world().unwrap().settings["browserPreferences"].clone();
        assert_eq!(
            serde_json::from_str::<Preferences>(&saved).unwrap().zoom,
            120
        );
        for address in [
            "javascript:alert(1)",
            "file:///C:/secret",
            "https://example.com",
            "https://archive.org@evil.com",
            "data:text/html,x",
            "wipedia.org\n",
        ] {
            let mut p = preferences();
            p.home = address.into();
            assert!(game
                .mutate(|_, w, _| save_preferences(w, p), false)
                .is_err());
        }
        let mut p = preferences();
        p.bookmarks[0].address = "https://evil.com".into();
        assert!(game
            .mutate(|_, w, _| save_preferences(w, p), false)
            .is_err());
        assert_eq!(game.world().unwrap().settings["browserPreferences"], saved);
    }

    #[test]
    fn every_virtual_site_returns_content_and_story_gates_are_preserved() {
        let mut world = WorldState::new("neo", "pc").unwrap();
        let archive = format!("{}/archive", domains::BLACKWIRE_ONION_ADDRESS);
        for address in vec![
            "wipedia.org",
            "https://archive.org/",
            "b1.tech",
            "https://www.meudominio.com.br",
            domains::BLACKWIRE_ONION_ADDRESS,
            archive.as_str(),
            "mercado.com.br",
            "meudominio.com.br",
            "vigilia.org",
        ] {
            let page = navigate(&mut world, address).unwrap();
            assert!(!page.title.is_empty(), "missing title for {address}");
            assert!(!page.body.is_empty(), "missing body for {address}");
        }

        assert!(navigate(&mut world, "fakebook.com").is_err());
        world.missions.insert(
            "girl".into(),
            MissionProgress {
                status: "active".into(),
                stage: 0,
                attempts: 1,
            },
        );
        let fakebook = navigate(&mut world, "fakebook.com").unwrap();
        assert_eq!(fakebook.action.as_deref(), Some("recover"));
        assert!(world.flags.contains("GIRL_RESEARCHED"));
        assert!(navigate(&mut world, "unknown.example.com").is_err());
    }

    #[test]
    fn sector_ix_download_writes_only_the_linux_package_and_readme() {
        let mut world = WorldState::new("neo", "pc").unwrap();
        action(&mut world, "sector-ix-download", "").unwrap();
        let installer = world
            .vfs
            .read("/home/kali/Downloads/sector-ix-linux.sh", "kali")
            .unwrap();
        assert_eq!(installer, SECTOR_IX_INSTALLER);
        let readme = world
            .vfs
            .read("/home/kali/Downloads/SECTOR-IX-README.txt", "kali")
            .unwrap();
        assert!(readme.contains("sector-ix.save"));
        assert!(!world.vfs.nodes.contains_key("/home/kali/Games/sector-ix"));
        assert!(world.flags.contains("SECTOR_IX_PACKAGE_DOWNLOADED"));
    }
}
#[derive(Serialize)]
pub struct BrowserPage {
    title: String,
    body: String,
    action: Option<String>,
}
pub fn navigate(w: &mut WorldState, address: &str) -> GameResult<BrowserPage> {
    let normalized = address.trim().to_lowercase();
    let normalized = normalized
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .to_owned();
    let (raw_host, path) = normalized.split_once('/').unwrap_or((&normalized, ""));
    if domains::looks_like_onion(raw_host) {
        if !w.network.connected {
            return Err(domain("Sem conexão com a rede Tor."));
        }
        domains::onion_online(w, raw_host)?;
        if raw_host != domains::BLACKWIRE_ONION_ADDRESS {
            return Err(domain("Onion Service não encontrado na rede Tor."));
        }
        let body = if path == "archive" {
            "Índice recuperado · 3 registros\n\n23:17:04 / sessão aberta\nOrigem: orion-edge · estado: preservado\n\n23:17:09 / transferência registrada\nObjeto: registro.enc · destino: não identificado\n\n23:17:12 / trilha interrompida\nA cópia termina antes do encerramento da conexão."
        } else {
            "Rede de espelhos · sessão anônima\n\nORION / 23:17\nO mesmo horário aparece em três registros. Ninguém publicou a origem.\n\nARQUIVO / integridade\nPreserve o arquivo original. Compare os hashes antes de confiar em uma cópia.\n\nNULL / mensagem fixada\nSe encontrou o espelho, alguém deixou o caminho aberto."
        };
        return Ok(BrowserPage {
            title: if path == "archive" {
                "BLACKWIRE / arquivo".into()
            } else {
                "BLACKWIRE / índice".into()
            },
            body: body.into(),
            action: None,
        });
    }
    let host = raw_host.strip_prefix("www.").unwrap_or(raw_host);
    let host = host.trim_end_matches('/');
    if host != "wipedia.org" && !w.network.connected {
        return Err(domain(
            "Sem conexão. Sua Wipédia local continua disponível.",
        ));
    }
    match host {
            "archive.org"=>Ok(BrowserPage{title:"Archive · Cyber Siege".into(),body:w.network.request("https://archive.org")?,action:Some("download".into())}),
            "fakebook.com"=>{
                if !w.missions.get("girl").is_some_and(|p|p.status=="active"||p.status=="completed"){return Err(domain("Perfil ainda não conhecido. Gregory pode fornecer o contato."));}
                w.flags.insert("GIRL_RESEARCHED".into());
                Ok(BrowserPage{title:"FakeBook · Perfil público".into(),body:if w.flags.contains("GIRL_ACCESS"){ "Mensagens privadas\nAs conversas confirmam a traição. Gregory pede o acesso pelo mensageiro.".into()}else{"Álbum público: FOTO-17\nRegistro de recuperação associado: 17\nCorrelacione o código do álbum no laboratório da plataforma.".into()},action:Some("recover".into())})
            }
            "b1.tech"=>Ok(BrowserPage{title:"B1 · Notícias".into(),body:if w.flags.contains("SESSION_1_COMPLETE"){ "Privacidade: invasões de perfis voltam ao debate\nVítimas procuram respostas sobre histórico de login e dispositivos desconhecidos.\n\nOrion investiga falha em sua rede corporativa\nA empresa confirmou uma revisão de acessos após relatos de tráfego incomum. A apuração continua.\n\nComunidades preservam registros para investigação\nEspecialistas recomendam manter os arquivos originais e conferir a integridade das cópias.".into()}else{"Nova atualização de Cyber Siege corrige comportamento inesperado\nA equipe publicou uma nova revisão nesta madrugada.\n\nTecnologia: comunidades se reúnem para desafios de segurança\nParticipantes compartilham descobertas em fóruns e laboratórios locais.".into()},action:None}),
            "wipedia.org"=>Ok(BrowserPage{title:"Wipédia · Biblioteca".into(),body:"ARQUIVOS\npwd mostra onde você está; ls lista; cd navega. echo texto > arquivo cria uma nota. cp preserva uma cópia; mv move; cat lê.\n\nREDE\nifconfig mostra sua interface. ping verifica se um host responde. Consultas usam nomes presentes no mundo: vex.local, archive.org.\n\nP2P\nPeers compartilham blocos; seeds possuem o arquivo completo.\n\nADMINISTRAÇÃO\nLeia logs antes de alterar configuração. No servidor de VEX, sudo administra o ambiente. Backup só vale se a restauração for validada.\n\nTECHNICAL JOURNEY\nConhecimento não é bloqueado por nível. Uma técnica conta quando produz resultado válido.".into(),action:None}),
            "mercado.com.br"=>Ok(BrowserPage{title:"Mercado Aberto".into(),body:format!("Memória virtual adicional · R$ 100\nSeu saldo: R$ {}\nMelhorias integram o inventário desta campanha.",w.money),action:Some("upgrade".into())}),
            "meudominio.com.br"=>Ok(BrowserPage{title:"MeuDomínio · Mercado de domínios".into(),body:"Pesquise, registre e administre os domínios da sua empresa.".into(),action:None}),
            "vigilia.org"=>Ok(BrowserPage{title:"SECTOR IX — Protocolo Zero".into(),body:"Download seguro do runtime interno. Use o terminal Linux para baixar, instalar e executar o jogo.".into(),action:Some("sector-ix-download".into())}),
            _=>Err(domain("Endereço não encontrado na internet do jogo."))
        }
}
pub fn action(w: &mut WorldState, action: &str, value: &str) -> GameResult<()> {
    if !w.network.connected {
        return Err(domain("network unreachable"));
    }
    match action {
        "download" => {
            if !w.inventory.contains("cyber-siege") {
                return Err(domain("manifest not requested"));
            }
            let text = w.network.request("https://archive.org")?;
            w.vfs
                .write("/home/kali/Downloads/cyber-siege.manifest", &text, "kali")?;
            w.flags.insert("GAME_DOWNLOADED".into());
        }
        "recover" => {
            if !w.missions.get("girl").is_some_and(|p| p.status == "active")
                || !w.flags.contains("GIRL_RESEARCHED")
                || value != "FOTO-17"
            {
                return Err(domain("correlation failed"));
            }
            w.flags.insert("GIRL_ACCESS".into());
            w.techniques.insert("recovery-correlation".into());
        }
        "upgrade" => {
            if w.inventory.contains("memory-upgrade") {
                return Err(domain("upgrade already installed"));
            }
            if w.money < 100 {
                return Err(domain("saldo insuficiente"));
            }
            w.money -= 100;
            w.inventory.insert("memory-upgrade".into());
        }
        "sector-ix-download" => {
            let installer = w.network.request("https://vigilia.org/download/sector-ix-linux.sh")?;
            w.vfs.write(
                "/home/kali/Downloads/sector-ix-linux.sh",
                &installer,
                "kali",
            )?;
            w.vfs.write(
                "/home/kali/Downloads/SECTOR-IX-README.txt",
                "SECTOR IX — Protocolo Zero\n\ncd ~/Downloads\nwget -O sector-ix-linux.sh https://www.vigilia.org/download/sector-ix-linux.sh\nchmod +x sector-ix-linux.sh\nbash sector-ix-linux.sh\nsector-ix\n\nA instalação cria uma árvore visual em ~/Games/sector-ix. O único arquivo funcional da instalação é saves/sector-ix.save.\n\nO jogo também está disponível em Aplicativos > Jogos > SECTOR IX. Ele inicia em janela; use o botão de fullscreen dentro do jogo quando quiser.\n",
                "kali",
            )?;
            w.flags.insert("SECTOR_IX_PACKAGE_DOWNLOADED".into());
        }
        _ => return Err(domain("unknown virtual action")),
    }
    Ok(())
}
