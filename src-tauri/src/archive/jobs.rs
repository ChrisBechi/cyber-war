//! Cooperative virtual jobs. Runtime state is never saved as a running host process.
use super::*;
use crate::{
    terminal,
    world::{TerminalSession, VirtualProcess, WorldState},
};
use parking_lot::Mutex;
use std::{
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        Arc, OnceLock,
    },
    time::{Duration, Instant},
};

static NEXT: AtomicU32 = AtomicU32::new(4800);
static CANCEL: OnceLock<Mutex<std::collections::BTreeMap<u32, Arc<AtomicBool>>>> = OnceLock::new();
thread_local! { static ACTIVE: std::cell::RefCell<Option<Arc<AtomicBool>>> = const { std::cell::RefCell::new(None) }; }
thread_local! { static DEFER_ALLOWED: std::cell::Cell<bool> = const { std::cell::Cell::new(true) }; }
pub fn with_deferral<T>(allowed: bool, f: impl FnOnce() -> T) -> T {
    let previous = DEFER_ALLOWED.with(|v| v.replace(allowed && v.get()));
    let result = f();
    DEFER_ALLOWED.with(|v| v.set(previous));
    result
}
pub fn may_defer() -> bool {
    DEFER_ALLOWED.with(|v| v.get()) && ACTIVE.with(|v| v.borrow().is_none())
}
pub fn check_cancel() -> GameResult<()> {
    if ACTIVE.with(|c| {
        c.borrow()
            .as_ref()
            .is_some_and(|c| c.load(Ordering::Relaxed))
    }) {
        Err(domain("archive: operation cancelled"))
    } else {
        Ok(())
    }
}
#[derive(Clone)]
pub enum Work {
    Package(crate::packages::model::Plan),
    Terminal(Vec<String>),
    Pending(cli::Pending),
    Gui(ipc::Request),
}
#[derive(Clone)]
pub struct Job {
    pub state: JobState,
    work: Work,
    session: TerminalSession,
    started: Instant,
    duration: Duration,
    cancelled: Arc<AtomicBool>,
    pub background: bool,
}
impl std::fmt::Debug for Job {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.state.fmt(f)
    }
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobState {
    pub id: u32,
    pub pid: u32,
    pub name: String,
    pub progress: u8,
    pub status: String,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub response: Option<ipc::Response>,
}
pub fn enqueue(
    world: &mut WorldState,
    work: Work,
    name: String,
    size: u64,
    background: bool,
) -> GameResult<JobState> {
    world
        .archive_jobs
        .retain(|j| j.state.status == "Running" || j.started.elapsed() < Duration::from_secs(60));
    if world.archive_jobs.len() >= 32 {
        return Err(domain("archive: too many jobs"));
    }
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    let cancelled = Arc::new(AtomicBool::new(false));
    CANCEL
        .get_or_init(Default::default)
        .lock()
        .insert(id, cancelled.clone());
    let performance = world
        .settings
        .get("virtualCPUPerformance")
        .and_then(|n| n.parse::<f64>().ok())
        .filter(|n| n.is_finite())
        .unwrap_or(1.0)
        .clamp(0.25, 4.0);
    let factor = if name.contains("xz") || name.contains('J') {
        2.0
    } else {
        1.0
    };
    let millis = ((size as f64 / (64.0 * 1024.0 * 1024.0)) * 1000.0 * factor / performance)
        .clamp(250.0, 8000.0) as u64;
    let state = JobState {
        id,
        pid: id,
        name: name.clone(),
        progress: 0,
        status: "Running".into(),
        stdout: String::new(),
        stderr: String::new(),
        exit_code: 0,
        response: None,
    };
    world.processes.push(VirtualProcess {
        pid: id,
        name,
        user: world.terminal.user.clone(),
        running: true,
    });
    world.archive_jobs.push(Job {
        state: state.clone(),
        work,
        session: world.terminal.clone(),
        started: Instant::now(),
        duration: Duration::from_millis(millis),
        cancelled,
        background,
    });
    Ok(state)
}
pub fn estimate(world: &WorldState, parts: &[String]) -> u64 {
    let Ok(fs) = world.fs() else { return 0 };
    parts
        .iter()
        .skip(1)
        .filter(|p| !p.starts_with('-'))
        .filter_map(|p| crate::vfs::normalize(p, &world.terminal.cwd).ok())
        .map(|p| {
            fs.nodes
                .iter()
                .filter(|(k, _)| **k == p || k.starts_with(&format!("{p}/")))
                .map(|(_, n)| {
                    n.metadata
                        .get("archiveOriginalSize")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or_else(|| n.logical_size())
                })
                .fold(0u64, u64::saturating_add)
        })
        .fold(0u64, u64::saturating_add)
}
pub fn maybe_start(world: &mut WorldState, parts: &[String]) -> Option<terminal::CommandResult> {
    let background = parts.last().is_some_and(|p| p == "\0&");
    let parts = if background {
        &parts[..parts.len() - 1]
    } else {
        parts
    };
    let name = parts.first()?.as_str();
    if !cli::COMMANDS.contains(&name) {
        return None;
    }
    // Foreground interactive commands retain their terminal's input context.
    // Explicit background jobs reject a prompt, as a detached terminal would.
    if !background
        && (name == "unzip"
            || (name == "zip"
                && parts
                    .iter()
                    .any(|p| p.starts_with('-') && (p.contains('e') || p == "--encrypt"))))
    {
        return None;
    }
    let size = estimate(world, parts);
    if !background
        && (size < 8 * 1024 * 1024 || name == "zcat" || parts.iter().any(|p| p.starts_with('\0')))
    {
        return None;
    }
    let mut result = terminal::execute_parts(world, Ok(Vec::new()));
    match enqueue(
        world,
        Work::Terminal(parts.to_vec()),
        parts.join(" "),
        size,
        background,
    ) {
        Ok(job) => {
            result.stdout = if background {
                format!("[{}] {}\n", job.id, job.pid)
            } else {
                String::new()
            };
            if !background {
                result.archive_job = Some(job.id);
            }
        }
        Err(e) => {
            result.stderr = format!("{e}\n");
            result.exit_code = 1;
        }
    }
    Some(result)
}
pub fn cancel(id: u32) {
    if let Some(c) = CANCEL.get_or_init(Default::default).lock().get(&id) {
        c.store(true, Ordering::Relaxed);
    }
}
pub fn reset(world: &mut WorldState) {
    world.package_lock = None;
    for job in world.archive_jobs.drain(..) {
        job.cancelled.store(true, Ordering::Relaxed);
        CANCEL
            .get_or_init(Default::default)
            .lock()
            .remove(&job.state.id);
    }
    // Jobs are runtime-only, including process rows restored from an older save.
    world.processes.retain(|p| {
        !(p.pid >= 4800
            && p.name
                .split_whitespace()
                .next()
                .is_some_and(|n| n == "archive" || n == "package" || cli::COMMANDS.contains(&n)))
    });
}
pub fn tick(world: &mut WorldState) -> Vec<JobState> {
    let mut jobs = std::mem::take(&mut world.archive_jobs);
    for job in &mut jobs {
        if job.state.status != "Running" {
            continue;
        }
        if !world
            .processes
            .iter()
            .any(|p| p.pid == job.state.pid && p.running)
        {
            job.cancelled.store(true, Ordering::Relaxed);
        }
        let cancelled = job.cancelled.load(Ordering::Relaxed);
        job.state.progress =
            ((job.started.elapsed().as_millis() * 100 / job.duration.as_millis()).min(95)) as u8;
        if !cancelled && job.started.elapsed() < job.duration {
            continue;
        }
        if cancelled {
            if let Work::Package(plan) = &job.work {
                crate::packages::service::record_cancellation(world, plan, 130);
            }
            job.state.stderr = "archive: operation cancelled\n".into();
            job.state.exit_code = 130;
            job.state.status = "Cancelled".into();
        } else {
            let saved = std::mem::replace(&mut world.terminal, job.session.clone());
            ACTIVE.with(|a| *a.borrow_mut() = Some(job.cancelled.clone()));
            match &job.work {
                Work::Package(plan) => {
                    let output = match crate::packages::service::apply(world, plan) {
                        Ok(output) => output,
                        Err(error) => crate::packages::cli::failure(
                            if plan.dpkg { "dpkg" } else { "apt" },
                            error,
                        ),
                    };
                    job.state.stdout = output.stdout;
                    job.state.stderr = output.stderr;
                    job.state.exit_code = output.status;
                }
                Work::Pending(pending) => {
                    match cli::run(world, pending.clone()) {
                        Ok(output) => {
                            job.state.stdout = output.stdout;
                            job.state.stderr = output.stderr;
                            job.state.exit_code = output.status;
                        }
                        Err(error) => {
                            job.state.stderr = error.to_string();
                            job.state.exit_code = 1;
                        }
                    }
                    if world.terminal.archive_pending.take().is_some() {
                        job.state.stderr =
                            "archive: files changed while waiting; run the command again\n".into();
                        job.state.exit_code = 1;
                    }
                }
                Work::Terminal(parts) => {
                    let result = terminal::execute_parts(world, Ok(parts.clone()));
                    if world.terminal.archive_pending.is_some() {
                        world.terminal.archive_pending = None;
                        job.state.stderr="archive: interactive input unavailable in jobs; use -o/-n or foreground input\n".into();
                        job.state.exit_code = 1;
                    } else {
                        job.state.stdout = result.stdout;
                        job.state.stderr = result.stderr;
                        job.state.exit_code = result.exit_code;
                    }
                }
                Work::Gui(request) => {
                    let result = ipc::operate(world, request.clone());
                    if let Some(e) = &result.error {
                        job.state.stderr = e.clone();
                        job.state.exit_code = 1;
                    }
                    job.state.response = Some(result);
                }
            }
            ACTIVE.with(|a| *a.borrow_mut() = None);
            world.terminal = saved;
            job.state.status = if job.cancelled.load(Ordering::Relaxed) {
                "Cancelled"
            } else if job.state.exit_code == 0 {
                "Done"
            } else {
                "Failed"
            }
            .into();
            job.state.progress = 100;
        }
        if matches!(job.work, Work::Package(_)) {
            world.package_lock = None;
        }
        if job.state.status == "Cancelled" {
            job.state.exit_code = 130;
        }
        for session in
            std::iter::once(&mut world.terminal).chain(world.terminal_sessions.values_mut())
        {
            if session.package_job == Some(job.state.id) {
                session.last_status = job.state.exit_code;
                session.package_job = None;
            }
        }
        if let Some(p) = world.processes.iter_mut().find(|p| p.pid == job.state.pid) {
            p.running = false;
        }
        CANCEL
            .get_or_init(Default::default)
            .lock()
            .remove(&job.state.id);
    }
    let states = jobs.iter().map(|j| j.state.clone()).collect();
    world.archive_jobs = jobs;
    states
}
pub fn listing(world: &WorldState) -> String {
    world
        .archive_jobs
        .iter()
        .filter(|j| j.background)
        .map(|j| format!("[{}] {} {}\n", j.state.id, j.state.status, j.state.name))
        .collect()
}
pub fn due(world: &WorldState) -> bool {
    world.archive_jobs.iter().any(|j| {
        j.state.status == "Running"
            && (j.started.elapsed() >= j.duration
                || j.cancelled.load(Ordering::Relaxed)
                || !world
                    .processes
                    .iter()
                    .any(|p| p.pid == j.state.pid && p.running))
    })
}
pub fn snapshot(world: &WorldState) -> Vec<JobState> {
    world
        .archive_jobs
        .iter()
        .map(|j| {
            let mut s = j.state.clone();
            if s.status == "Running" {
                s.progress = ((j.started.elapsed().as_millis() * 100 / j.duration.as_millis())
                    .min(95)) as u8;
            }
            s
        })
        .collect()
}
