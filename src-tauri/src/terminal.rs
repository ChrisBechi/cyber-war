use crate::{
    error::GameResult,
    vfs::{domain, normalize, HOME},
    world::WorldState,
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

pub const COMMANDS: &[&str] = &[
    "jobs",
    "zip",
    "unzip",
    "tar",
    "gzip",
    "gunzip",
    "zcat",
    "bzip2",
    "bunzip2",
    "xz",
    "unxz",
    "help",
    "pwd",
    "cd",
    "echo",
    "clear",
    "sh",
    "bash",
    "source",
    "true",
    "false",
    "yes",
    "whoami",
    "id",
    "groups",
    "uname",
    "fastfetch",
    "neofetch",
    "hostname",
    "date",
    "env",
    "printenv",
    "export",
    "ls",
    "mkdir",
    "rmdir",
    "touch",
    "cat",
    "head",
    "tail",
    "wc",
    "sort",
    "uniq",
    "cut",
    "tr",
    "seq",
    "cp",
    "mv",
    "rm",
    "basename",
    "dirname",
    "realpath",
    "readlink",
    "ln",
    "which",
    "type",
    "command",
    "tee",
    "whereis",
    "df",
    "du",
    "tree",
    "stat",
    "chmod",
    "chown",
    "sudo",
    "su",
    "grep",
    "find",
    "less",
    "edit",
    "nano",
    "file",
    "strings",
    "base64",
    "sha256sum",
    "man",
    "apt",
    "apt-get",
    "apt-cache",
    "apt-mark",
    "dpkg",
    "dpkg-query",
    "ip",
    "ifconfig",
    "lsblk",
    "ipconfig",
    "ping",
    "route",
    "arp",
    "ss",
    "netstat",
    "dig",
    "nslookup",
    "whois",
    "traceroute",
    "curl",
    "wget",
    "ssh",
    "scp",
    "nmap",
    "ps",
    "top",
    "kill",
    "killall",
    "pkill",
    "free",
    "uptime",
    "service",
    "systemctl",
    "journalctl",
    "sector-ix",
    "exit",
    "lab",
];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandResult {
    pub termination: crate::shell::signals::Termination,
    #[serde(skip)]
    #[allow(dead_code)]
    // Raw command transport is inspected by development capture, never rendered as text.
    pub stdout_bytes: Vec<u8>,
    #[serde(skip)]
    #[allow(dead_code)]
    pub stderr_bytes: Vec<u8>,
    pub shell_incomplete: bool,
    pub(crate) ordered: Vec<(u8, String)>,
    pub stdout: String,
    pub stderr: String,
    pub cwd: String,
    pub user: String,
    pub host: String,
    pub exit_code: i32,
    pub interactive: Option<crate::nano::NanoLaunch>,
    pub launch_app: Option<String>,
    pub archive_prompt: Option<crate::archive::cli::Prompt>,
    pub archive_job: Option<u32>,
}

/// Only lexical parsing. No shell, interpolation, executable lookup or host fallback.
#[cfg(test)]
pub fn tokenize(input: &str) -> GameResult<Vec<String>> {
    if input.len() > 8192 {
        return Err(domain("command too long"));
    }
    let mut tokens = Vec::new();
    let mut token = String::new();
    let mut quote = None;
    let mut started = false;
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c.is_control() {
            return Err(domain("control characters are not supported"));
        }
        if let Some(q) = quote {
            if c == q {
                quote = None;
            } else {
                token.push(c);
            }
        } else {
            match c {
                '\'' | '"' => {
                    quote = Some(c);
                    started = true;
                }
                ';' | '|' | '&' | '`' | '$' | '<' => {
                    return Err(domain("shell operators and expansion are not supported"))
                }
                '>' => {
                    if started {
                        tokens.push(std::mem::take(&mut token));
                        started = false;
                    }
                    if chars.peek() == Some(&'>') {
                        chars.next();
                        tokens.push("\0>>".into());
                    } else {
                        tokens.push("\0>".into());
                    }
                }
                c if c.is_whitespace() => {
                    if started {
                        tokens.push(std::mem::take(&mut token));
                        started = false;
                    }
                }
                c => {
                    token.push(c);
                    started = true;
                }
            }
        }
    }
    if quote.is_some() {
        return Err(domain("unterminated quote"));
    }
    if started {
        tokens.push(token);
    }
    Ok(tokens)
}

pub fn execute_interactive(world: &mut WorldState, line: &str) -> CommandResult {
    if line == "\x03" {
        world.terminal.shell.continuation.clear();
        return execute_with(world, |_| {
            Ok(crate::terminal_io::Output {
                status: 130,
                ..Default::default()
            })
        });
    }
    let previous = std::mem::take(&mut world.terminal.shell.continuation);
    let command = if previous.is_empty() {
        line.into()
    } else {
        format!("{previous}\n{line}")
    };
    if matches!(
        crate::shell::syntax::parse(&command),
        crate::shell::syntax::ParseResult::Incomplete(_)
    ) {
        world.terminal.shell.continuation = command;
        let status = world.terminal.last_status;
        let mut result = execute_with(world, |_| {
            Ok(crate::terminal_io::Output {
                status,
                ..Default::default()
            })
        });
        result.shell_incomplete = true;
        return result;
    }
    world.terminal.shell.flow = None;
    execute(world, &command)
}

pub fn execute(world: &mut WorldState, command: &str) -> CommandResult {
    if !command.trim().is_empty() {
        world.terminal.history.push(command.into());
        if world.terminal.history.len() > 500 {
            world.terminal.history.remove(0);
        }
    }
    let command_name = command.split_whitespace().next().unwrap_or("").to_string();
    let mut result = execute_with(world, |candidate| crate::shell::execute(candidate, command));
    if result.exit_code == 0 && command_name == "sector-ix" {
        result.launch_app = Some("vigilia".into());
    }
    result
}

pub(crate) fn execute_parts(
    world: &mut WorldState,
    parts: GameResult<Vec<String>>,
) -> CommandResult {
    execute_with(world, |candidate| {
        parts.and_then(|parts| crate::shell_pipeline::run(candidate, &parts))
    })
}
fn execute_with(
    world: &mut WorldState,
    work: impl FnOnce(&mut WorldState) -> GameResult<crate::terminal_io::Output>,
) -> CommandResult {
    if world.terminal.shell.environment_order.is_empty() {
        world.terminal.shell.environment_order = virtual_env(world, &world.terminal.user)
            .into_keys()
            .collect();
    }
    let mut candidate = world.clone();
    let outcome = work(&mut candidate).map(|mut output| {
        if let Some(signal) = crate::shell::control::termination_signal() {
            let termination = crate::shell::signals::Termination::Signal { signal };
            output.status = termination.status();
            output.termination = Some(termination);
        }
        output
    });
    let (stdout, stderr, code, archive_job, ordered, stdout_bytes, stderr_bytes, termination) =
        match outcome {
            Ok(out) => {
                // Process umask belongs to the shell context, not the shared desktop VFS.
                candidate.vfs.umask = world.vfs.umask;
                for (id, host) in &mut candidate.network.hosts {
                    host.files.umask = world.network.hosts.get(id).map_or(0o022, |h| h.files.umask);
                }
                if candidate.terminal.cwd != world.terminal.cwd
                    || candidate.terminal.host != world.terminal.host
                {
                    candidate
                        .terminal
                        .env
                        .insert("PWD".into(), candidate.terminal.cwd.clone());
                }
                if candidate.terminal.host != world.terminal.host {
                    // A previous local directory is not the previous directory of the remote shell.
                    candidate.terminal.env.remove("OLDPWD");
                }
                observe(&mut candidate);
                *world = candidate;
                let raw = out.binary.unwrap_or_else(|| out.stdout.as_bytes().to_vec());
                let raw_error = if out.byte_ordered.is_empty() {
                    out.stderr.as_bytes().to_vec()
                } else {
                    out.byte_ordered
                        .iter()
                        .filter(|(fd, _)| *fd == 2)
                        .flat_map(|(_, bytes)| bytes.iter().copied())
                        .collect()
                };
                (
                    out.stdout,
                    out.stderr,
                    out.status,
                    out.archive_job,
                    out.ordered,
                    raw,
                    raw_error,
                    out.termination
                        .unwrap_or(crate::shell::signals::Termination::Exit { code: out.status }),
                )
            }
            Err(e) => {
                let text = e.to_string();
                (
                    String::new(),
                    format!("{text}\n"),
                    2,
                    None,
                    Vec::new(),
                    Vec::new(),
                    format!("{text}\n").into_bytes(),
                    crate::shell::signals::Termination::Exit { code: 2 },
                )
            }
        };
    world.terminal.last_status = code;
    CommandResult {
        termination,
        stdout_bytes,
        stderr_bytes,
        shell_incomplete: false,
        ordered,
        stdout,
        stderr,
        cwd: world.terminal.cwd.clone(),
        user: world.terminal.user.clone(),
        host: world
            .terminal
            .host
            .clone()
            .unwrap_or_else(|| world.hostname.clone()),
        exit_code: code,
        interactive: if code == 0 {
            world
                .terminal
                .nano
                .as_ref()
                .map(|session| crate::nano::NanoLaunch {
                    path: session.path.clone(),
                    display_name: session.display_name.clone(),
                    content: if session.options.no_read {
                        String::new()
                    } else {
                        session.original_content.clone().unwrap_or_default()
                    },
                    expected_content: session.original_content.clone(),
                    options: session.options.clone(),
                    starting_line: session.starting_line,
                    starting_column: session.starting_column,
                })
        } else {
            None
        },
        launch_app: None,
        archive_prompt: world
            .terminal
            .archive_pending
            .as_ref()
            .map(|p| p.prompt.clone())
            .or_else(|| {
                world
                    .terminal
                    .package_pending
                    .as_ref()
                    .map(|_| crate::archive::cli::Prompt {
                        message: "Do you want to continue? [Y/n] ".into(),
                        secret: false,
                    })
            }),
        archive_job,
    }
}

fn arg(args: &[String], i: usize, usage: &str) -> GameResult<String> {
    args.get(i)
        .cloned()
        .ok_or_else(|| domain(format!("usage: {usage}")))
}
fn path(world: &WorldState, value: &str) -> GameResult<String> {
    normalize(value, &world.terminal.cwd)
}

fn has_flag(args: &[String], short: char, long: &str) -> bool {
    args.iter().any(|arg| {
        arg == long
            || (arg.starts_with('-')
                && !arg.starts_with("--")
                && arg.chars().skip(1).any(|flag| flag == short))
    })
}

fn is_short_flag_cluster(value: &str, allowed: &str) -> bool {
    value.starts_with('-')
        && !value.starts_with("--")
        && value.len() > 1
        && value.chars().skip(1).all(|flag| allowed.contains(flag))
}

fn operands(args: &[String]) -> Vec<String> {
    let mut after_options = false;
    args.iter()
        .filter_map(|arg| {
            if arg == "--" {
                after_options = true;
                return None;
            }
            if after_options || !arg.starts_with('-') || arg == "-" {
                Some(arg.clone())
            } else {
                None
            }
        })
        .collect()
}

fn operands_with_values(args: &[String], value_options: &[&str]) -> Vec<String> {
    let mut after_options = false;
    let mut skip_next = false;
    args.iter()
        .filter_map(|arg| {
            if skip_next {
                skip_next = false;
                return None;
            }
            if arg == "--" {
                after_options = true;
                return None;
            }
            if !after_options
                && value_options.iter().any(|option| {
                    arg == option
                        || (option.len() == 2
                            && arg.starts_with(option)
                            && arg.len() > option.len())
                })
            {
                if value_options.iter().any(|option| arg == *option) {
                    skip_next = true;
                }
                return None;
            }
            if after_options || !arg.starts_with('-') || arg == "-" {
                Some(arg.clone())
            } else {
                None
            }
        })
        .collect()
}

fn option_value(
    args: &[String],
    short: char,
    long: &str,
    usage: &str,
) -> GameResult<Option<String>> {
    for (index, arg) in args.iter().enumerate() {
        if arg == long || arg == &format!("-{short}") {
            return args
                .get(index + 1)
                .cloned()
                .map(Some)
                .ok_or_else(|| domain(format!("usage: {usage}")));
        }
        if let Some(value) = arg.strip_prefix(&format!("{long}=")) {
            return Ok(Some(value.into()));
        }
        if let Some(value) = arg.strip_prefix(&format!("-{short}")) {
            if !value.is_empty() {
                return Ok(Some(value.into()));
            }
        }
    }
    Ok(None)
}

fn chmod_mode(spec: &str, current: u16) -> GameResult<u16> {
    if spec.chars().all(|value| ('0'..='7').contains(&value)) && !spec.is_empty() {
        return u16::from_str_radix(spec, 8).map_err(|_| domain("chmod: invalid mode"));
    }
    let (classes, operation, permissions) = if let Some((left, right)) = spec.split_once('+') {
        (left, '+', right)
    } else if let Some((left, right)) = spec.split_once('-') {
        (left, '-', right)
    } else if let Some((left, right)) = spec.split_once('=') {
        (left, '=', right)
    } else {
        return Err(domain("chmod: invalid mode"));
    };
    if permissions.is_empty()
        || !permissions
            .chars()
            .all(|value| matches!(value, 'r' | 'w' | 'x'))
        || !classes
            .chars()
            .all(|value| matches!(value, 'u' | 'g' | 'o' | 'a'))
    {
        return Err(domain("chmod: invalid mode"));
    }
    let classes = if classes.is_empty() { "a" } else { classes };
    let mut mask = 0;
    for class in classes.chars() {
        let shift = match class {
            'u' => 6,
            'g' => 3,
            'o' => 0,
            'a' => 0,
            _ => unreachable!(),
        };
        for permission in permissions.chars() {
            let bit = match permission {
                'r' => 4,
                'w' => 2,
                'x' => 1,
                _ => unreachable!(),
            };
            if class == 'a' {
                mask |= bit << 6 | bit << 3 | bit;
            } else {
                mask |= bit << shift;
            }
        }
    }
    match operation {
        '+' => Ok(current | mask),
        '-' => Ok(current & !mask),
        '=' => {
            let class_mask = classes.chars().fold(0, |value, class| {
                value
                    | match class {
                        'u' => 0o700,
                        'g' => 0o070,
                        'o' => 0o007,
                        'a' => 0o777,
                        _ => 0,
                    }
            });
            Ok((current & !class_mask) | mask)
        }
        _ => unreachable!(),
    }
}

fn service_defaults() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("apache2".into(), "inactive".into()),
        ("cron".into(), "inactive".into()),
        ("networking".into(), "inactive".into()),
        ("postgresql".into(), "inactive".into()),
        ("ssh".into(), "inactive".into()),
        ("web".into(), "inactive".into()),
    ])
}

fn service_states(world: &WorldState) -> BTreeMap<String, String> {
    world
        .settings
        .get("services")
        .and_then(|value| serde_json::from_str(value).ok())
        .unwrap_or_else(service_defaults)
}

fn save_service_states(
    world: &mut WorldState,
    states: &BTreeMap<String, String>,
) -> GameResult<()> {
    world
        .settings
        .insert("services".into(), serde_json::to_string(states)?);
    Ok(())
}

fn service_name(value: &str) -> String {
    value.strip_suffix(".service").unwrap_or(value).to_owned()
}

fn enabled_services(world: &WorldState) -> BTreeSet<String> {
    world
        .settings
        .get("servicesEnabled")
        .and_then(|value| serde_json::from_str::<BTreeSet<String>>(value).ok())
        .unwrap_or_default()
}

fn service_operation(
    world: &mut WorldState,
    name: &str,
    operation: &str,
    actor: &str,
) -> GameResult<String> {
    let name = service_name(name);
    if let Some(host_key) = world.terminal.host.clone() {
        if name != "web" && name != "apache2" {
            return Err(domain("unknown service on virtual host"));
        }
        let (running, health_checked) = {
            let host = world
                .network
                .hosts
                .get_mut(&host_key)
                .ok_or_else(|| domain("host missing"))?;
            let restart_enabled = if operation == "restart" {
                host.files
                    .read("/etc/web.conf", actor)
                    .is_ok_and(|content| content.contains("enabled=true"))
            } else {
                false
            };
            let running = {
                let web = host
                    .services
                    .iter_mut()
                    .find(|service| service.port == 80)
                    .ok_or_else(|| domain("service missing"))?;
                if matches!(operation, "start" | "stop" | "restart") {
                    if actor != "root" {
                        return Err(domain("permission denied: use sudo"));
                    }
                    web.running = match operation {
                        "stop" => false,
                        "start" => true,
                        _ => restart_enabled,
                    };
                } else if !matches!(operation, "status" | "is-active") {
                    return Err(domain("unknown service operation"));
                }
                web.running
            };
            let health_checked = running
                && host
                    .files
                    .read("/srv/www/health.txt", actor)
                    .is_ok_and(|content| content.contains("OK"));
            (running, health_checked)
        };
        if health_checked {
            world.flags.insert("VEX_HEALTH_CHECKED".into());
        }
        if matches!(operation, "start" | "stop" | "restart") {
            world.events.push(format!(
                "service|{host_key}|{name}|{}|{operation}",
                world.playtime_seconds
            ));
        }
        return Ok(format!(
            "{}: {}\n",
            name,
            if running { "active" } else { "inactive" }
        ));
    }
    let mut states = service_states(world);
    let state = states
        .get(&name)
        .cloned()
        .ok_or_else(|| domain(format!("unknown service: {name}")))?;
    if matches!(operation, "start" | "stop" | "restart") {
        if actor != "root" {
            return Err(domain("permission denied: use sudo"));
        }
        if name == "networking" && operation != "stop" && !world.network.connected {
            return Err(domain("networking: virtual link is disconnected"));
        }
        states.insert(
            name.clone(),
            if operation == "stop" {
                "inactive".into()
            } else {
                "active".into()
            },
        );
        save_service_states(world, &states)?;
        sync_service_process(world, &name, operation != "stop");
        world.events.push(format!(
            "service|local|{name}|{}|{operation}",
            world.playtime_seconds
        ));
    } else if !matches!(operation, "status" | "is-active" | "is-enabled") {
        return Err(domain("unknown service operation"));
    }
    if operation == "is-enabled" {
        let enabled = enabled_services(world);
        return Ok(format!(
            "{}\n",
            if enabled.contains(&name) {
                "enabled"
            } else {
                "disabled"
            }
        ));
    }
    let final_state = states.get(&name).map(String::as_str).unwrap_or(&state);
    Ok(format!(
        "{}: {}\n",
        name,
        if final_state == "active" {
            "active"
        } else {
            "inactive"
        }
    ))
}

fn sync_service_process(world: &mut WorldState, name: &str, running: bool) {
    for process in world
        .processes
        .iter_mut()
        .filter(|p| p.name == name && p.user == "root")
    {
        process.running = false;
    }
    if running {
        let pid = world
            .processes
            .iter()
            .map(|p| p.pid)
            .max()
            .unwrap_or(99)
            .saturating_add(1)
            .max(100);
        world.processes.push(crate::world::VirtualProcess {
            pid,
            name: name.into(),
            user: "root".into(),
            running,
        });
    }
}

fn service_contract(
    world: &mut WorldState,
    name: &str,
    args: &[String],
    actor: &str,
) -> GameResult<crate::terminal_io::Output> {
    use crate::terminal_io::{options, Output};
    let opts = options(
        name,
        args,
        "q",
        "",
        &[
            ("quiet", 'q'),
            ("no-pager", 'p'),
            ("now", 'N'),
            ("status-all", 's'),
            ("version", 'V'),
        ],
    )?;
    if opts.help {
        return Ok(Output::success(format!("{name} — virtual services\nservice UNIT start|stop|restart|status; service --status-all\nsystemctl status|is-active|is-enabled|start|stop|restart|enable|disable UNIT\nsystemctl list-units|list-unit-files; enable/disable --now; --quiet; --no-pager\nOnly built-in units. No arbitrary units, daemon-reload, remote enablement or real daemons.\n")));
    }
    if opts.has('V') {
        return Ok(Output::success(
            "Cyber War virtual service manager (not systemd)\n".into(),
        ));
    }
    let mut values = opts.files.clone();
    if name == "systemctl" && opts.has('s') {
        return Err(domain("systemctl: --status-all is a service option"));
    }
    if name == "service" && opts.has('s') && values.is_empty() {
        return Ok(Output::success(if world.terminal.host.is_some() {
            return Err(domain("service: remote status-all not implemented"));
        } else {
            list_services(world)
        }));
    }
    if name == "service" && values.len() != 2 {
        return Err(domain("usage: service UNIT OPERATION"));
    }
    if name == "systemctl" && values.is_empty() {
        values.push("list-units".into());
    }
    let operation = if name == "service" {
        values[1].as_str()
    } else {
        values[0].as_str()
    };
    let unit = if name == "service" {
        Some(values[0].as_str())
    } else {
        values.get(1).map(String::as_str)
    };
    let list = matches!(operation, "list-units" | "list-unit-files");
    if (!list && unit.is_none()) || (list && values.len() != 1) || values.len() > 2 {
        return Err(domain("service subset requires one operation and one unit"));
    }
    if opts.has('N') && (name != "systemctl" || !matches!(operation, "enable" | "disable")) {
        return Err(domain(
            "--now is only implemented for systemctl enable/disable",
        ));
    }
    if let Some(unit) = unit {
        let key = service_name(unit);
        if world.terminal.host.is_none() && !service_states(world).contains_key(&key) {
            return Err(domain(format!("unknown service: {key}")));
        }
    }
    if world.terminal.host.is_some()
        && (list || matches!(operation, "enable" | "disable" | "is-enabled"))
    {
        return Err(domain(
            "remote service enablement/listing is not implemented",
        ));
    }
    let mut stdout = if name == "service" {
        service_operation(world, &values[0], &values[1], actor)?
    } else {
        systemctl_command(world, &values, actor)?
    };
    if opts.has('N') {
        service_operation(
            world,
            unit.unwrap(),
            if operation == "enable" {
                "start"
            } else {
                "stop"
            },
            actor,
        )?;
    }
    let active = stdout.contains(": active") || stdout.contains("Active: active");
    let status = if matches!(operation, "status" | "is-active") && !active && !list {
        3
    } else if operation == "is-enabled" && stdout.trim() == "disabled" {
        1
    } else {
        0
    };
    if operation == "is-active" {
        stdout = if active { "active\n" } else { "inactive\n" }.into();
    }
    if opts.has('q') {
        stdout.clear();
    }
    Ok(Output {
        stdout,
        status,
        ..Output::default()
    })
}

fn process_contract(
    world: &mut WorldState,
    name: &str,
    args: &[String],
    actor: &str,
) -> GameResult<crate::terminal_io::Output> {
    use crate::terminal_io::Output;
    if args == ["--help"] {
        return Ok(Output::success("Virtual processes: ps [-e|-A|-ef|aux] [-p PID]; kill [-0|-9|-15|-TERM|-KILL] PID...; killall [-0|-9|-15] NAME; pkill [-0|-9|-15] [-x] REGEX\nOnly local running processes. TERM/KILL terminate; signal 0 checks existence/permission. No process groups, remote process model, other signals or full argv matching.\n".into()));
    }
    if world.terminal.host.is_some() {
        return Err(domain(
            "remote process table is not implemented; local processes are unchanged",
        ));
    }
    if name == "ps" {
        let mut all = false;
        let mut full = false;
        let mut pid = None;
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "-e" | "-A" => all = true,
                "aux" | "-ef" => {
                    all = true;
                    full = true;
                }
                "-f" => full = true,
                "-p" => {
                    i += 1;
                    pid = Some(
                        args.get(i)
                            .ok_or_else(|| domain("ps: PID required"))?
                            .parse::<u32>()
                            .map_err(|_| domain("ps: invalid PID"))?,
                    );
                }
                other => return Err(domain(format!("ps: unsupported option {other}"))),
            }
            i += 1;
        }
        let mut out = if full {
            "UID          PID TTY          TIME CMD\n".to_owned()
        } else {
            "    PID TTY          TIME CMD\n".to_owned()
        };
        let mut found = false;
        // ps itself is a foreground virtual command, even if no persistent
        // process belongs to this terminal user. It is not saved as a daemon.
        if pid.is_none() {
            let current_pid = world
                .processes
                .iter()
                .map(|p| p.pid)
                .max()
                .unwrap_or(1)
                .saturating_add(1);
            if full {
                out.push_str(&format!("{actor:<10} "));
            }
            out.push_str(&format!("{current_pid:>5} pts/0    00:00:00 ps\n"));
            found = true;
        }
        for p in world
            .processes
            .iter()
            .filter(|p| p.running && pid.map_or(all || p.user == actor, |pid| p.pid == pid))
        {
            found = true;
            if full {
                out.push_str(&format!("{:<10} ", p.user));
            }
            out.push_str(&format!("{:>5} pts/0    00:00:00 {}\n", p.pid, p.name));
        }
        return Ok(Output {
            stdout: out,
            status: if found { 0 } else { 1 },
            ..Output::default()
        });
    }
    let mut signal = 15;
    let mut exact = false;
    let mut operands = Vec::new();
    let mut end = false;
    for arg in args {
        if end {
            operands.push(arg);
            continue;
        }
        match arg.as_str() {
            "--" => end = true,
            "-0" => signal = 0,
            "-9" | "-KILL" | "-SIGKILL" => signal = 9,
            "-15" | "-TERM" | "-SIGTERM" => signal = 15,
            "-x" if name == "pkill" => exact = true,
            a if a.starts_with('-') => {
                return Err(domain(format!("{name}: unsupported signal/option {a}")))
            }
            _ => operands.push(arg),
        }
    }
    if operands.is_empty() || (name != "kill" && operands.len() != 1) {
        return Err(domain(format!("{name}: target required")));
    }
    let pids: Vec<u32> = if name == "kill" {
        operands
            .iter()
            .map(|p| p.parse().map_err(|_| domain("kill: invalid PID")))
            .collect::<GameResult<_>>()?
    } else {
        Vec::new()
    };
    let matcher = if name == "pkill" {
        Some(
            regex::Regex::new(&if exact {
                format!("^(?:{})$", operands[0])
            } else {
                operands[0].to_string()
            })
            .map_err(|_| domain("pkill: invalid regex"))?,
        )
    } else {
        None
    };
    let targets: Vec<_> = world
        .processes
        .iter()
        .filter(|p| {
            p.running
                && match name {
                    "kill" => pids.contains(&p.pid),
                    "killall" => p.name == *operands[0],
                    _ => matcher.as_ref().is_some_and(|r| r.is_match(&p.name)),
                }
        })
        .map(|p| (p.pid, p.name.clone(), p.user.clone()))
        .collect();
    if targets.is_empty()
        || (name == "kill"
            && pids
                .iter()
                .any(|id| !targets.iter().any(|(pid, _, _)| pid == id)))
    {
        return Err(domain(format!("{name}: no such process")));
    }
    signal_local_targets(world, targets, actor, signal)?;
    Ok(Output::default())
}

pub(crate) fn signal_local_process(
    world: &mut WorldState,
    pid: u32,
    actor: &str,
    signal: u8,
) -> GameResult<()> {
    let p = world
        .processes
        .iter()
        .find(|p| p.pid == pid && p.running)
        .ok_or_else(|| domain("kill: no such process"))?;
    signal_local_targets(
        world,
        vec![(p.pid, p.name.clone(), p.user.clone())],
        actor,
        signal,
    )
}

fn signal_local_targets(
    world: &mut WorldState,
    targets: Vec<(u32, String, String)>,
    actor: &str,
    signal: u8,
) -> GameResult<()> {
    if ![0, 9, 15].contains(&signal) {
        return Err(domain("kill: unsupported signal"));
    }
    for (pid, _, user) in &targets {
        if (*pid == 1 && signal != 0) || (user != actor && actor != "root") {
            return Err(domain("permission denied"));
        }
    }
    if signal != 0 {
        let mut states = service_states(world);
        for (pid, unit, _) in targets {
            if let Some(process) = world.processes.iter_mut().find(|p| p.pid == pid) {
                process.running = false;
            }
            if states.contains_key(&unit) {
                states.insert(unit.clone(), "inactive".into());
                world.events.push(format!(
                    "service|local|{unit}|{}|signal {signal}",
                    world.playtime_seconds
                ));
            }
        }
        save_service_states(world, &states)?;
    }
    Ok(())
}

fn list_services(world: &WorldState) -> String {
    service_states(world)
        .iter()
        .map(|(name, state)| {
            format!(
                "{}.service - Virtual {} service\n  Active: {}\n",
                name, name, state
            )
        })
        .collect()
}

fn service_command(world: &mut WorldState, args: &[String], actor: &str) -> GameResult<String> {
    if has_flag(args, 's', "--status-all") {
        return Ok(list_services(world));
    }
    let name = arg(args, 0, "service SERVICE COMMAND")?;
    let operation = args.get(1).map(String::as_str).unwrap_or("status");
    service_operation(world, &name, operation, actor)
}

fn systemctl_command(world: &mut WorldState, args: &[String], actor: &str) -> GameResult<String> {
    if has_flag(args, 'v', "--version") {
        return Ok("systemd 257.5 (virtual)\n+PAM +AUDIT +SELINUX +APPARMOR\n".into());
    }
    if args.iter().any(|arg| arg == "--failed") {
        return Ok("0 loaded units listed.\n".into());
    }
    let values = operands(args);
    let operation = values.first().map(String::as_str).unwrap_or("list-units");
    let unit = values.get(1).map(String::as_str);
    match operation {
        "list-units" | "list-unit-files" => Ok(list_services(world)),
        "daemon-reload" => Err(domain("systemctl: daemon-reload is not implemented; virtual units are built into this version")),
        "status" => {
            if let Some(unit) = unit {
                let concise = service_operation(world, unit, "status", actor)?;
                let name = service_name(unit);
                let status = concise
                    .split_once(':')
                    .map(|(_, value)| value.trim())
                    .unwrap_or("unknown");
                let enabled = if enabled_services(world).contains(&name) {
                    "enabled"
                } else {
                    "disabled"
                };
                let activity = if status == "active" {
                    "running"
                } else {
                    "dead"
                };
                Ok(format!("● {name}.service - Virtual {name} service\n   Loaded: loaded (/etc/systemd/system/{name}.service; {enabled})\n   Active: {status} ({activity})\n"))
            } else {
                Ok(list_services(world))
            }
        }
        "is-active" | "is-enabled" | "start" | "stop" | "restart" | "enable" | "disable" => {
            let unit = unit
                .ok_or_else(|| domain(format!("systemctl {operation}: unit name is required")))?;
            if matches!(operation, "enable" | "disable") {
                if actor != "root" {
                    return Err(domain("permission denied: use sudo"));
                }
                let mut enabled = enabled_services(world);
                let name = service_name(unit);
                if operation == "enable" {
                    enabled.insert(name.clone());
                } else {
                    enabled.remove(&name);
                }
                world
                    .settings
                    .insert("servicesEnabled".into(), serde_json::to_string(&enabled)?);
                return Ok(String::new());
            }
            service_operation(world, unit, operation, actor)
        }
        _ => Err(domain(format!(
            "systemctl: unknown operation '{operation}'"
        ))),
    }
}

/// Starts only services explicitly enabled by the player for the next login.
/// Runtime service state is intentionally separate from the enablement policy:
/// a stopped service can be saved, while a new login still follows systemd's
/// normal enable/disable rule.
pub fn prepare_session_start(world: &mut WorldState) -> GameResult<()> {
    crate::terminal_sessions::reset(world);
    world.processes = vec![crate::world::VirtualProcess {
        pid: 1,
        name: "cyber-war-session".into(),
        user: "kali".into(),
        running: true,
    }];
    let enabled = enabled_services(world);
    let mut states = service_states(world);
    for state in states.values_mut() {
        *state = "inactive".into();
    }
    for service in enabled {
        if service == "networking" && !world.network.connected {
            continue;
        }
        if states.contains_key(&service) {
            states.insert(service, "active".into());
        }
    }
    for (index, (name, _)) in states
        .iter()
        .filter(|(_, state)| *state == "active")
        .enumerate()
    {
        world.processes.push(crate::world::VirtualProcess {
            pid: 100 + index as u32,
            name: name.clone(),
            user: "root".into(),
            running: true,
        });
    }
    save_service_states(world, &states)
}

pub fn stop_session_runtime(world: &mut WorldState) -> GameResult<()> {
    crate::terminal_sessions::reset(world);
    world.processes.clear();
    save_service_states(world, &service_defaults())
}

fn journalctl_command(world: &WorldState, args: &[String]) -> GameResult<String> {
    let opts = crate::terminal_io::options(
        "journalctl",
        args,
        "q",
        "un",
        &[
            ("unit", 'u'),
            ("lines", 'n'),
            ("no-pager", 'p'),
            ("quiet", 'q'),
        ],
    )?;
    if opts.help {
        return Ok("journalctl [-u UNIT] [-n N] [--no-pager]\nOnly recorded virtual service operations for this host. No follow, date filters or fabricated health logs.\n".into());
    }
    if !opts.files.is_empty() {
        return Err(domain("journalctl: unsupported positional filter"));
    }
    let unit = opts
        .counts
        .iter()
        .rev()
        .find(|(f, _)| *f == 'u')
        .map(|(_, v)| service_name(v));
    let count = opts
        .counts
        .iter()
        .rev()
        .find(|(f, _)| *f == 'n')
        .map(|(_, v)| {
            v.parse::<usize>()
                .map_err(|_| domain("journalctl: invalid line count"))
        })
        .transpose()?
        .unwrap_or(100);
    let host = world.terminal.host.as_deref().unwrap_or("local");
    let entries: Vec<String> = world
        .events
        .iter()
        .filter_map(|e| {
            let fields: Vec<_> = e.split('|').collect();
            if fields.len() != 5
                || fields[0] != "service"
                || fields[1] != host
                || unit.as_ref().is_some_and(|u| u != fields[2])
            {
                return None;
            }
            Some(format!(
                "[{}] {} {}.service: {}\n",
                fields[3], fields[1], fields[2], fields[4]
            ))
        })
        .collect();
    Ok(entries[entries.len().saturating_sub(count)..].concat())
}

pub(crate) fn virtual_env(world: &WorldState, actor: &str) -> BTreeMap<String, String> {
    let mut env = BTreeMap::from([
        (
            "HOME".into(),
            if actor == "root" {
                "/root".into()
            } else {
                HOME.into()
            },
        ),
        ("HOSTNAME".into(), world.hostname.clone()),
        ("LANG".into(), "C.UTF-8".into()),
        ("LOGNAME".into(), actor.into()),
        (
            "PATH".into(),
            "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".into(),
        ),
        ("PWD".into(), world.terminal.cwd.clone()),
        ("SHELL".into(), "/bin/bash".into()),
        ("USER".into(), actor.into()),
    ]);
    env.extend(world.terminal.env.clone());
    for key in &world.terminal.shell.unset {
        env.remove(key);
    }
    env
}

fn manual_page(command: &str) -> String {
    if command == "cat" {
        return crate::terminal_io::manual(command).unwrap();
    }
    if let Some(manual) = crate::coreutils::help(command) {
        return manual;
    }
    if ["fastfetch", "neofetch"].contains(&command) {
        return crate::system_info::HELP.into();
    }
    if let Some(manual) = crate::packages::cli::manual(command) {
        return manual;
    }
    if let Some(manual) = crate::archive::cli::manual(command) {
        return manual;
    }
    if ["bash", "sh", "source"].contains(&command) {
        return crate::shell::HELP.into();
    }
    if let Some(manual) = crate::terminal_query::manual(command) {
        return manual;
    }
    if let Some(manual) = crate::terminal_io::manual(command) {
        return manual;
    }
    if let Some(manual) = crate::terminal_text::manual(command) {
        return manual;
    }
    let synopsis = match command {
        "pwd" => "pwd [OPTION]...",
        "tail" => "tail [OPTION]... [FILE]...",
        "apt" | "apt-get" => "apt [options] command",
        "systemctl" => "systemctl [OPTIONS...] COMMAND [UNIT...]",
        "service" => "service SERVICE COMMAND",
        "journalctl" => "journalctl [OPTIONS...]",
        "nano" => "nano [OPTIONS] [[+LINE,COLUMN] FILE]...",
        "grep" => "grep [OPTION]... PATTERNS [FILE]...",
        "find" => "find [path] [expression]",
        "ip" => "ip [ OPTIONS ] OBJECT { COMMAND | help }",
        "ps" => "ps [options]",
        "df" => "df [OPTION]... [FILE]...",
        "du" => "du [OPTION]... [FILE]...",
        "sector-ix" => "sector-ix",
        "wc" => "wc [OPTION]... [FILE]...",
        _ => "virtual Linux command",
    };
    format!("{}(1)                 CYBER WAR virtual manual\n\nNAME\n    {command} - {synopsis}\n\nDESCRIPTION\n    Partial game implementation; this synopsis is not a complete GNU/Linux contract.\n    Flags and outputs of this legacy command still require a vertical audit.\n    It never invokes a host executable or touches the real filesystem.\n\nSEE ALSO\n    help(1), nano(1), systemctl(1), apt(8)\n", command)
}

pub(crate) fn dispatch(
    world: &mut WorldState,
    parts: &[String],
    actor: &str,
) -> GameResult<crate::terminal_io::Output> {
    let anchors = world.cwd_anchors();
    let result = dispatch_command(world, parts, actor);
    world.repair_cwds(anchors);
    result
}
fn dispatch_command(
    world: &mut WorldState,
    parts: &[String],
    actor: &str,
) -> GameResult<crate::terminal_io::Output> {
    use crate::terminal_io::Output;
    if parts[0].contains('/') {
        let absolute = normalize(&parts[0], &world.terminal.cwd)?;
        if let Ok(resolved) = world
            .fs()?
            .resolve(&absolute, actor, crate::vfs::Follow::Yes)
        {
            if world.packages.ownership.contains_key(&resolved) && resolved != parts[0] {
                let mut argv = parts.to_vec();
                argv[0] = resolved;
                return dispatch(world, &argv, actor);
            }
        }
    }
    let original_name = parts[0].as_str();
    let mut resolved_parts = parts.to_vec();
    if !original_name.contains('/') && !crate::shell::is_builtin(original_name) {
        if let Some(path) = crate::shell::path_file(world, original_name, actor) {
            if !world.packages.ownership.contains_key(&path) {
                resolved_parts[0] = path;
                return dispatch(world, &resolved_parts, actor);
            }
        }
    }
    if world.terminal.host.is_none() {
        let base = original_name.rsplit('/').next().unwrap_or(original_name);
        if crate::packages::executables::is_builtin(world, base)
            && !(crate::shell::is_builtin(original_name) && !original_name.contains('/'))
        {
            if crate::packages::executables::resolve(world, original_name, actor).is_none() {
                return Ok(crate::packages::executables::unavailable(
                    world,
                    original_name,
                    actor,
                ));
            }
            resolved_parts[0] = base.into();
        } else if let Some(result) =
            crate::packages::executables::dispatch(world, original_name, &parts[1..], actor)
        {
            return result;
        } else if original_name.contains('/') {
            let path = normalize(original_name, &world.terminal.cwd)?;
            if let Ok(node) = world.fs()?.stat(&path, actor) {
                if node.kind != "file" || !world.fs()?.allowed(node, actor, 1) {
                    return Ok(Output {
                        stderr: format!("bash: {original_name}: Permission denied\n"),
                        status: 126,
                        ..Output::default()
                    });
                }
                if node.content.starts_with("#!") && node.blob.is_none() {
                    let mut args = vec![path];
                    args.extend_from_slice(&parts[1..]);
                    return crate::shell::invoke(world, "bash", &args, actor);
                }
                return Ok(Output {
                    stderr: format!("bash: {original_name}: cannot execute virtual file\n"),
                    status: 126,
                    ..Output::default()
                });
            }
        }
    }
    let parts = resolved_parts.as_slice();
    let name = parts[0].as_str();
    let args = &parts[1..];
    if let Some(result) = crate::packages::cli::execute(world, name, args, actor) {
        return result;
    }
    if let Some(result) = crate::coreutils::execute(world, original_name, args, actor) {
        return result;
    }
    if name != "wc"
        && world
            .terminal
            .stdin_bytes
            .as_ref()
            .is_some_and(|b| std::str::from_utf8(b).is_err())
    {
        return Err(domain(format!(
            "{name}: binary stdin is not supported by this text-only command"
        )));
    }
    if name == "umask" {
        if args.is_empty() {
            return Ok(Output::success(format!(
                "{:04o}\n",
                world.terminal.shell.umask
            )));
        }
        if args.len() != 1
            || args[0].is_empty()
            || !args[0].chars().all(|c| ('0'..='7').contains(&c))
        {
            return Err(domain("umask: supported subset: octal 000..777"));
        }
        let mask = u16::from_str_radix(&args[0], 8).map_err(|_| domain("umask: invalid mask"))?;
        if mask > 0o777 {
            return Err(domain("umask: invalid mask"));
        }
        world.terminal.shell.umask = mask;
        return Ok(Output::default());
    }
    if name == "jobs" {
        return Ok(Output::success(crate::archive::jobs::listing(world)));
    }
    if let Some(result) = crate::archive::cli::execute(world, name, args, actor) {
        return result;
    }
    if name == "export" {
        if args.is_empty() || args == ["-p"] {
            return Ok(Output::success(
                world
                    .terminal
                    .exported
                    .iter()
                    .map(|key| {
                        format!(
                            "declare -x {key}=\"{}\"\n",
                            world
                                .terminal
                                .env
                                .get(key)
                                .map(String::as_str)
                                .unwrap_or("")
                        )
                    })
                    .collect(),
            ));
        }
        for value in args {
            let (key, assigned) = value
                .split_once('=')
                .map_or((value.as_str(), None), |(k, v)| (k, Some(v)));
            if !crate::shell::identifier(key) {
                return Err(domain("export: invalid identifier or unsupported option"));
            }
            if let Some(value) = assigned {
                world.terminal.env.insert(key.into(), value.into());
                world.terminal.shell.unset.remove(key);
            }
            world.terminal.exported.insert(key.into());
            if !world
                .terminal
                .shell
                .environment_order
                .iter()
                .any(|k| k == key)
            {
                world.terminal.shell.environment_order.push(key.into());
            }
        }
        return Ok(Output::default());
    }
    if name == "unset" {
        for key in args {
            if !crate::shell::identifier(key) {
                return Ok(Output {
                    status: 2,
                    stderr: "unset: invalid identifier or unsupported option\n".into(),
                    ..Output::default()
                });
            }
            world.terminal.env.remove(key);
            world.terminal.exported.remove(key);
            world.terminal.shell.environment_order.retain(|k| k != key);
            world.terminal.shell.unset.insert(key.clone());
        }
        return Ok(Output::default());
    }
    if name == "return" || (name == "exit" && world.terminal.shell_depth > 0) {
        if name == "return" && world.terminal.shell.source_depth == 0 {
            return Ok(Output {
                status: 1,
                stderr: "return: only valid in a sourced file\n".into(),
                ..Output::default()
            });
        }
        if args.len() > 1 {
            return Ok(Output {
                status: 1,
                stderr: format!("{name}: too many arguments\n"),
                ..Output::default()
            });
        }
        let status = match args.first() {
            None => world.terminal.last_status,
            Some(s) => match s.parse::<i64>() {
                Ok(n) => n.rem_euclid(256) as i32,
                Err(_) => {
                    return Ok(Output {
                        status: 2,
                        stderr: format!("{name}: numeric argument required\n"),
                        ..Output::default()
                    })
                }
            },
        };
        world.terminal.shell.flow = Some(if name == "exit" {
            crate::shell::Flow::Exit
        } else {
            crate::shell::Flow::Return
        });
        return Ok(Output {
            status,
            ..Output::default()
        });
    }
    if [
        "set", "if", "then", "fi", "for", "while", "until", "case", "function", "eval", "exec",
    ]
    .contains(&name)
    {
        return Ok(Output {
            status: 2,
            stderr: "shell: control syntax/set is not implemented\n".into(),
            ..Output::default()
        });
    }
    if ["bash", "sh", "source", "."].contains(&name) {
        return crate::shell::invoke(world, name, args, actor);
    }
    if let Some(result) = crate::terminal_query::execute(world, name, args, actor) {
        return result;
    }
    if let Some(result) = crate::terminal_io::execute(world, name, args, actor) {
        return result;
    }
    if ["wc", "sort", "uniq"].contains(&name) {
        return crate::terminal_text::execute(world, name, args, actor);
    }
    if name == "sudo" {
        if args.is_empty() || args[0] == "sudo" || args[0] == "su" {
            return Err(domain("usage: sudo BUILTIN [ARGS]"));
        }
        if !["kali", "vex", "root"].contains(&actor) {
            return Err(domain("not in sudoers"));
        }
        if args[0] == "cd" {
            return Err(domain("sudo: cd is a shell builtin; changing the caller's directory through sudo is not supported"));
        }
        return dispatch(world, args, "root");
    }
    if name == "false" {
        return Ok(Output {
            status: 1,
            ..Output::default()
        });
    }
    if name == "service" || name == "systemctl" {
        return service_contract(world, name, args, actor);
    }
    if ["ps", "kill", "pkill", "killall"].contains(&name) {
        return process_contract(world, name, args, actor);
    }
    run(world, parts, actor).map(Output::success)
}

fn run(world: &mut WorldState, parts: &[String], actor: &str) -> GameResult<String> {
    let name = parts[0].as_str();
    let args = &parts[1..];
    if !COMMANDS.contains(&name) {
        if let Some(tool) = crate::software::by_command(name) {
            return crate::software::command(world, tool, args, actor);
        }
        return Err(domain(format!("{name}: command not found")));
    }
    match name {
        "help" => Ok(format!("{}\n\nQuotes e redirecionamento > / >> são suportados.\nedit CAMINHO TEXTO · sudo COMANDO · ssh usuário@host senha\nlab help mostra os instrumentos dos laboratórios.\n", COMMANDS.join("  "))),
        "sector-ix" => {
            let install = if actor == "root" {
                "/root/Games/sector-ix/saves/sector-ix.save"
            } else {
                "/home/kali/Games/sector-ix/saves/sector-ix.save"
            };
            if !world.vfs.nodes.contains_key(install) {
                return Err(domain("sector-ix: jogo não instalado; visite vigilia.org ou use wget"));
            }
            world.flags.insert("SECTOR_IX_LAUNCHED".into());
            Ok("SECTOR IX — Protocolo Zero\nRuntime interno iniciado em modo janela.\nUse Aplicativos > Jogos > SECTOR IX para abrir a janela do jogo.\n".into())
        }
        "fastfetch" | "neofetch" => crate::system_info::run(world, name, args, actor),
        "id" => Ok(format!("uid={}({actor}) gid={}({actor})\n", if actor == "root" { 0 } else { 1000 }, if actor == "root" { 0 } else { 1000 })),
        "groups" => Ok(format!("{actor} : {actor} sudo adm\n")),
        "hostname" => {
            if has_flag(args, 'I', "--all-ip-addresses") {
                Ok("10.20.4.2\n".into())
            } else if has_flag(args, 'f', "--fqdn") {
                Ok(format!("{}.lifeos.local\n", world.hostname))
            } else {
                Ok(format!("{}\n", world.hostname))
            }
        }
        "date" => {
            let format = args.first().map(String::as_str).unwrap_or("");
            Ok(match format {
                "+%s" => format!("{}\n", world.playtime_seconds),
                "+%F" => "1970-01-01\n".into(),
                _ => "Thu Jan 01 00:00:00 UTC 1970\n".into(),
            })
        }
        "export" => {
            for assignment in args.iter().filter(|value| value.contains('=')) {
                let (key, value) = assignment.split_once('=').ok_or_else(|| domain("export: invalid assignment"))?;
                if key.is_empty() || !key.chars().all(|char| char.is_ascii_alphanumeric() || char == '_') {
                    return Err(domain("export: not a valid identifier"));
                }
                world.terminal.env.insert(key.into(), value.into());
            }
            Ok(String::new())
        }
        "true" => Ok(String::new()),
        "yes" => {
            let text = operands(args).join(" ");
            let text = if text.is_empty() { "y".into() } else { text };
            Ok(format!("{text}\n"))
        }
        "uname" => {
            let all = has_flag(args, 'a', "--all");
            let fields = [
                ('s', "--kernel-name", "Linux"),
                ('n', "--nodename", world.hostname.as_str()),
                ('r', "--kernel-release", "6.6.0-lifeos-sim"),
                ('v', "--kernel-version", "#1 SMP virtual"),
                ('m', "--machine", "x86_64"),
                ('p', "--processor", "x86_64"),
                ('i', "--hardware-platform", "x86_64"),
                ('o', "--operating-system", "GNU/Linux"),
            ];
            let selected = fields
                .iter()
                .filter(|(short, long, _)| all || has_flag(args, *short, long))
                .map(|(_, _, value)| *value)
                .collect::<Vec<_>>();
            let selected = if selected.is_empty() {
                vec!["Linux"]
            } else {
                selected
            };
            Ok(format!("{}\n", selected.join(" ")))
        }
        "clear" => Ok("\u{1b}[2J\u{1b}[H".into()),
        "echo" => {
            let no_newline = has_flag(args, 'n', "--no-newline");
            let escape = has_flag(args, 'e', "--enable-escape");
            let values = args
                .iter()
                .filter(|arg| !is_short_flag_cluster(arg, "ne") && !arg.starts_with("--"))
                .cloned()
                .collect::<Vec<_>>();
            let text = values.join(" ");
            let text = if escape {
                text.replace("\\n", "\n")
                    .replace("\\t", "\t")
                    .replace("\\r", "\r")
                    .replace("\\x1b", "\u{1b}")
            } else {
                text
            };
            Ok(if no_newline {
                text
            } else {
                format!("{text}\n")
            })
        }
        "rmdir" => {
            let options=crate::terminal_io::options("rmdir",args,"","",&[])?;
            if options.help { return Ok("rmdir DIRECTORY... — remove empty virtual directories\n".into()); }
            if options.files.is_empty() { return Err(domain("rmdir: missing operand")); }
            for name in options.files { let p=path(world,&name)?; world.fs_mut()?.rmdir(&p,actor)?; }
            Ok(String::new())
        }
        "ln" => {
            let options = crate::terminal_io::options("ln",args,"s","",&[("symbolic",'s')])?;
            if options.help { return Ok("ln [-s] TARGET LINK_NAME — virtual hard/symbolic links\n".into()); }
            if options.files.len()!=2 { return Err(domain("usage: ln [-s] TARGET LINK_NAME")); }
            let target=path(world,&options.files[1])?;
            if options.has('s') {world.fs_mut()?.symlink(&target,&options.files[0],actor)?;}
            else {let source=path(world,&options.files[0])?;world.fs_mut()?.link(&source,&target,actor)?;}
            Ok(String::new())
        }
        "realpath" | "readlink" => {
            let value = arg(args, 0, &format!("{name} PATH"))?;
            let p = path(world, &value)?;
            let value = if name == "readlink" {world.fs()?.readlink(&p, actor)?} else {world.fs()?.resolve(&p, actor, crate::vfs::Follow::Yes)?};
            Ok(format!("{value}\n"))
        }
        "which" | "whereis" | "type" | "command" => {
            if name == "command" && args.first().map(String::as_str) != Some("-v") { return Err(domain("usage: command -v COMMAND")); }
            let value = arg(args, usize::from(name == "command"), &format!("{name} COMMAND"))?;
            if world.terminal.host.is_none() && (crate::packages::executables::managed(&value) || world.packages.ownership.contains_key(&format!("/usr/bin/{value}"))) {
                let resolved = crate::packages::executables::resolve(world, &value, actor);
                return Ok(resolved.map_or_else(String::new, |path| if name == "whereis" {
                    let man = format!("/usr/share/man/man1/{value}.1");
                    format!("{value}: {path}{}\n", if world.vfs.nodes.contains_key(&man) { format!(" {man}") } else { String::new() })
                } else { format!("{path}\n") }));
            }
            if COMMANDS.contains(&value.as_str()) || crate::software::by_command(&value).is_some() {
                Ok(if name == "whereis" {
                    format!("{value}: /usr/bin/{value} /usr/share/man/man1/{value}.1.gz\n")
                } else {
                    format!("/usr/bin/{value}\n")
                })
            } else {
                Ok(String::new())
            }
        }
        "ls" => {
            let values = operands(args);
            let p = path(world, values.first().map(String::as_str).unwrap_or("."))?;
            let show_all = has_flag(args, 'a', "--all");
            let almost_all = has_flag(args, 'A', "--almost-all");
            let long = has_flag(args, 'l', "--format=long");
            let classify = has_flag(args, 'F', "--classify");
            let directory = has_flag(args, 'd', "--directory");
            let reverse = has_flag(args, 'r', "--reverse");
            let mut entries = if directory {
                vec![world.fs()?.stat(&p, actor)?.clone()]
            } else {
                world.fs()?.list(&p, actor)?
            };
            entries.retain(|entry| show_all || almost_all || !entry.name.starts_with('.'));
            entries.sort_by(|left, right| left.name.cmp(&right.name));
            if reverse {
                entries.reverse();
            }
            let mut out = String::new();
            for entry in entries {
                let suffix = if entry.kind == "directory" {
                    if classify { "/" } else { "" }
                } else {
                    ""
                };
                if long {
                    out.push_str(&format!(
                        "{:03o} {:8} {:>8} {entry_name}{suffix}\n",
                        entry.mode,
                        entry.owner,
                        entry.content.len(),
                        entry_name = entry.name
                    ));
                } else {
                    out.push_str(&format!("{entry_name}{suffix}\n", entry_name = entry.name));
                }
            }
            Ok(out)
        }
        "tree" | "find" => {
            let p = path(world, operands(args).first().map(String::as_str).unwrap_or("."))?;
            let mut out = String::new();
            let mut pending = vec![p];
            while let Some(dir) = pending.pop() {
                for n in world.fs()?.list(&dir, actor)? {
                    out.push_str(&format!("{}{}\n", n.id, if n.kind == "directory" { "/" } else { "" }));
                    if n.kind == "directory" {
                        pending.push(n.id);
                    }
                }
            }
            Ok(out)
        }
        "mkdir" | "touch" => {
            let values = if name == "mkdir" {
                operands_with_values(args, &["-m", "--mode"])
            } else {
                operands(args)
            };
            if values.is_empty() {
                return Err(domain("missing path"));
            }
            let parents = has_flag(args, 'p', "--parents");
            let no_create = has_flag(args, 'c', "--no-create");
            for a in values {
                let p = path(world, &a)?;
                if name == "mkdir" {
                    if world.fs()?.nodes.contains_key(&p) {
                        if !parents || world.fs()?.nodes.get(&p).is_some_and(|n| n.kind != "directory") {
                            return Err(domain("file exists"));
                        }
                        continue;
                    }
                    if parents {
                        let mut current = String::new();
                        for part in p.split('/').filter(|part| !part.is_empty()) {
                            current.push('/');
                            current.push_str(part);
                            if !world.fs()?.nodes.contains_key(&current) {
                                world.fs_mut()?.mkdir(&current, actor)?;
                            }
                        }
                    } else {
                        world.fs_mut()?.mkdir(&p, actor)?;
                    }
                    if let Some(mode) = option_value(args, 'm', "--mode", "mkdir [-p] [-m MODE] DIRECTORY")? {
                        let mode = u16::from_str_radix(&mode, 8).map_err(|_| domain("mode must be octal"))?;
                        world.fs_mut()?.chmod(&p, actor, mode)?;
                    }
                } else if !no_create || world.fs()?.path_exists(&p, actor)? {
                    world.fs_mut()?.touch(&p, actor)?;
                }
            }
            Ok(String::new())
        }
        "cut" => {
            let values = operands_with_values(args, &["-d", "--delimiter", "-f", "--fields"]);
            let value = values.first().ok_or_else(|| domain("usage: cut -d DELIMITER -f LIST FILE"))?;
            let p = path(world, value)?;
            let content = world.fs()?.read(&p, actor)?;
            let delimiter = option_value(args, 'd', "--delimiter", "cut -d DELIMITER -f LIST FILE")?.unwrap_or_else(|| "\t".into());
            let fields = option_value(args, 'f', "--fields", "cut -d DELIMITER -f LIST FILE")?.unwrap_or_else(|| "1".into());
            let indexes = fields.split(',').filter_map(|field| field.parse::<usize>().ok()).collect::<Vec<_>>();
            if indexes.is_empty() { return Err(domain("cut: invalid field list")); }
            Ok(content.lines().map(|line| indexes.iter().filter_map(|index| line.split(delimiter.as_str()).nth(index.saturating_sub(1))).collect::<Vec<_>>().join(delimiter.as_str())).map(|line| format!("{line}\n")).collect())
        }
        "tr" => {
            let values = operands(args);
            if values.len() < 3 {
                return Err(domain("usage: tr [OPTION] SET1 SET2 TEXT"));
            }
            let from = &values[0];
            let to = &values[1];
            let mut output = values[2..].join(" ");
            if has_flag(args, 'd', "--delete") {
                output.retain(|char| !from.contains(char));
            } else {
                let from_chars = from.chars().collect::<Vec<_>>();
                let to_chars = to.chars().collect::<Vec<_>>();
                output = output.chars().map(|char| from_chars.iter().position(|item| *item == char).map(|index| to_chars.get(index).copied().unwrap_or_else(|| *to_chars.last().unwrap_or(&char))).unwrap_or(char)).collect();
            }
            Ok(format!("{output}\n"))
        }
        "seq" => {
            let values = operands(args);
            let numbers = values.iter().map(|value| value.parse::<i64>().map_err(|_| domain("seq: invalid number"))).collect::<GameResult<Vec<_>>>()?;
            if numbers.is_empty() || numbers.len() > 3 { return Err(domain("usage: seq [FIRST [INCREMENT]] LAST")); }
            let (first, increment, last) = match numbers.as_slice() {
                [last] => (1, 1, *last),
                [first, last] => (*first, 1, *last),
                [first, increment, last] => (*first, *increment, *last),
                _ => unreachable!(),
            };
            if increment == 0 { return Err(domain("seq: zero increment")); }
            let mut output = String::new();
            let mut current = first;
            while (increment > 0 && current <= last) || (increment < 0 && current >= last) {
                output.push_str(&format!("{current}\n"));
                current += increment;
                if output.len() > 8192 { break; }
            }
            Ok(output)
        }
        "less" => {
            let value = operands(args)
                .first()
                .cloned()
                .ok_or_else(|| domain("usage: less FILE"))?;
            let p = path(world, &value)?;
            let content = world.fs()?.read(&p, actor)?;
            Ok(if has_flag(args, 'N', "--LINE-NUMBERS") {
                content
                    .lines()
                    .enumerate()
                    .map(|(index, line)| format!("{:>6}  {line}\n", index + 1))
                    .collect()
            } else {
                content
            })
        }
        "strings" => {
            let value = operands(args)
                .first()
                .cloned()
                .ok_or_else(|| domain("usage: strings FILE"))?;
            let p = path(world, &value)?;
            let content = world.fs()?.read(&p, actor)?;
            Ok(content
                .lines()
                .filter(|line| line.chars().filter(|char| !char.is_control()).count() >= 4)
                .map(|line| format!("{line}\n"))
                .collect())
        }
        "df" => {
            let value = operands(args).first().cloned().unwrap_or_else(|| "/".into());
            let p = path(world, &value)?;
            world.fs()?.stat(&p, actor)?;
            let fs = world.fs()?;
            let capacity = fs.capacity_bytes;
            let used = fs.used_bytes();
            let available = capacity.saturating_sub(used);
            let percent = used.saturating_mul(100).checked_div(capacity).unwrap_or(100);
            if has_flag(args, 'T', "--print-type") {
                Ok(format!("Filesystem Type 1024-blocks Used Available Capacity Mounted on\nvirtual ext4 {} {} {} {percent}% /\n", capacity / 1024, used.div_ceil(1024), available / 1024))
            } else if has_flag(args, 'h', "--human-readable") {
                Ok(format!("Filesystem Size Used Avail Use% Mounted on\nvirtual {:.1}G {:.1}M {:.1}G {percent}% /\n", capacity as f64 / 1073741824.0, used as f64 / 1048576.0, available as f64 / 1073741824.0))
            } else {
                Ok(format!("Filesystem 1024-blocks Used Available Capacity Mounted on\nvirtual {} {} {} {percent}% /\n", capacity / 1024, used.div_ceil(1024), available / 1024))
            }
        }
        "du" => {
            let value = operands(args).first().cloned().unwrap_or_else(|| ".".into());
            let p = path(world, &value)?;
            let node = world.fs()?.stat(&p, actor)?.clone();
            let prefix = format!("{p}/");
            let total = world.fs()?.nodes.iter().filter(|(key, _)| *key == &p || key.starts_with(&prefix)).map(|(_, item)| item.logical_size()).fold(0u64, u64::saturating_add);
            let human = |size: u64| if size >= 1024 { format!("{:.1}K", size as f64 / 1024.0) } else { format!("{size}B") };
            if node.kind == "file" || has_flag(args, 's', "--summarize") {
                Ok(format!("{} {}\n", if has_flag(args, 'h', "--human-readable") { human(total) } else { total.div_ceil(1024).to_string() }, p))
            } else {
                let mut output = String::new();
                for child in world.fs()?.list(&p, actor)? {
                    let child_prefix = format!("{}/", child.id);
                    let child_size = world.fs()?.nodes.iter().filter(|(key, _)| *key == &child.id || key.starts_with(&child_prefix)).map(|(_, item)| item.logical_size()).fold(0u64, u64::saturating_add);
                    output.push_str(&format!("{} {}\n", if has_flag(args, 'h', "--human-readable") { human(child_size) } else { child_size.div_ceil(1024).to_string() }, child.id));
                }
                output.push_str(&format!("{} {}\n", if has_flag(args, 'h', "--human-readable") { human(total) } else { total.div_ceil(1024).to_string() }, p));
                Ok(output)
            }
        }
        "file" => {
            let values = operands(args);
            if values.is_empty() { return Err(domain("usage: file FILE...")); }
            let mut output = String::new();
            for value in values {
                let p = path(world, &value)?;
                let n = world.fs()?.stat(&p, actor)?;
                if n.kind != "file" { output.push_str(&format!("{value}: {}\n",n.kind)); continue; }
                let data = crate::archive::bytes(world, &p, actor)?;
                let kind = if crate::packages::deb::detected(&data) { "Debian binary package (virtual)" }
                    else if let Some(format) = crate::archive::detect(&data) { format.description() }
                    else if std::str::from_utf8(&data).is_ok() { if data.is_ascii() { "ASCII text (virtual file)" } else { "UTF-8 text (virtual file)" } }
                    else { "data" };
                output.push_str(&format!("{value}: {kind}\n"));
            }
            Ok(output)
        }
        "man" => {
            let values = operands(args);
            let command = values.first().ok_or_else(|| domain("What manual page do you want?"))?;
            if let Some(result) = crate::packages::executables::manual(world, command, actor) { return result; }
            if !COMMANDS.contains(&command.as_str()) && crate::software::by_command(command).is_none() {
                return Err(domain(format!("No manual entry for {command}")));
            }
            Ok(manual_page(command))
        }
        "grep" => {
            let values = operands_with_values(
                args,
                &[
                    "-e",
                    "--regexp",
                    "-A",
                    "-B",
                    "-C",
                    "--after-context",
                    "--before-context",
                    "--context",
                ],
            );
            let pattern = option_value(args, 'e', "--regexp", "grep PATTERN FILE")?
                .or_else(|| values.first().cloned())
                .ok_or_else(|| domain("usage: grep [OPTION]... PATTERN [FILE]..."))?;
            let files = if option_value(args, 'e', "--regexp", "grep PATTERN FILE")?.is_some() {
                values
            } else {
                values.into_iter().skip(1).collect()
            };
            if files.is_empty() {
                return Err(domain("usage: grep [OPTION]... PATTERN [FILE]..."));
            }
            let ignore_case = has_flag(args, 'i', "--ignore-case");
            let invert = has_flag(args, 'v', "--invert-match");
            let line_number = has_flag(args, 'n', "--line-number");
            let count_only = has_flag(args, 'c', "--count");
            let files_only = has_flag(args, 'l', "--files-with-matches");
            let quiet = has_flag(args, 'q', "--quiet");
            let needle = if ignore_case { pattern.to_lowercase() } else { pattern.clone() };
            let mut output = String::new();
            let multiple_files = files.len() > 1;
            for value in files {
                let p = path(world, &value)?;
                let content = world.fs()?.read(&p, actor)?;
                let matches = content
                    .lines()
                    .enumerate()
                    .filter(|(_, line)| {
                        let haystack: String = if ignore_case {
                            line.to_lowercase()
                        } else {
                            (*line).to_owned()
                        };
                        haystack.contains(&needle) != invert
                    })
                    .collect::<Vec<_>>();
                if quiet && !matches.is_empty() {
                    return Ok(String::new());
                }
                if count_only {
                    output.push_str(&format!("{}:{}\n", p, matches.len()));
                } else if files_only {
                    if !matches.is_empty() {
                        output.push_str(&format!("{p}\n"));
                    }
                } else {
                    for (line, text) in matches {
                        if multiple_files {
                            output.push_str(&format!("{p}:"));
                        }
                        if line_number {
                            output.push_str(&format!("{}:", line + 1));
                        }
                        output.push_str(&format!("{text}\n"));
                    }
                }
            }
            Ok(output)
        }
        "edit" => { let p = path(world, &arg(args, 0, "edit FILE TEXT")?)?; if args.len() < 2 { return Err(domain("usage: edit FILE TEXT")); } world.fs_mut()?.write(&p, &format!("{}\n", args[1..].join(" ")), actor)?; Ok(String::new()) }
        "nano" => crate::nano::command(world, args, actor),
        "stat" => { let p = path(world, &arg(args, 0, "stat PATH")?)?; let n = world.fs()?.stat(&p, actor)?; Ok(format!("File: {p}\nType: {}\nSize: {}\nMode: {:03o}\nOwner: {}\nModified: {}\n", n.kind, n.content.len(), n.mode, n.owner, n.modified_at)) }
        "chmod" => {
            let spec = arg(args, 0, "chmod MODE PATH")?;
            let p = path(world, &arg(args, 1, "chmod MODE PATH")?)?;
            let current = world.fs()?.stat(&p, actor)?.mode;
            let mode = chmod_mode(&spec, current)?;
            world.fs_mut()?.chmod(&p, actor, mode)?;
            Ok(String::new())
        }
        "chown" => { let owner = arg(args, 0, "chown USER PATH")?; let p = path(world, &arg(args, 1, "chown USER PATH")?)?; world.fs_mut()?.chown(&p, actor, &owner)?; Ok(String::new()) }
        "su" => { if args.first().is_some_and(|a| a != "root" && a != "kali") { return Err(domain("unknown virtual user")); } world.terminal.user = args.first().cloned().unwrap_or_else(|| "root".into()); world.terminal.cwd = HOME.into(); Ok("Virtual user changed.\n".into()) }
        "ip" | "ifconfig" => {
            let interface = world.settings.get("network").map(String::as_str).unwrap_or("wlan0");
            if name == "ip" && operands(args).first().is_some_and(|value| value == "route") {
                Ok(format!("default via {} dev {interface}\n10.20.4.0/24 dev {interface} proto kernel scope link src 10.20.4.2\n", world.network.gateway))
            } else {
                Ok(format!("{interface}: {}\n    inet 10.20.4.2/24\n    gateway {}\n", if world.network.connected { "UP" } else { "DOWN" }, world.network.gateway))
            }
        }
        "lsblk" => {
            if let Some(layout) = crate::installer::lsblk(world) { return Ok(layout); }
            let disk = world.settings.get("disk").map(String::as_str).unwrap_or("nvme0n1");
            let scheme = world.settings.get("partitionScheme").map(String::as_str).unwrap_or("all");
            let layout = match scheme {
                "home" => "nvme0n1p1 1G vfat /boot/efi\nnvme0n1p2 470G ext4 /\nnvme0n1p3 30G ext4 /home",
                "var-tmp" => "nvme0n1p1 1G vfat /boot/efi\nnvme0n1p2 430G ext4 /\nnvme0n1p3 35G ext4 /home\nnvme0n1p4 5G ext4 /var",
                "server" => "nvme0n1p1 1G vfat /boot/efi\nnvme0n1p2 470G ext4 /\nnvme0n1p3 512M swap",
                "small" => "nvme0n1p1 512M vfat /boot/efi\nnvme0n1p2 9G ext4 /",
                _ => "nvme0n1p1 1G vfat /boot/efi\nnvme0n1p2 482G ext4 /\nnvme0n1p3 17G swap",
            };
            Ok(format!("NAME        SIZE TYPE MOUNTPOINT\n{disk}\n{layout}\n"))
        }
        "ipconfig" => Err(domain("ipconfig: command not found. Neste sistema, tente ifconfig.")),
        "whois" => {
            let target = args
                .iter()
                .rev()
                .find(|a| !a.starts_with('-'))
                .ok_or_else(|| domain("whois: target required"))?;
            let target = target.trim_start_matches("https://").trim_start_matches("http://").trim_end_matches('/');
            if let Ok(info) = crate::domains::whois(world, target) {
                let mut output = format!("Domain Name: {}\nRegistry Status: {}\n", info.address, info.status);
                if let Some(ip) = info.ip {
                    output.push_str(&format!("IP Address: {ip}\n"));
                }
                if let Some(owner) = info.owner {
                    output.push_str(&format!("Registrant Organization: {owner}\n"));
                }
                if info.network.as_deref() == Some("tor") {
                    output.push_str("Network: Tor\nService Type: Onion Service v3\nTraditional DNS: no\nIP Address: not published\n");
                }
                if let Some(version) = info.address_version {
                    output.push_str(&format!("Address Version: v{version}\n"));
                }
                if let Some(year) = info.registered_at_year {
                    output.push_str(&format!("Creation Year: {year}\n"));
                }
                if let Some(year) = info.expires_at_year {
                    output.push_str(&format!("Expiration Year: {year}\n"));
                }
                output.push_str(&format!("TLD: {}\nCategory: {}\n", info.suffix, info.category));
                if !info.subdomains.is_empty() {
                    output.push_str(&format!("Name Servers / Subdomains: {}\n", info.subdomains.join(", ")));
                }
                if let Some(target) = info.redirect_to {
                    output.push_str(&format!("Redirect: {target}\n"));
                }
                world.techniques.insert("whois".into());
                Ok(output)
            } else if crate::domains::looks_like_onion(target) {
                Err(domain(
                    "whois: Onion Service não encontrado ou endereço v3 inválido; .onion não usa DNS tradicional.",
                ))
            } else {
                let host = world.network.host(target)?;
                world.techniques.insert("whois".into());
                Ok(format!("Domain Name: {target}\nIP Address: {}\nRegistry Status: active\nRegistrar: Blackwire registry\nOrganization: private mirror network\n", host.address))
            }
        }
        "ping" | "nmap" | "dig" | "nslookup" | "traceroute" => {
            let target = args.iter().rev().find(|a| !a.starts_with('-')).ok_or_else(|| domain("target required"))?;
            if crate::domains::looks_like_onion(target) {
                return Err(domain(
                    "este comando consulta DNS/IP tradicional; use whois e o navegador Tor para um endereço .onion",
                ));
            }
            let host = world.network.host(target)?;
            let output = match name {
                "nmap" => host.services.iter().filter(|s| s.running && host.firewall.contains(&s.port)).map(|s| format!("{}/tcp open {} {}\n", s.port, s.name, s.version)).collect(),
                "ping" => format!("PING {target} ({})\n64 bytes: seq=1 ttl=64 time=4ms\n", host.address),
                "traceroute" => format!("1 {} 1ms\n2 {} 4ms\n", world.network.gateway, host.address),
                _ => format!("{target} A {}\n", host.address),
            };
            world.techniques.insert(name.into()); Ok(output)
        }
        "arp" => {
            let interface = world.settings.get("network").map(String::as_str).unwrap_or("wlan0");
            Ok(format!("{} 02:00:00:00:00:01 {interface}\n", world.network.gateway))
        }
        "route" => Ok(format!("Kernel IP routing table\nDestination     Gateway         Genmask         Flags Iface\n0.0.0.0         {}       0.0.0.0         UG    {}\n10.20.4.0       0.0.0.0         255.255.255.0 U     {}\n", world.network.gateway, world.settings.get("network").map(String::as_str).unwrap_or("wlan0"), world.settings.get("network").map(String::as_str).unwrap_or("wlan0"))),
        "ss" | "netstat" => Ok(world.terminal.host.as_ref().map(|h| format!("ESTABLISHED virtual:ssh -> {h}:22\n")).unwrap_or_else(|| "No virtual sessions.\n".into())),
        "curl" | "wget" => {
            let values = operands_with_values(args, &["-o", "--output"]);
            let url = values
                .last()
                .cloned()
                .ok_or_else(|| domain("usage: curl|wget [OPTION] URL"))?;
            if crate::archive::downloads::available(&url) {
                let file = if name == "wget" { option_value(args,'O',"--output-document","wget URL")? }
                    else { option_value(args,'o',"--output","curl -o FILE URL")? };
                let file = file.or_else(|| if name=="wget" || has_flag(args,'O',"--remote-name") {url.rsplit('/').next().map(String::from)} else {None})
                    .ok_or_else(||domain("binary download: specify -o FILE or -O"))?;
                let file = crate::archive::downloads::download(world,&url,&file,actor)?;
                return Ok(format!("Saved {file}\n"));
            }
            let body = world.network.request(&url)?;
            if name == "wget" {
                let file = option_value(args, 'O', "--output-document", "wget URL")?
                    .or_else(|| url.rsplit('/').next().map(str::to_owned))
                    .unwrap_or_else(|| "download.txt".into());
                let p = path(world, &file)?;
                let p = world.fs()?.available_path(&p, false);
                world.fs_mut()?.write(&p, &body, actor)?;
                if body.starts_with("CYBER SIEGE") && world.inventory.contains("cyber-siege") { world.flags.insert("GAME_DOWNLOADED".into()); }
                Ok(format!("Saved {p} ({} bytes)\n", body.len()))
            } else if let Some(file) = option_value(args, 'o', "--output", "curl URL")? {
                let p = path(world, &file)?;
                let p = world.fs()?.available_path(&p, false);
                world.fs_mut()?.write(&p, &body, actor)?;
                Ok(format!("Saved {p}\n"))
            } else if has_flag(args, 'O', "--remote-name") {
                let file = url.rsplit('/').next().filter(|value| !value.is_empty()).unwrap_or("index.html");
                let p = path(world, file)?;
                let p = world.fs()?.available_path(&p, false);
                world.fs_mut()?.write(&p, &body, actor)?;
                Ok(format!("Saved {p}\n"))
            } else if has_flag(args, 'I', "--head") {
                Ok("HTTP/1.1 200 OK\nContent-Type: text/plain\n\n".into())
            } else {
                Ok(format!("{body}\n"))
            }
        }
        "ssh" => {
            let (host, user) = world.network.ssh(&arg(args, 0, "ssh USER@HOST PASSWORD")?, &arg(args, 1, "ssh USER@HOST PASSWORD")?)?;
            world.terminal.host = Some(host); world.terminal.user = user; world.terminal.cwd = HOME.into(); world.techniques.insert("ssh".into()); Ok("Connected to virtual host. exit returns to your computer.\n".into())
        }
        "scp" => {
            let source = arg(args, 0, "scp USER@HOST:/FILE DEST PASSWORD")?; let (target, remote_path) = source.split_once(':').ok_or_else(|| domain("remote source required"))?;
            let (host, user) = world.network.ssh(target, &arg(args, 2, "scp USER@HOST:/FILE DEST PASSWORD")?)?;
            let p = normalize(remote_path, HOME)?;
            let text = world.network.hosts.get(&host).ok_or_else(|| domain("host missing"))?.files.read(&p, &user)?;
            let dest = path(world, &arg(args, 1, "scp USER@HOST:/FILE DEST PASSWORD")?)?; world.fs_mut()?.write(&dest, &text, actor)?; Ok("Transfer complete.\n".into())
        }
        "exit" => { world.terminal.host = None; world.terminal.user = "kali".into(); world.terminal.cwd = HOME.into(); Ok("Local session.\n".into()) }
        "ps" => {
            let all = has_flag(args, 'e', "--everyone") || has_flag(args, 'a', "--all") || args.iter().any(|value| value == "aux" || value == "ef");
            let mut output = if all { String::from("UID          PID %CPU %MEM    VSZ   RSS TTY      STAT START   TIME COMMAND\n") } else { String::from("    PID TTY          TIME CMD\n") };
            for process in world.processes.iter().filter(|process| all || process.running) {
                if all {
                    output.push_str(&format!("{:<12} {:>4}  0.0  0.0  10240  4096 pts/0    {}      00:00:00 {}\n", process.user, process.pid, if process.running { "Ss" } else { "T" }, process.name));
                } else {
                    output.push_str(&format!("{:>7} pts/0    00:00:00 {}\n", process.pid, process.name));
                }
            }
            Ok(output)
        }
        "top" => {
            let running = world.processes.iter().filter(|process| process.running).count();
            let mut output = format!("top - virtual uptime {} seconds, 1 user, load average: 0.00, 0.00, 0.00\nTasks: {} total, {} running, {} sleeping\n%Cpu(s): 0.0 us, 0.0 sy, 100.0 id\n\n  PID USER      S  %CPU %MEM     TIME+ COMMAND\n", world.playtime_seconds, world.processes.len(), running, world.processes.len().saturating_sub(running));
            output.push_str(&world.processes.iter().filter(|process| process.running).map(|process| format!("{:>5} {:<8} {}   0.0  0.0   0:00.00 {}\n", process.pid, process.user, "S", process.name)).collect::<String>());
            Ok(output)
        }
        "kill" => {
            let pid = operands(args)
                .first()
                .ok_or_else(|| domain("usage: kill [SIGNAL] PID"))?
                .parse::<u32>()
                .map_err(|_| domain("invalid pid"))?;
            let process = world
                .processes
                .iter_mut()
                .find(|process| process.pid == pid)
                .ok_or_else(|| domain("unknown virtual process"))?;
            if pid == 1 || (process.user != actor && actor != "root") {
                return Err(domain("permission denied"));
            }
            process.running = false;
            Ok(String::new())
        }
        "killall" | "pkill" => {
            let values = operands(args);
            let target = values.first().ok_or_else(|| domain(format!("usage: {name} PROCESS")))?;
            let mut found = false;
            for process in &mut world.processes {
                if (name == "killall" && process.name == *target) || (name == "pkill" && process.name.contains(target)) {
                    if process.pid == 1 || (process.user != actor && actor != "root") {
                        return Err(domain("permission denied"));
                    }
                    process.running = false;
                    found = true;
                }
            }
            if !found { return Err(domain(format!("{name}: no process found"))); }
            Ok(String::new())
        }
        "free" => {
            if has_flag(args, 'h', "--human") {
                Ok("               total        used        free      shared  buff/cache   available\nMem:           8.0Gi       128Mi       7.8Gi       0.0Ki       64Mi       7.8Gi\nSwap:          512Mi          0B       512Mi\n".into())
            } else {
                Ok("               total        used        free      shared  buff/cache   available\nMem:         8388608      131072     8060928           0       65536     8126464\nSwap:         524288           0      524288\n".into())
            }
        }
        "uptime" => Ok(format!(" 00:00:00 up {} min,  1 user,  load average: 0.00, 0.00, 0.00\n", world.playtime_seconds / 60)),
        "service" => service_command(world, args, actor),
        "systemctl" => systemctl_command(world, args, actor),
        "journalctl" => journalctl_command(world, args),
        "lab" => lab(world, args),
        _ => Err(domain("command not found")),
    }
}

fn lab(world: &mut WorldState, args: &[String]) -> GameResult<String> {
    let operation = args.first().map(String::as_str).unwrap_or("help");
    match operation {
        "trace" => {
            if !world.network.connected { return Err(domain("network unreachable")); }
            let mut output = String::from("node discovered\nroute established\nsession opened\npacket received\naccess granted\n");
            for (index, host) in world.network.hosts.values().enumerate() {
                output.push_str(&format!("0x{:06X}  01001001  {}  {}\n", 0x00AF21 + index * 31, host.address, host.hostname));
            }
            Ok(output)
        }
        "help" => Ok("Instrumentos: scan VALOR, play, build-v1, inspect, build-v2, wifi, collect, analyze, connect, recover, verify-backup, contain, restore, trace.\n".into()),
        "scan" | "play" | "build-v1" | "inspect" | "build-v2" => {
            if !world.flags.contains("GAME_DOWNLOADED") { return Err(domain("Cyber Siege is not installed")); }
            match operation {
                "scan" => {
                    let value = arg(args, 1, "lab scan VALUE")?.parse::<i64>().map_err(|_| domain("invalid memory value"))?;
                    if value != world.memory_value { return Err(domain("no matching values")); }
                    if world.memory_value != 100 { world.memory_candidates.retain(|a| *a == 8192); world.flags.insert("MEMORY_LOCATED".into()); }
                    Ok(format!("Candidates: {:?}\n", world.memory_candidates))
                }
                "play" => { world.memory_value -= 1; Ok(format!("GOLD: {}\n", world.memory_value)) }
                "build-v1" => {
                    if !world.flags.contains("MEMORY_LOCATED") { return Err(domain("BUILD FAILED: memory address unknown")); }
                    if world.flags.contains("V1_COMPLETE") { return Err(domain("INCOMPATIBLE VERSION: target changed")); }
                    world.flags.insert("V1_BUILT".into()); world.memory_value = 999999; world.vfs.write("/home/kali/projects/v1.sim", &format!("author={}\naddress=8192\n", world.nickname), "kali")?; Ok("BUILD SUCCESSFUL · GOLD: 999999\n".into())
                }
                "inspect" => { world.flags.insert("ROUTINE_ANALYZED".into()); Ok("LOAD gold → SUB 1 → STORE gold. A save file also persists gold.\n".into()) }
                _ => {
                    if !world.flags.contains("ROUTINE_ANALYZED") || !world.flags.contains("V1_COMPLETE") { return Err(domain("BUILD FAILED: inspect the updated routine")); }
                    world.flags.insert("V2_SOLVED".into()); world.techniques.insert("runtime-patch".into()); Ok("BUILD SUCCESSFUL · virtual runtime patch applied.\n".into())
                }
            }
        }
        "wifi" => Ok(world.network.wifi.iter().map(|w| format!("{} {} channel={} signal={} {}\n", w.ssid, w.bssid, w.channel, w.signal, w.encryption)).collect()),
        "collect" => { if world.network.connected { return Err(domain("use the available connection")); } world.flags.insert("WIFI_SAMPLES".into()); Ok("Amostras do laboratório: 9.873 IVs. Conjunto pronto para análise.\n".into()) }
        "analyze" => { if !world.flags.contains("WIFI_SAMPLES") { return Err(domain("not enough samples")); } world.flags.insert("WIFI_KEY".into()); Ok("Padrão estatístico identificado. Chave virtual recuperada.\n".into()) }
        "connect" => { if !world.flags.contains("WIFI_KEY") { return Err(domain("virtual access key missing")); } world.network.connected = true; if let Some(wifi) = world.network.wifi.get_mut(1) { wifi.access = true; } world.flags.insert("WIFI_CONNECTED".into()); world.techniques.insert("wireless-analysis".into()); Ok("CONNECTED · Oficina antiga\n".into()) }
        "recover" => {
            let backup = world.vfs.read("/home/kali/Documents/usb-backup.img", "kali")?;
            if !backup.contains("FOTO_01") { return Err(domain("invalid image")); }
            world.vfs.write("/home/kali/Documents/fotos-recuperadas.txt", "FOTO_01 recuperada\nFOTO_02 recuperada\npessoal.zip protegido\n", "kali")?;
            world.flags.insert("USB_RECOVERED".into()); world.techniques.insert("file-recovery".into()); Ok("Fotos recuperadas; original preservado.\n".into())
        }
        "verify-backup" | "contain" | "restore" => {
            let key = world.terminal.host.clone().ok_or_else(|| domain("connect to the virtual server"))?;
            let host = world.network.hosts.get_mut(&key).ok_or_else(|| domain("host missing"))?;
            let backup = host.files.read("/home/kali/web-backup.conf", "root")?;
            if !backup.contains("enabled=true") { return Err(domain("backup is not valid")); }
            match operation {
                "verify-backup" => { world.flags.insert("VEX_BACKUP_VALID".into()); }
                "contain" => { host.files.seed("/var/log/incident-preserved.log", "file", "unexpected login via permissive policy\n", "root"); world.flags.insert("VEX_CONTAINED".into()); }
                _ => { if !world.flags.contains("VEX_CONTAINED") { return Err(domain("contain and preserve evidence first")); } host.files.write("/etc/web.conf", "enabled=true\nroot_login=false\nfirewall=strict\n", "root")?; host.patched = true; world.flags.insert("VEX_RECOVERED".into()); }
            }
            Ok("Operação validada.\n".into())
        }
        _ => Err(domain("unknown lab instrument")),
    }
}

pub fn observe(world: &mut WorldState) {
    if !world.flags.contains("V2_SOLVED")
        && world.flags.contains("V1_COMPLETE")
        && world
            .vfs
            .nodes
            .get("/home/kali/projects/profile_01.dat")
            .is_some_and(|n| n.content.contains("gold=999999"))
    {
        world.flags.insert("V2_SOLVED".into());
        world.techniques.insert("save-editing".into());
        world.notify(
            "NULL",
            "... olha só. parece que você já sabia o que eu ia falar, né?",
        );
    }
    if world.flags.contains("VEX_SECURE") && !world.flags.contains("VEX_HARDENED") {
        if let Some(host) = world.network.hosts.get_mut("10.20.4.15") {
            host.files.seed(
                "/etc/web.conf",
                "file",
                "enabled=true\nroot_login=false\nfirewall=strict\n",
                "root",
            );
            host.patched = true;
        }
        world.flags.insert("VEX_HARDENED".into());
        world.notify("VEX", "Tentaram alguma coisa ontem. Não conseguiram.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sector_ix_installer_builds_visual_tree_and_keeps_save_functional() {
        let mut world = WorldState::new("neo", "pc").expect("world");
        world
            .vfs
            .write(
                "/home/kali/sector-ix-linux.sh",
                crate::browser::SECTOR_IX_INSTALLER,
                "kali",
            )
            .expect("installer");

        let result = execute(&mut world, "bash sector-ix-linux.sh");
        assert_eq!(result.exit_code, 0, "{}", result.stderr);
        for directory in [
            "/home/kali/Games/sector-ix",
            "/home/kali/Games/sector-ix/bin",
            "/home/kali/Games/sector-ix/assets",
            "/home/kali/Games/sector-ix/config",
            "/home/kali/Games/sector-ix/data",
            "/home/kali/Games/sector-ix/docs",
            "/home/kali/Games/sector-ix/logs",
            "/home/kali/Games/sector-ix/mods",
            "/home/kali/Games/sector-ix/runtime",
            "/home/kali/Games/sector-ix/saves",
        ] {
            assert_eq!(
                world
                    .vfs
                    .nodes
                    .get(directory)
                    .map(|node| node.kind.as_str()),
                Some("directory"),
                "missing {directory}"
            );
        }
        for placeholder in [
            "/home/kali/Games/sector-ix/bin/sector-ix",
            "/home/kali/Games/sector-ix/bin/sector-ix-launcher",
            "/home/kali/Games/sector-ix/assets/.keep",
            "/home/kali/Games/sector-ix/config/settings.conf",
            "/home/kali/Games/sector-ix/data/levels.dat",
            "/home/kali/Games/sector-ix/docs/README.txt",
            "/home/kali/Games/sector-ix/logs/sector-ix.log",
            "/home/kali/Games/sector-ix/mods/.keep",
            "/home/kali/Games/sector-ix/runtime/engine.bin",
            "/home/kali/Games/sector-ix/SECTOR-IX.desktop",
        ] {
            assert_eq!(
                world.vfs.read(placeholder, "kali").expect("placeholder"),
                "",
                "unexpected content in {placeholder}"
            );
        }
        let save = world
            .vfs
            .read("/home/kali/Games/sector-ix/saves/sector-ix.save", "kali")
            .expect("save");
        assert!(save.contains("\"game\":\"SECTOR IX\""));
        assert_eq!(execute(&mut world, "sector-ix").exit_code, 0);
    }

    #[test]
    fn terminal_creates_shared_files_and_rejects_shell() {
        let mut world = WorldState::new("neo", "pc").expect("world");
        assert_eq!(
            execute(&mut world, "echo \"two words\" > Documents/notes.txt").exit_code,
            0
        );
        assert_eq!(
            world
                .vfs
                .read("/home/kali/Documents/notes.txt", "kali")
                .expect("file"),
            "two words\n"
        );
        for command in [
            "powershell Get-Process",
            "cmd.exe /c dir",
            "bash -c 'cmd.exe /c dir'",
            "ls; curl external.com",
            "echo x | cmd",
            "cat C:\\Windows\\win.ini",
            "ssh user@127.0.0.1 password",
        ] {
            assert_ne!(execute(&mut world, command).exit_code, 0, "{command}");
        }
        assert_ne!(execute(&mut world, "cd missing").exit_code, 0);
        assert_eq!(execute(&mut world, "bash -c ls").exit_code, 0);
        assert_eq!(world.terminal.cwd, HOME);
    }
    #[test]
    fn failed_redirect_rolls_back() {
        let mut world = WorldState::new("neo", "pc").expect("world");
        assert_ne!(
            execute(&mut world, "mkdir test > /etc/forbidden").exit_code,
            0
        );
        assert!(!world.vfs.nodes.contains_key("/home/kali/test"));
        assert!(tokenize("echo 'unterminated").is_err());
        assert_eq!(execute(&mut world, "echo '>'").stdout, ">\n");
        assert_eq!(
            execute(&mut world, "echo '>>' > Documents/literal.txt").exit_code,
            0
        );
        assert_eq!(
            world
                .vfs
                .read("/home/kali/Documents/literal.txt", "kali")
                .expect("file"),
            ">>\n"
        );
    }

    #[test]
    fn rm_removes_files_without_using_gui_trash_and_force_does_not_bypass_permissions() {
        let mut world = WorldState::new("neo", "pc").expect("world");
        world
            .vfs
            .write("/home/kali/Documents/remove-me.txt", "safe", "kali")
            .expect("create");
        assert_eq!(
            execute(&mut world, "rm Documents/remove-me.txt").exit_code,
            0
        );
        assert!(!world
            .vfs
            .nodes
            .contains_key("/home/kali/Documents/remove-me.txt"));
        assert!(!world
            .vfs
            .nodes
            .contains_key("/home/kali/.local/share/Trash/files/remove-me.txt"));
        assert_eq!(execute(&mut world, "rm -f missing.txt").exit_code, 0);
        assert_eq!(execute(&mut world, "rm -f").exit_code, 0);
        assert_ne!(execute(&mut world, "rm -f /etc/os-release").exit_code, 0);
        execute(&mut world, "mkdir Documents/empty");
        assert_ne!(execute(&mut world, "rm -f Documents/empty").exit_code, 0);
        assert!(world.vfs.nodes.contains_key("/home/kali/Documents/empty"));
        assert_eq!(execute(&mut world, "rm -r Documents/empty").exit_code, 0);
    }
    #[test]
    fn installer_network_and_partition_choices_are_visible_in_terminal() {
        let mut world = WorldState::new("neo", "pc").expect("world");
        world.settings.insert("network".into(), "eth0".into());
        world.settings.insert("disk".into(), "sda".into());
        world
            .settings
            .insert("partitionScheme".into(), "home".into());
        let network = execute(&mut world, "ifconfig");
        assert_eq!(network.exit_code, 0);
        assert!(network.stdout.starts_with("eth0: UP"));
        let disks = execute(&mut world, "lsblk");
        assert_eq!(disks.exit_code, 0);
        assert!(disks.stdout.contains("sda"));
        assert!(disks.stdout.contains("/home"));
    }

    #[test]
    fn common_command_flags_change_virtual_output() {
        let mut world = WorldState::new("neo", "pc").expect("world");
        world
            .vfs
            .write("/home/kali/sample.txt", "Alpha\nBeta\nGamma\n", "kali")
            .expect("file");
        assert_eq!(
            execute(&mut world, "head -n 2 sample.txt").stdout,
            "Alpha\nBeta\n"
        );
        assert!(execute(&mut world, "cat -n sample.txt")
            .stdout
            .contains("     1\tAlpha"));
        assert_eq!(execute(&mut world, "echo -ne 'A\\nB'").stdout, "A\nB");
        assert!(execute(&mut world, "grep -in beta sample.txt")
            .stdout
            .contains("2:Beta"));
    }

    #[test]
    fn package_manager_and_services_keep_virtual_state() {
        let mut world = WorldState::new("neo", "pc").expect("world");
        world.network.connected = true;
        assert_eq!(execute(&mut world, "sudo apt update").exit_code, 0);
        let install = crate::archive::jobs::with_deferral(false, || {
            execute(&mut world, "sudo apt install -y netscan")
        });
        assert_eq!(install.exit_code, 0);
        assert!(execute(&mut world, "apt list --installed")
            .stdout
            .contains("netscan"));
        assert!(execute(&mut world, "dpkg -s netscan")
            .stdout
            .contains("install ok installed"));
        assert_eq!(
            crate::archive::jobs::with_deferral(false, || execute(
                &mut world,
                "sudo apt purge -y netscan"
            ))
            .exit_code,
            0
        );
        assert!(execute(&mut world, "dpkg -s netscan").exit_code != 0);

        assert!(execute(&mut world, "service ssh status")
            .stdout
            .contains("inactive"));
        assert_eq!(
            execute(&mut world, "sudo systemctl enable ssh").exit_code,
            0
        );
        assert_eq!(execute(&mut world, "sudo service ssh stop").exit_code, 0);
        assert!(execute(&mut world, "systemctl is-active ssh")
            .stdout
            .contains("inactive"));
        assert_eq!(execute(&mut world, "sudo systemctl start ssh").exit_code, 0);
        assert!(execute(&mut world, "systemctl status ssh.service")
            .stdout
            .contains("Active: active"));
        assert!(
            execute(&mut world, "journalctl -u ssh -n 2")
                .stdout
                .lines()
                .count()
                <= 2
        );
    }

    #[test]
    fn text_disk_and_environment_commands_use_virtual_files() {
        let mut world = WorldState::new("neo", "pc").expect("world");
        world
            .vfs
            .write(
                "/home/kali/sample.txt",
                "zulu:2\nalpha:1\nalpha:1\n",
                "kali",
            )
            .expect("sample");
        assert!(execute(&mut world, "tail -n 2 sample.txt")
            .stdout
            .contains("alpha:1"));
        assert!(execute(&mut world, "wc -l sample.txt")
            .stdout
            .starts_with("3 "));
        assert_eq!(
            execute(&mut world, "cut -d : -f 2 sample.txt").stdout,
            "2\n1\n1\n"
        );
        assert!(execute(&mut world, "sort -u sample.txt")
            .stdout
            .starts_with("alpha:1"));
        assert!(execute(&mut world, "du -sh .")
            .stdout
            .contains("/home/kali"));
        assert!(execute(&mut world, "df -h").stdout.contains("virtual"));
        assert_eq!(execute(&mut world, "export DEMO=ok").exit_code, 0);
        assert_eq!(execute(&mut world, "printenv DEMO").stdout, "ok\n");
    }

    #[test]
    fn shell_scripts_run_line_by_line_inside_the_virtual_filesystem() {
        let mut world = WorldState::new("neo", "pc").expect("world");
        world
            .vfs
            .write(
                "/home/kali/demo.sh",
                "#!/usr/bin/env bash\nexport DEMO=ok\necho \"$DEMO\" > script-output.txt\n",
                "kali",
            )
            .expect("script");
        let result = execute(&mut world, "bash demo.sh");
        assert_eq!(result.exit_code, 0);
        assert_eq!(
            world
                .vfs
                .read("/home/kali/script-output.txt", "kali")
                .expect("output"),
            "ok\n"
        );
    }
    #[test]
    fn unsupported_shell_options_do_not_succeed_silently() {
        let mut world = WorldState::new("neo", "pc").unwrap();
        world
            .vfs
            .write(
                "/home/kali/bad.sh",
                "set -e\necho should-not-run > output.txt\n",
                "kali",
            )
            .unwrap();
        let result = execute(&mut world, "bash bad.sh");
        assert_ne!(result.exit_code, 0);
        assert!(result.stderr.contains("not implemented"));
        assert!(!world.vfs.nodes.contains_key("/home/kali/output.txt"));
        assert_ne!(
            execute(&mut world, "bash --unsupported bad.sh").exit_code,
            0
        );
        assert_ne!(execute(&mut world, "bash -c 'exit 7'").exit_code, 0);
        assert_eq!(
            execute(&mut world, "bash -c 'echo \"$0 $1\"' name argument").stdout,
            "name argument\n"
        );
    }
}
