//! Deterministic LifeOS resource budgets, never host OS telemetry.
use crate::{error::GameResult, vfs::domain, world::WorldState};
use serde::Serialize;

const MIB: u64 = 1024 * 1024;
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub pid: u32,
    pub name: String,
    pub user: String,
    pub rss_bytes: u64,
    pub group_rss_bytes: u64,
    pub cpu_percent: f64,
    pub group_cpu_percent: f64,
    pub can_terminate: bool,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub tasks: Vec<Task>,
    pub cpu_percent: f64,
    pub memory_used_bytes: u64,
    pub memory_total_bytes: u64,
    pub metrics_kind: &'static str,
}
pub fn snapshot(world: &WorldState) -> Snapshot {
    let tasks: Vec<_> = world
        .processes
        .iter()
        .filter(|p| p.running)
        .map(|p| {
            // Each virtual process is its own group until child processes are modeled.
            let (memory_mib, cpu) = match p.name.as_str() {
                "lifeos-session" => (32, 0.5),
                "networking" => (8, 0.1),
                "ssh" => (7, 0.1),
                "web" | "apache2" | "nginx" => (24, 0.2),
                "postgresql" => (48, 0.2),
                "cron" => (3, 0.0),
                _ => (4, 0.0),
            };
            Task {
                pid: p.pid,
                name: p.name.clone(),
                user: p.user.clone(),
                rss_bytes: memory_mib * MIB,
                group_rss_bytes: memory_mib * MIB,
                cpu_percent: cpu,
                group_cpu_percent: cpu,
                can_terminate: p.pid != 1 && p.user == "kali",
            }
        })
        .collect();
    Snapshot {
        cpu_percent: tasks.iter().map(|p| p.cpu_percent).sum::<f64>().min(100.0),
        memory_used_bytes: (512 * MIB + tasks.iter().map(|p| p.rss_bytes).sum::<u64>())
            .min(8192 * MIB),
        memory_total_bytes: 8192 * MIB,
        metrics_kind: "virtual-budget-v1",
        tasks,
    }
}
pub fn terminate(
    world: &mut WorldState,
    pid: u32,
    expected_name: &str,
    force: bool,
) -> GameResult<()> {
    if !world
        .processes
        .iter()
        .any(|p| p.pid == pid && p.running && p.name == expected_name)
    {
        return Err(domain("Processo encerrado ou alterado; atualize a lista."));
    }
    crate::terminal::signal_local_process(world, pid, "kali", if force { 9 } else { 15 })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{service::GameService, terminal, world::VirtualProcess};
    use rusqlite::Connection;
    #[test]
    fn snapshot_is_read_only_local_and_deterministic() {
        let mut w = WorldState::new("neo", "pc").unwrap();
        w.processes.push(VirtualProcess {
            pid: 9,
            name: "worker".into(),
            user: "kali".into(),
            running: false,
        });
        let before = serde_json::to_string(&w).unwrap();
        let first = snapshot(&w);
        assert_eq!(first.tasks.len(), 1);
        assert_eq!(first.tasks[0].rss_bytes, 32 * MIB);
        assert!(!first.tasks[0].can_terminate);
        assert_eq!(first.memory_used_bytes, 544 * MIB);
        assert_eq!(first.metrics_kind, "virtual-budget-v1");
        assert_eq!(
            serde_json::to_string(&snapshot(&w)).unwrap(),
            serde_json::to_string(&first).unwrap()
        );
        assert_eq!(serde_json::to_string(&w).unwrap(), before);
    }
    #[test]
    fn termination_preserves_terminal_context_permissions_and_service_state() {
        let mut w = WorldState::new("neo", "pc").unwrap();
        terminal::execute(&mut w, "sudo service ssh start");
        let pid = w
            .processes
            .iter()
            .find(|p| p.name == "ssh" && p.running)
            .unwrap()
            .pid;
        assert!(terminate(&mut w, pid, "ssh", false).is_err());
        assert!(terminate(&mut w, 1, "lifeos-session", true).is_err());
        w.processes.iter_mut().find(|p| p.pid == pid).unwrap().user = "kali".into();
        w.terminal.host = Some("unrelated-ssh-context".into());
        let context = serde_json::to_string(&w.terminal).unwrap();
        assert!(terminate(&mut w, pid, "stale-name", false).is_err());
        terminate(&mut w, pid, "ssh", false).unwrap();
        assert!(!w.processes.iter().find(|p| p.pid == pid).unwrap().running);
        assert!(w.events.iter().any(|e| e.ends_with("signal 15")));
        assert_eq!(serde_json::to_string(&w.terminal).unwrap(), context);
        w.terminal.host = None;
        assert_eq!(
            terminal::execute(&mut w, "systemctl is-active ssh").exit_code,
            3
        );
    }
    #[test]
    fn failed_commit_does_not_publish_a_terminated_process() {
        let mut game = GameService::new(Connection::open_in_memory().unwrap()).unwrap();
        game.new_game(1, "neo", "pc", false).unwrap();
        game.mutate(
            |_, w, _| {
                w.processes.push(VirtualProcess {
                    pid: 9,
                    name: "worker".into(),
                    user: "kali".into(),
                    running: true,
                });
                Ok(())
            },
            false,
        )
        .unwrap();
        game.connection.execute_batch("CREATE TRIGGER reject_task BEFORE UPDATE ON save_slots BEGIN SELECT RAISE(ABORT,'test'); END;").unwrap();
        assert!(game
            .mutate(|_, w, _| terminate(w, 9, "worker", true), false)
            .is_err());
        assert!(
            game.world()
                .unwrap()
                .processes
                .iter()
                .find(|p| p.pid == 9)
                .unwrap()
                .running
        );
    }
}
