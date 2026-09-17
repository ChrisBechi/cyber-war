use super::{deb, model::*};
use crate::{error::GameResult, vfs::domain, world::WorldState};
use std::{collections::BTreeMap, sync::LazyLock};

pub const OFFICIAL: &str = "https://mirror.kali.game/kali";
pub const COMMUNITY: &str = "https://repo.blackwire.net/tools";

pub fn file(
    path: &str,
    content: &str,
    logical_size: u64,
    conffile: bool,
    binding: Option<&str>,
) -> Payload {
    Payload {
        path: path.into(),
        kind: "file".into(),
        content: content.into(),
        mode: if binding.is_some() { 0o755 } else { 0o644 },
        logical_size: logical_size.max(content.len() as u64),
        conffile,
        binding: binding.map(String::from),
    }
}
pub fn definition(name: &str, version: &str, description: &str) -> Definition {
    Definition {
        name: name.into(),
        version: version.into(),
        architecture: "amd64".into(),
        description: description.into(),
        section: "utils".into(),
        maintainer: "CYBER WAR Virtual Software Team".into(),
        homepage: OFFICIAL.into(),
        priority: "optional".into(),
        essential: false,
        download_size: 64 * 1024,
        keywords: Vec::new(),
        depends: Vec::new(),
        recommends: Vec::new(),
        suggests: Vec::new(),
        conflicts: Vec::new(),
        provides: Vec::new(),
        replaces: Vec::new(),
        files: vec![file(
            &format!("/usr/share/doc/{name}/README"),
            description,
            description.len() as u64,
            false,
            None,
        )],
        actions: Vec::new(),
    }
}
pub fn relation(name: &str, op: &str, version: &str) -> Relation {
    Relation {
        name: name.into(),
        op: op.into(),
        version: version.into(),
    }
}
pub fn tool(name: &str, version: &str, description: &str, binding: &str) -> Definition {
    let mut p = definition(name, version, description);
    p.files.extend([
        file(&format!("/usr/bin/{name}"),&format!("CYBER-WAR EXECUTABLE\n{binding}\n{version}\n"),2*1024*1024,false,Some(binding)),
        file(&format!("/etc/{name}/{name}.conf"),"# Virtual tool configuration\nverbose=false\n",128,true,None),
        file(&format!("/usr/share/man/man1/{name}.1"),&format!("{name}(1)\n{description}\n\n{name} [--version|--help] [VIRTUAL_TARGET]\nOnly the game world is inspected.\n"),512,false,None),
    ]);
    p
}

pub static BASELINE: &[(&str, &str, &str)] = &[
    ("base-files", "13.8", "Debian base system"),
    ("bash", "5.2.37", "Virtual shell"),
    ("coreutils", "9.7", "Virtual core utilities"),
    ("curl", "8.14.1", "Virtual transfer client"),
    ("iproute2", "6.15.0", "Virtual network utilities"),
    ("nano", "8.7", "Text editor"),
    ("nmap", "7.98", "Virtual network scanner"),
    ("openssh-client", "10.0p2", "Virtual SSH client"),
    ("procps", "4.0.4", "Virtual process tools"),
    ("sudo", "1.9.17p1", "Virtual privilege adapter"),
    ("systemd", "257.5", "Virtual service controls"),
    ("wget", "1.25", "Virtual download client"),
];
pub fn baseline() -> Vec<Definition> {
    BASELINE
        .iter()
        .map(|(name, version, description)| {
            let mut p = definition(name, version, description);
            p.essential = matches!(
                *name,
                "base-files" | "bash" | "coreutils" | "sudo" | "systemd"
            );
            let commands: &[&str] = match *name {
                "base-files" => &[
                    "apt",
                    "apt-get",
                    "dpkg",
                    "dpkg-query",
                    "apt-cache",
                    "apt-mark",
                ],
                "coreutils" => &[
                    "base64",
                    "basename",
                    "cat",
                    "chmod",
                    "chown",
                    "cp",
                    "cut",
                    "date",
                    "df",
                    "dirname",
                    "du",
                    "env",
                    "groups",
                    "head",
                    "id",
                    "ls",
                    "mkdir",
                    "mv",
                    "printenv",
                    "readlink",
                    "realpath",
                    "rm",
                    "seq",
                    "sha256sum",
                    "sort",
                    "stat",
                    "tail",
                    "tee",
                    "touch",
                    "tr",
                    "uname",
                    "uniq",
                    "wc",
                    "whoami",
                    "yes",
                    "ln",
                    "rmdir",
                ],
                "bash" => &["bash", "sh"],
                "iproute2" => &["ip", "ifconfig"],
                "openssh-client" => &["ssh", "scp"],
                "procps" => &["ps", "top", "kill"],
                "systemd" => &["systemctl", "service", "journalctl"],
                _ => &[],
            };
            let commands = if commands.is_empty() {
                vec![*name]
            } else {
                commands.to_vec()
            };
            for command in commands {
                p.files.push(file(
                    &format!("/usr/bin/{command}"),
                    &format!("CYBER-WAR BUILTIN\n{command}\n"),
                    1024,
                    false,
                    Some("builtin"),
                ));
            }
            p
        })
        .collect()
}

pub struct Repository {
    pub suite: &'static str,
    pub entries: Vec<(u32, IndexEntry)>,
}
pub static REPOSITORIES: LazyLock<BTreeMap<String, Repository>> = LazyLock::new(|| {
    let mut official = baseline();
    let mut core = definition(
        "cyber-core",
        "1.0",
        "Virtual runtime data shared by packages",
    );
    core.architecture = "all".into();
    core.files.push(file(
        "/usr/lib/cyber-core/runtime.dat",
        "CYBER WAR runtime data",
        2 * 1024 * 1024,
        false,
        None,
    ));
    let mut library = definition("libpacket2", "2.0", "Packet inspection data library");
    library.depends = vec![vec![relation("cyber-core", ">=", "1.0")]];
    library.files.push(file(
        "/usr/lib/libpacket2/packets.dat",
        "ethernet ipv4 ipv6 tcp udp",
        7 * 1024 * 1024,
        false,
        None,
    ));
    library.download_size = 2 * 1024 * 1024;
    let mut scan = tool(
        "netscan",
        "2.4.1",
        "Network discovery and analysis utility",
        "netscan",
    );
    scan.depends = vec![vec![relation("libpacket2", ">=", "2.0")]];
    scan.download_size = 1800 * 1024;
    scan.keywords = vec!["network".into(), "wireless".into(), "discovery".into()];
    scan.files.push(file("/usr/share/applications/netscan.desktop","[Desktop Entry]\nType=Application\nName=NetScan\nExec=netscan\nTerminal=true\nIcon=utilities-terminal\n",256,false,None));
    let mut old = scan.clone();
    old.version = "2.3.0".into();
    for f in &mut old.files {
        f.content = f.content.replace("2.4.1", "2.3.0");
    }
    let wireless = tool(
        "wireless-utils",
        "1.0",
        "Wireless access point inspection utilities",
        "wireless",
    );
    official.extend([core, library, scan, old, wireless]);
    let mut community = tool(
        "iot-discovery",
        "1.0",
        "Discover IoT services in the virtual network",
        "iot",
    );
    community.depends = vec![vec![relation("libpacket2", ">=", "2.0")]];
    community.homepage = COMMUNITY.into();
    let make = |url: &str, suite: &'static str, packages: Vec<Definition>| Repository {
        suite,
        entries: packages
            .into_iter()
            .map(|package| {
                let checksum =
                    deb::hash(&deb::encode(&package).expect("bundled package validated"));
                (
                    1,
                    IndexEntry {
                        package,
                        repository: url.into(),
                        suite: suite.into(),
                        component: "main".into(),
                        checksum,
                    },
                )
            })
            .collect(),
    };
    let mut official = make(OFFICIAL, "rolling", official);
    let mut upgrade = tool(
        "netscan",
        "2.5.0",
        "Network discovery and analysis utility",
        "netscan",
    );
    upgrade.depends = vec![vec![relation("libpacket2", ">=", "2.0")]];
    upgrade.files.push(file("/usr/share/applications/netscan.desktop", "[Desktop Entry]\nType=Application\nName=NetScan\nExec=netscan\nTerminal=true\nIcon=utilities-terminal\n", 256, false, None));
    upgrade
        .files
        .iter_mut()
        .filter(|f| f.conffile)
        .for_each(|f| {
            f.content = "# Virtual tool configuration\nverbose=false\nipv6=true\n".into()
        });
    let checksum = deb::hash(&deb::encode(&upgrade).expect("bundled upgrade validated"));
    official.entries.push((
        2,
        IndexEntry {
            package: upgrade,
            repository: OFFICIAL.into(),
            suite: "rolling".into(),
            component: "main".into(),
            checksum,
        },
    ));
    BTreeMap::from([
        (OFFICIAL.into(), official),
        (COMMUNITY.into(), make(COMMUNITY, "stable", vec![community])),
    ])
});

#[derive(Debug, Clone)]
pub struct Source {
    pub uri: String,
    pub suite: String,
    pub components: Vec<String>,
}
pub fn parse_sources(world: &WorldState) -> GameResult<Vec<Source>> {
    let mut out = Vec::new();
    for (path, node) in &world.vfs.nodes {
        if path != "/etc/apt/sources.list"
            && !(path.starts_with("/etc/apt/sources.list.d/") && path.ends_with(".list"))
        {
            continue;
        }
        if node.kind != "file" || node.blob.is_some() {
            return Err(domain(format!("E: Invalid source file {path}")));
        }
        for (line, text) in node.content.lines().enumerate() {
            let text = text.split('#').next().unwrap_or("").trim();
            if text.is_empty() {
                continue;
            }
            let parts = text.split_whitespace().collect::<Vec<_>>();
            if parts.len() < 4
                || parts[0] != "deb"
                || !parts[1].starts_with("https://")
                || parts[1].contains(['@', '?', '#', '\\'])
                || parts[2..].iter().any(|s| {
                    !s.bytes()
                        .all(|c| c.is_ascii_alphanumeric() || b"-_.".contains(&c))
                })
            {
                return Err(domain(format!(
                    "E: Malformed source {path}:{} (supported: deb HTTPS_URI SUITE COMPONENT...)",
                    line + 1
                )));
            }
            out.push(Source {
                uri: parts[1].trim_end_matches('/').into(),
                suite: parts[2].into(),
                components: parts[3..].iter().map(|s| (*s).into()).collect(),
            });
        }
    }
    Ok(out)
}
pub fn artifact(entry: &IndexEntry, world: &WorldState) -> GameResult<Vec<u8>> {
    let repo = REPOSITORIES
        .get(&entry.repository)
        .ok_or_else(|| domain("E: Repository not found"))?;
    let state = world
        .packages
        .repositories
        .get(&entry.repository)
        .cloned()
        .unwrap_or_default();
    if !world.network.connected || !state.available {
        return Err(domain("E: Repository offline"));
    }
    if !state.trusted {
        return Err(domain("E: Repository is untrusted"));
    }
    if !repo.entries.iter().any(|(release, e)| {
        *release <= state.release
            && e.package.key() == entry.package.key()
            && e.checksum == entry.checksum
    }) {
        return Err(domain("E: Package version is no longer available"));
    }
    let bytes = deb::encode(&entry.package)?;
    if deb::hash(&bytes) != entry.checksum {
        return Err(domain("E: Package checksum mismatch"));
    }
    Ok(bytes)
}
