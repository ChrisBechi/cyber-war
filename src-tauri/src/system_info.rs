//! Fetch-style information for the selected virtual computer, never host telemetry.
use crate::{error::GameResult, packages::model::Status, vfs::domain, world::WorldState};
use serde::Deserialize;

const DRAGON: &str = include_str!("../../public/assets/fastfetch/kali.txt");
const BLUE: &str = "\x1b[38;2;54;123;240m";
const RESET: &str = "\x1b[0m";
pub const HELP: &str = "fastfetch / neofetch — informações do computador virtual\n\nUso: fastfetch [--pipe] [--logo none] [--help] [--version]\n     neofetch [--stdout]\n\nExibe o dragão Kali, usuário, sistema, pacotes, recursos e rede da campanha.\n--pipe / --stdout: somente dados em texto, sem cores nem desenho.\n--logo none: oculta o desenho. A largura acompanha a janela do terminal.\nCPU, GPU e memória são recursos virtuais do jogo.\n";

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Presentation {
    pub columns: usize,
    pub display_width: u32,
    pub display_height: u32,
}
impl Default for Presentation {
    fn default() -> Self {
        Self {
            columns: 100,
            display_width: 0,
            display_height: 0,
        }
    }
}

fn clean(value: &str) -> String {
    value
        .chars()
        .filter(|c| !c.is_control())
        .take(256)
        .collect()
}
fn setting<'a>(world: &'a WorldState, key: &str, default: &'a str) -> &'a str {
    world
        .settings
        .get(key)
        .map(String::as_str)
        .unwrap_or(default)
}
fn human_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.2} GiB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else {
        format!("{:.2} MiB", bytes as f64 / (1024.0 * 1024.0))
    }
}
fn usage(used: u64, total: u64) -> String {
    format!(
        "{} / {} ({}%)",
        human_bytes(used),
        human_bytes(total),
        used.saturating_mul(100).checked_div(total).unwrap_or(0)
    )
}

fn fields(world: &WorldState, actor: &str) -> GameResult<Vec<(String, String)>> {
    let fs = world.fs()?;
    let remote = world
        .terminal
        .host
        .as_ref()
        .and_then(|id| world.network.hosts.get(id));
    let hostname = remote.map_or(world.hostname.as_str(), |host| host.hostname.as_str());
    let release = fs.read("/etc/os-release", actor).unwrap_or_default();
    let release_field = |key: &str| {
        release.lines().find_map(|line| {
            let (name, value) = line.split_once('=')?;
            (name == key).then(|| value.trim_matches(['\'', '"']).to_string())
        })
    };
    let os = release_field("PRETTY_NAME").unwrap_or_else(|| {
        format!(
            "{} {}",
            release_field("NAME").unwrap_or_else(|| "LifeOS".into()),
            release_field("VERSION").unwrap_or_default()
        )
        .trim()
        .to_string()
    });
    let packages = if remote.is_none() {
        world
            .packages
            .installed
            .values()
            .filter(|p| p.status == Status::Installed)
            .count()
    } else {
        fs.read("/var/lib/dpkg/status", actor)
            .unwrap_or_default()
            .lines()
            .filter(|line| *line == "Status: install ok installed")
            .count()
    };
    let seconds = world.playtime_seconds;
    let uptime = if seconds >= 86400 {
        format!(
            "{} days, {} hours, {} mins",
            seconds / 86400,
            seconds / 3600 % 24,
            seconds / 60 % 60
        )
    } else if seconds >= 3600 {
        format!("{} hours, {} mins", seconds / 3600, seconds / 60 % 60)
    } else if seconds >= 60 {
        format!("{} mins", seconds / 60)
    } else {
        format!("{seconds} secs")
    };
    let presentation = &world.terminal.presentation;
    let display = if presentation.display_width > 0 && presentation.display_height > 0 {
        format!(
            "{}x{}",
            presentation.display_width.min(16384),
            presentation.display_height.min(16384)
        )
    } else {
        setting(world, "displayResolution", "1440x900").to_string()
    };
    let environment = crate::terminal::virtual_env(world, actor);
    let shell = environment
        .get("SHELL")
        .map(String::as_str)
        .unwrap_or("/bin/bash")
        .rsplit('/')
        .next()
        .unwrap_or("bash");
    let shell_version = world
        .packages
        .installed
        .get(shell)
        .filter(|p| p.status == Status::Installed)
        .map(|p| p.definition.version.as_str())
        .unwrap_or("");
    let (desktop, manager) = match setting(world, "desktopEnvironment", "xfce")
        .split(',')
        .next()
        .unwrap_or("xfce")
    {
        "gnome" => ("GNOME", "Mutter"),
        "kde" => ("KDE Plasma", "KWin"),
        _ => ("Xfce", "Xfwm4"),
    };
    let resources = crate::task_manager::snapshot(world);
    let ip = remote.map_or_else(
        || {
            if world.network.connected {
                "10.20.4.2/24".to_string()
            } else {
                "desconectada".into()
            }
        },
        |host| host.address.clone(),
    );
    let locale = environment
        .get("LC_ALL")
        .or_else(|| environment.get("LANG"))
        .map(String::as_str)
        .unwrap_or("C.UTF-8");
    let mut result = vec![
        (String::new(), format!("{actor}@{hostname}")),
        ("OS".into(), format!("{os} GNU/Linux x86_64")),
        (
            "Host".into(),
            if remote.is_some() {
                "LifeOS Virtual Server".into()
            } else {
                "LifeOS Virtual Computer".into()
            },
        ),
        ("Kernel".into(), "6.6.0-lifeos-sim".into()),
        ("Uptime".into(), uptime),
        ("Packages".into(), format!("{packages} (dpkg)")),
        (
            "Shell".into(),
            format!("{shell} {shell_version}").trim().to_string(),
        ),
    ];
    if remote.is_none() {
        result.extend([
            ("Display (Virtual-1)".into(), display),
            ("DE".into(), desktop.into()),
            ("WM".into(), manager.into()),
            ("WM Theme".into(), "Kali-Dark".into()),
            ("Theme".into(), "Kali-Dark".into()),
            ("Icons".into(), "Flat-Remix-Blue-Dark".into()),
            ("Font".into(), "Segoe UI (13px)".into()),
            ("Cursor".into(), "Adwaita".into()),
        ]);
    }
    result.extend([
        ("Terminal".into(), "LifeOS Terminal (xterm.js)".into()),
        (
            "Terminal Font".into(),
            format!(
                "Cascadia Mono / Consolas ({}px)",
                setting(world, "fontSize", "14")
            ),
        ),
        ("CPU".into(), "LifeOS Virtual CPU (x86_64)".into()),
        ("GPU".into(), "LifeOS Virtual Display Adapter".into()),
        (
            "Memory".into(),
            if remote.is_none() {
                usage(resources.memory_used_bytes, resources.memory_total_bytes)
            } else {
                "não disponível nesta sessão SSH".into()
            },
        ),
        ("Swap".into(), "0 B / 512 MiB (0%)".into()),
        (
            "Disk (/)".into(),
            format!("{} - ext4", usage(fs.used_bytes(), fs.capacity_bytes)),
        ),
        (
            format!(
                "Local IP ({})",
                if remote.is_none() {
                    setting(world, "network", "eth0")
                } else {
                    "eth0"
                }
            ),
            ip,
        ),
        ("Locale".into(), locale.into()),
    ]);
    Ok(result
        .into_iter()
        .map(|(key, value)| (clean(&key), clean(&value)))
        .collect())
}

// Wrap values before adding ANSI sequences, so escape bytes never affect alignment.
fn chunks(text: &str, width: usize) -> Vec<String> {
    let chars: Vec<_> = text.chars().collect();
    if chars.is_empty() {
        return vec![String::new()];
    }
    chars
        .chunks(width.max(1))
        .map(|chunk| chunk.iter().collect())
        .collect()
}

pub fn run(world: &WorldState, name: &str, args: &[String], actor: &str) -> GameResult<String> {
    let mut plain = !world.terminal.io.stdout_tty;
    let mut logo = true;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--help" | "-h" => return Ok(HELP.into()),
            "--version" | "-v" => {
                return Ok(format!(
                    "{name} — CYBER WAR virtual implementation {}\n",
                    env!("CARGO_PKG_VERSION")
                ))
            }
            "--pipe" | "--stdout" => {
                plain = true;
                logo = false;
            }
            "--logo" if args.get(index + 1).is_some_and(|value| value == "none") => {
                logo = false;
                index += 1;
            }
            arg => {
                return Err(domain(format!(
                    "{name}: opção não suportada: {arg}\nUse {name} --help"
                )))
            }
        }
        index += 1;
    }
    let data = fields(world, actor)?;
    if plain {
        return Ok(data
            .iter()
            .map(|(key, value)| {
                if key.is_empty() {
                    format!("{value}\n")
                } else {
                    format!("{key}: {value}\n")
                }
            })
            .collect());
    }
    let columns = world.terminal.presentation.columns.clamp(20, 500);
    let dragon: Vec<_> = DRAGON
        .lines()
        .map(|line| line.replace("$1", "").replace("$2", ""))
        .collect();
    let logo_width = dragon.iter().map(String::len).max().unwrap_or(0);
    let side_by_side = logo && columns >= logo_width + 3 + 28;
    let width = if side_by_side {
        columns - logo_width - 3
    } else {
        columns
    };
    let mut lines = Vec::new();
    for (key, value) in data {
        if key.is_empty() {
            lines.extend(
                chunks(&value, width)
                    .into_iter()
                    .map(|line| format!("\x1b[1m{line}{RESET}")),
            );
            lines.push("-".repeat(value.chars().count().min(width)));
        } else {
            let prefix = format!("{key}: ");
            let prefix_width = prefix.chars().count();
            if prefix_width >= width {
                lines.extend(chunks(&format!("{prefix}{value}"), width));
            } else {
                for (index, part) in chunks(&value, width - prefix_width).iter().enumerate() {
                    lines.push(if index == 0 {
                        format!("\x1b[1m{BLUE}{prefix}{RESET}{part}")
                    } else {
                        format!("{}{part}", " ".repeat(prefix_width))
                    });
                }
            }
        }
    }
    lines.push(String::new());
    for start in [40, 100] {
        lines.push(
            (start..start + 8)
                .map(|code| format!("\x1b[{code}m{}", " ".repeat((width / 8).clamp(1, 3))))
                .collect::<String>()
                + RESET,
        );
    }
    let mut output = String::new();
    if side_by_side {
        for index in 0..dragon.len().max(lines.len()) {
            let art = dragon.get(index).map(String::as_str).unwrap_or("");
            let line = lines.get(index).map(String::as_str).unwrap_or("");
            output.push_str(&format!(
                "{BLUE}{art}{RESET}{}{line}\n",
                " ".repeat(logo_width - art.len() + 3)
            ));
        }
    } else {
        if logo && columns >= logo_width {
            for line in dragon {
                output.push_str(&format!("{BLUE}{line}{RESET}\n"));
            }
            output.push('\n');
        }
        for line in lines {
            output.push_str(&line);
            output.push('\n');
        }
    }
    output.push_str(RESET);
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{completion, terminal::execute, terminal_sessions};

    fn plain(text: &str) -> String {
        regex::Regex::new(r"\x1b\[[0-9;]*m")
            .unwrap()
            .replace_all(text, "")
            .into_owned()
    }

    #[test]
    #[ignore = "exports backend output for the isolated xterm visual preview"]
    fn export_fastfetch_visual_fixture() {
        let mut outputs = serde_json::Map::new();
        for columns in [48, 80, 100, 132, 160] {
            let mut world = WorldState::new("kali", "kali").unwrap();
            world.playtime_seconds = 12 * 60;
            world.terminal.presentation = Presentation {
                columns,
                display_width: 1920,
                display_height: 1080,
            };
            outputs.insert(
                columns.to_string(),
                serde_json::to_value(execute(&mut world, "fastfetch")).unwrap(),
            );
        }
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../artifacts/fastfetch-qa.json");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, serde_json::to_string_pretty(&outputs).unwrap()).unwrap();
    }

    #[test]
    fn fastfetch_renders_dragon_colors_and_current_campaign_data() {
        let mut world = WorldState::new("neo", "cyber-pc").unwrap();
        world.playtime_seconds = 7380;
        world.terminal.presentation = Presentation {
            columns: 132,
            display_width: 1920,
            display_height: 1080,
        };
        let packages = world
            .packages
            .installed
            .values()
            .filter(|p| p.status == Status::Installed)
            .count();
        let result = execute(&mut world, "fastfetch");
        assert_eq!(result.exit_code, 0, "{}", result.stderr);
        let text = plain(&result.stdout);
        assert!(result.stdout.contains(BLUE));
        assert!(result.stdout.contains("\x1b[100m"));
        assert!(text.lines().next().unwrap().contains(".............."));
        assert!(text.lines().next().unwrap().contains("kali@cyber-pc"));
        for value in [
            "0Xxoc:,.  ...",
            "Uptime: 2 hours, 3 mins",
            "Display (Virtual-1): 1920x1080",
            "6.6.0-lifeos-sim",
            "Flat-Remix-Blue-Dark",
            "Local IP (eth0): 10.20.4.2/24",
        ] {
            assert!(text.contains(value), "missing {value}");
        }
        assert!(text.contains(&format!("Packages: {packages} (dpkg)")));
        assert!(!text.contains("$1"));
        assert!(!text.contains("$2"));
        assert_eq!(result.launch_app, None);
        assert!(text.lines().all(|line| line.chars().count() <= 132));
    }

    #[test]
    fn neofetch_plain_output_tracks_packages_disk_memory_network_and_environment() {
        let mut world = WorldState::new("neo", "pc").unwrap();
        let before = execute(&mut world, "neofetch --stdout").stdout;
        let count = world
            .packages
            .installed
            .values()
            .filter(|p| p.status == Status::Installed)
            .count();
        world.packages.installed.get_mut("nano").unwrap().status = Status::ConfigFiles;
        world.network.connected = false;
        world
            .terminal
            .env
            .insert("LANG".into(), "pt_BR.UTF-8".into());
        world.vfs.capacity_bytes = 12 * 1024 * 1024 * 1024;
        world
            .vfs
            .write("/home/kali/test.txt", &"x".repeat(1024 * 1024), "kali")
            .unwrap();
        let after = execute(&mut world, "neofetch --stdout").stdout;
        assert_ne!(before, after);
        assert!(after.contains(&format!("Packages: {} (dpkg)", count - 1)));
        assert!(after.contains("Local IP (eth0): desconectada"));
        assert!(after.contains("Locale: pt_BR.UTF-8"));
        assert!(after.contains("/ 12.00 GiB"));
        let resources = crate::task_manager::snapshot(&world);
        assert!(after.contains(&format!(
            "Memory: {}",
            usage(resources.memory_used_bytes, resources.memory_total_bytes)
        )));
        assert!(!after.contains('\x1b'));
        assert!(!after.contains("0Xxoc"));
        let piped = execute(&mut world, "fastfetch --pipe | grep '^Packages:'");
        assert_eq!(piped.stdout, format!("Packages: {} (dpkg)\n", count - 1));
        assert_eq!(
            execute(&mut world, "fastfetch --pipe > Documents/system.txt").exit_code,
            0
        );
        assert!(world
            .vfs
            .read("/home/kali/Documents/system.txt", "kali")
            .unwrap()
            .starts_with("kali@pc\n"));
    }

    #[test]
    fn fastfetch_handles_narrow_terminals_and_resets_ansi() {
        let mut world = WorldState::new("neo", "pc").unwrap();
        for columns in [20, 48, 60, 80, 100, 132] {
            world.terminal.presentation.columns = columns;
            let result = execute(&mut world, "fastfetch");
            assert_eq!(result.exit_code, 0);
            assert!(result.stdout.ends_with(RESET));
            assert!(
                plain(&result.stdout)
                    .lines()
                    .all(|line| line.chars().count() <= columns),
                "width {columns}"
            );
        }
        assert!(!execute(&mut world, "fastfetch --logo none")
            .stdout
            .contains("0Xxoc"));
        assert_ne!(execute(&mut world, "fastfetch --bad-option").exit_code, 0);
        assert!(execute(&mut world, "fastfetch --help")
            .stdout
            .contains("Uso:"));
        assert!(execute(&mut world, "man neofetch")
            .stdout
            .contains("computador virtual"));
        assert_eq!(
            completion::complete(&world, "fastf", 5).unwrap().line,
            "fastfetch "
        );
    }

    #[test]
    fn fastfetch_respects_root_ssh_and_per_window_presentation() {
        let mut world = WorldState::new("neo", "pc").unwrap();
        assert!(execute(&mut world, "sudo fastfetch --pipe")
            .stdout
            .starts_with("root@pc\n"));
        assert_eq!(world.terminal.user, "kali");
        terminal_sessions::open(&mut world, "A").unwrap();
        terminal_sessions::open(&mut world, "B").unwrap();
        terminal_sessions::with_session(&mut world, Some("A"), |w| {
            w.terminal.presentation.columns = 60;
            assert_eq!(execute(w, "ssh vex@vex.local lab-only").exit_code, 0);
            let output = execute(w, "fastfetch --pipe").stdout;
            assert!(output.starts_with("vex@vex.local\n"));
            assert!(!output.contains("DE:"));
            assert!(output.contains("não disponível nesta sessão SSH"));
            Ok(())
        })
        .unwrap();
        assert_eq!(world.terminal_sessions["A"].presentation.columns, 60);
        assert_eq!(world.terminal_sessions["B"].presentation.columns, 100);
        assert!(world.terminal.host.is_none());
        assert!(!serde_json::to_string(&world)
            .unwrap()
            .contains("presentation"));
    }

    #[test]
    fn fastfetch_does_not_execute_escape_sequences_from_virtual_files() {
        let mut world = WorldState::new("neo", "pc").unwrap();
        world
            .vfs
            .write(
                "/etc/os-release",
                "PRETTY_NAME=\"Custom\x1b[2J Linux\"\n",
                "root",
            )
            .unwrap();
        let output = execute(&mut world, "fastfetch --pipe").stdout;
        assert!(!output.contains('\x1b'));
        assert!(output.contains("Custom[2J Linux"));
    }
}
