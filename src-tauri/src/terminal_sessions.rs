//! Runtime-only per-window contexts. All commands still use the shared virtual
//! world, selected under GameService's mutex; never a host shell or socket.
use crate::{
    error::GameResult,
    vfs::{domain, HOME},
    world::{TerminalSession, WorldState},
};
use std::collections::BTreeMap;

pub fn fresh_session() -> TerminalSession {
    TerminalSession {
        shell: Default::default(),
        presentation: Default::default(),
        shell_depth: 0,
        last_status: 0,
        exported: Default::default(),
        cwd: HOME.into(),
        user: "kali".into(),
        host: None,
        nano: None,
        env: BTreeMap::new(),
        history: Vec::new(),
        foreground: None,
        archive_pending: None,
        package_pending: None,
        package_job: None,
        stdin: None,
        stdin_bytes: None,
        io: Default::default(),
    }
}
fn validate_id(id: &str) -> GameResult<()> {
    if id.is_empty()
        || id.len() > 100
        || !id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_:".contains(c))
    {
        return Err(domain("invalid terminal session id"));
    }
    Ok(())
}
pub fn open(world: &mut WorldState, id: &str) -> GameResult<TerminalSession> {
    validate_id(id)?;
    if !world.terminal_sessions.contains_key(id) && world.terminal_sessions.len() >= 32 {
        return Err(domain("too many open terminals"));
    }
    Ok(world
        .terminal_sessions
        .entry(id.into())
        .or_insert_with(fresh_session)
        .clone())
}
pub fn close(world: &mut WorldState, id: &str) -> GameResult<()> {
    validate_id(id)?;
    if let Some(session) = world.terminal_sessions.remove(id) {
        if let Some(job) = session.package_job {
            crate::archive::jobs::cancel(job);
        }
        if session
            .package_pending
            .is_some_and(|p| world.package_lock.as_deref() == Some(&p.plan.id))
        {
            world.package_lock = None;
        }
    }
    Ok(())
}
pub fn open_at(
    world: &mut WorldState,
    id: &str,
    cwd: Option<&str>,
    as_root: bool,
) -> GameResult<TerminalSession> {
    if world.terminal_sessions.contains_key(id) {
        return open(world, id);
    }
    let cwd = crate::vfs::normalize(cwd.unwrap_or(HOME), HOME)?;
    let user = if as_root { "root" } else { "kali" };
    world.vfs.list(&cwd, user)?;
    let mut session = open(world, id)?;
    session.cwd = cwd;
    session.user = user.into();
    world.terminal_sessions.insert(id.into(), session.clone());
    Ok(session)
}
pub fn reset(world: &mut WorldState) {
    world.scheduler = Default::default();
    world.vfs.reset_watches();
    for host in world.network.hosts.values_mut() {
        host.files.reset_watches();
    }
    world.package_lock = None;
    crate::archive::jobs::reset(world);
    world.terminal = fresh_session();
    world.terminal_sessions.clear();
    world.settings.retain(|key, _| !key.starts_with("env:"));
}
pub fn with_session<T>(
    world: &mut WorldState,
    id: Option<&str>,
    change: impl FnOnce(&mut WorldState) -> GameResult<T>,
) -> GameResult<T> {
    let Some(id) = id else {
        return change(world);
    };
    validate_id(id)?;
    let selected = world
        .terminal_sessions
        .remove(id)
        .ok_or_else(|| domain("terminal session is closed"))?;
    let backup = selected.clone();
    let default = std::mem::replace(&mut world.terminal, selected);
    let scope = crate::shell::cooperative::SessionScope::enter(id, &default);
    let result = change(world);
    let default = scope.finish(default);
    let selected = std::mem::replace(&mut world.terminal, default);
    world
        .terminal_sessions
        .insert(id.into(), if result.is_ok() { selected } else { backup });
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal;
    #[test]
    fn desktop_terminal_context_is_local_isolated_and_not_reinitialized() {
        let mut world = WorldState::new("kali", "pc").unwrap();
        let regular = open_at(&mut world, "desktop", Some("/home/kali/Desktop"), false).unwrap();
        let root = open_at(&mut world, "admin", Some("/root"), true).unwrap();
        assert_eq!(regular.cwd, "/home/kali/Desktop");
        assert_eq!(regular.user, "kali");
        assert_eq!(root.user, "root");
        assert_eq!(world.terminal.user, "kali");
        assert!(open_at(&mut world, "denied", Some("/root"), false).is_err());
        assert!(!world.terminal_sessions.contains_key("denied"));
        let same = open_at(&mut world, "desktop", Some("/root"), true).unwrap();
        assert_eq!(same.user, "kali");
        assert_eq!(same.cwd, "/home/kali/Desktop");
    }
    fn command(world: &mut WorldState, id: &str, line: &str) -> terminal::CommandResult {
        with_session(world, Some(id), |w| Ok(terminal::execute(w, line))).unwrap()
    }
    #[test]
    fn three_terminals_isolate_cwd_ssh_privilege_environment_and_nano() {
        let mut world = WorldState::new("neo", "pc").unwrap();
        for id in ["A", "B", "C"] {
            open(&mut world, id).unwrap();
        }
        assert_eq!(command(&mut world, "A", "cd Documents").exit_code, 0);
        assert_eq!(
            command(&mut world, "B", "ssh vex@vex.local lab-only").exit_code,
            0
        );
        assert_eq!(command(&mut world, "C", "export LOCAL=value").exit_code, 0);
        assert_eq!(
            command(&mut world, "A", "pwd").stdout,
            "/home/kali/Documents\n"
        );
        assert_eq!(command(&mut world, "C", "pwd").stdout, "/home/kali\n");
        assert_eq!(command(&mut world, "B", "whoami").stdout, "vex\n");
        assert_eq!(command(&mut world, "B", "sudo whoami").stdout, "root\n");
        assert_eq!(command(&mut world, "C", "whoami").stdout, "kali\n");
        assert!(!command(&mut world, "A", "env").stdout.contains("LOCAL="));
        assert!(command(&mut world, "C", "env")
            .stdout
            .contains("LOCAL=value"));
        assert_eq!(command(&mut world, "A", "nano a.txt").exit_code, 0);
        assert_eq!(command(&mut world, "C", "nano c.txt").exit_code, 0);
        assert_eq!(
            world.terminal_sessions["A"].nano.as_ref().unwrap().path,
            "/home/kali/Documents/a.txt"
        );
        assert_eq!(
            world.terminal_sessions["C"].nano.as_ref().unwrap().path,
            "/home/kali/c.txt"
        );
        assert!(world.terminal_sessions["B"].nano.is_none());
        assert!(world.terminal.host.is_none());
        close(&mut world, "A").unwrap();
        assert!(with_session(&mut world, Some("A"), |_| Ok(())).is_err());
        assert!(world.terminal_sessions["C"].nano.is_some());
    }
    #[test]
    fn shared_files_but_no_context_survives_serialization_or_reset() {
        let mut world = WorldState::new("neo", "pc").unwrap();
        open(&mut world, "A").unwrap();
        open(&mut world, "B").unwrap();
        command(&mut world, "A", "echo shared > Documents/shared.txt");
        assert_eq!(
            command(&mut world, "B", "cat Documents/shared.txt").stdout,
            "shared\n"
        );
        let loaded: WorldState =
            serde_json::from_str(&serde_json::to_string(&world).unwrap()).unwrap();
        assert!(loaded.terminal_sessions.is_empty());
        assert!(!world.terminal_sessions["A"].history.is_empty());
        reset(&mut world);
        assert!(world.terminal_sessions.is_empty());
        assert!(world.terminal.nano.is_none());
        assert_eq!(
            world
                .vfs
                .read("/home/kali/Documents/shared.txt", "kali")
                .unwrap(),
            "shared\n"
        );
    }
}
