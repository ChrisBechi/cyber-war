use crate::{
    error::GameResult,
    vfs::{domain, VirtualFileSystem},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VirtualService {
    pub port: u16,
    pub name: String,
    pub version: String,
    pub running: bool,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VirtualHost {
    pub address: String,
    pub hostname: String,
    pub online: bool,
    pub services: Vec<VirtualService>,
    pub credentials: BTreeMap<String, String>,
    pub firewall: Vec<u16>,
    pub patched: bool,
    pub files: VirtualFileSystem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VirtualWifi {
    pub ssid: String,
    pub bssid: String,
    pub channel: u8,
    pub signal: i32,
    pub encryption: String,
    pub clients: u32,
    pub access: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VirtualNetwork {
    pub connected: bool,
    pub gateway: String,
    pub subnet: String,
    pub hosts: BTreeMap<String, VirtualHost>,
    pub dns: BTreeMap<String, String>,
    pub wifi: Vec<VirtualWifi>,
}

impl VirtualNetwork {
    pub fn initial() -> GameResult<Self> {
        let mut network: Self =
            serde_json::from_str(include_str!("../../content/networks/initial.json"))?;
        for host in network.hosts.values_mut() {
            host.files = VirtualFileSystem::default();
            for node in host.files.nodes.values_mut() {
                if node.owner == "kali" {
                    node.owner = "vex".into();
                    node.group = "vex".into();
                }
            }
            host.files.seed(
                "/etc/web.conf",
                "file",
                "enabled=false\nroot_login=true\nfirewall=partial\n",
                "root",
            );
            host.files.seed(
                "/var/log/web.log",
                "file",
                "ERROR: service disabled in /etc/web.conf\n",
                "root",
            );
            host.files.seed(
                "/srv/www/index.txt",
                "file",
                "Serviço VEX restaurado.\n",
                "root",
            );
        }
        Ok(network)
    }

    pub fn resolve(&self, target: &str) -> GameResult<String> {
        if !self.connected {
            return Err(domain("network is unreachable"));
        }
        if target.trim().to_lowercase().ends_with(".onion") {
            return Err(domain(
                "Endereços .onion não usam DNS tradicional; acesse pelo navegador Tor.",
            ));
        }
        let canonical = target.strip_prefix("www.").unwrap_or(target);
        let legacy = match canonical {
            "archive.org" => Some("archive.local"),
            _ => None,
        };
        let address = self
            .dns
            .get(target)
            .or_else(|| self.dns.get(canonical))
            .or_else(|| legacy.and_then(|name| self.dns.get(name)))
            .map(String::as_str)
            .unwrap_or(target);
        if self.hosts.contains_key(address) {
            Ok(address.into())
        } else {
            Err(domain(format!("{target}: unknown virtual host")))
        }
    }

    pub fn host(&self, target: &str) -> GameResult<&VirtualHost> {
        let address = self.resolve(target)?;
        let host = self
            .hosts
            .get(&address)
            .ok_or_else(|| domain("unknown host"))?;
        if !host.online {
            return Err(domain("host is offline"));
        }
        Ok(host)
    }

    pub fn request(&self, url: &str) -> GameResult<String> {
        let (scheme, rest) = url.split_once("://").unwrap_or(("https", url));
        if !["http", "https"].contains(&scheme) {
            return Err(domain("unsupported virtual protocol"));
        }
        let (target, request_path) = rest.split_once('/').unwrap_or((rest, ""));
        let port = if scheme == "https" { 443 } else { 80 };
        let host = self.host(target)?;
        let service = host
            .services
            .iter()
            .find(|s| s.port == port && s.running && host.firewall.contains(&port))
            .ok_or_else(|| domain("connection refused"))?;
        if host.hostname == "vigilia.org" && request_path == "download/sector-ix-linux.sh" {
            return Ok(crate::browser::SECTOR_IX_INSTALLER.into());
        }
        Ok(service.body.clone())
    }

    pub fn ssh(&self, target: &str, password: &str) -> GameResult<(String, String)> {
        let (user, address) = target
            .split_once('@')
            .ok_or_else(|| domain("usage: ssh user@host password"))?;
        let host = self.host(address)?;
        if !host.firewall.contains(&22) || !host.services.iter().any(|s| s.port == 22 && s.running)
        {
            return Err(domain("connection refused"));
        }
        if host.credentials.get(user).map(String::as_str) != Some(password) {
            return Err(domain("authentication failed"));
        }
        if user == "root"
            && host
                .files
                .nodes
                .get("/etc/web.conf")
                .is_some_and(|n| n.content.contains("root_login=false"))
        {
            return Err(domain("root login disabled"));
        }
        Ok((host.address.clone(), user.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_seeded_hosts_resolve() {
        let net = VirtualNetwork::initial().expect("content");
        assert!(net.host("vex.local").is_ok());
        for (site, ip) in [
            ("archive.org", "10.20.4.20"),
            ("www.wipedia.org", "10.20.4.30"),
            ("www.meudominio.com.br", "10.20.4.34"),
        ] {
            assert_eq!(net.resolve(site).unwrap(), ip);
        }
        assert!(net.resolve("blackwire.onion").is_err());
        for host in ["google.com", "127.0.0.1", "192.168.1.1", "169.254.169.254"] {
            assert!(net.host(host).is_err());
        }
        assert!(net.request("file:///etc/passwd").is_err());
        assert!(net.ssh("root@vex.local", "wrong").is_err());
        assert!(net.ssh("vex@vex.local", "lab-only").is_ok());
    }
}
