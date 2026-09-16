use super::*;
use crate::{
    error::GameResult,
    service::GameService,
    terminal,
    vfs::{normalize, HOME},
    world::WorldState,
};
use parking_lot::Mutex;
use tauri::Manager;

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub operation: String,
    pub path: String,
    pub format: Option<ArchiveFormat>,
    #[serde(default)]
    pub inputs: Vec<String>,
    pub options: Option<ExtractOptions>,
    pub password: Option<String>,
    #[serde(default)]
    pub as_root: bool,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub inspection: Option<ArchiveInspection>,
    pub entries: Vec<String>,
    pub error: Option<String>,
}

#[tauri::command]
pub fn archive_job_cancel(id: u32) {
    jobs::cancel(id);
}
#[tauri::command]
pub async fn archive_job_start(
    request: Request,
    app: tauri::AppHandle,
) -> GameResult<jobs::JobState> {
    tauri::async_runtime::spawn_blocking(move || {
        app.state::<Mutex<GameService>>().lock().mutate(
            |_, world, _| {
                let size = world
                    .vfs
                    .nodes
                    .iter()
                    .filter(|(p, _)| {
                        request
                            .inputs
                            .iter()
                            .any(|input| *p == input || p.starts_with(&format!("{input}/")))
                    })
                    .map(|(_, n)| n.logical_size())
                    .fold(0u64, u64::saturating_add)
                    .max(world.vfs.nodes.get(&request.path).map_or(0, |n| {
                        n.metadata
                            .get("archiveOriginalSize")
                            .and_then(|n| n.parse().ok())
                            .unwrap_or_else(|| n.logical_size())
                    }));
                jobs::enqueue(
                    world,
                    jobs::Work::Gui(request.clone()),
                    format!("archive {}", request.operation),
                    size,
                    false,
                )
            },
            false,
        )
    })
    .await
    .map_err(|e| domain(e.to_string()))?
}
#[tauri::command]
pub async fn archive_jobs_tick(app: tauri::AppHandle) -> GameResult<Vec<jobs::JobState>> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<Mutex<GameService>>();
        let mut service = state.lock();
        if jobs::due(service.world()?) {
            service.mutate(|_, w, _| Ok(jobs::tick(w)), false)
        } else {
            Ok(jobs::snapshot(service.world()?))
        }
    })
    .await
    .map_err(|e| domain(e.to_string()))?
}
pub fn operate(world: &mut WorldState, request: Request) -> Response {
    let actor = if request.as_root { "root" } else { "kali" };
    let saved = std::mem::replace(
        &mut world.terminal,
        crate::terminal_sessions::fresh_session(),
    );
    let service = ArchiveService::default();
    let result = (|| -> GameResult<Response> {
        let path = normalize(&request.path, HOME)?;
        if crate::vfs::VirtualFileSystem::is_trash_path(&path) {
            return Err(domain("restore trash items before opening"));
        }
        let mut response = Response {
            inspection: None,
            entries: Vec::new(),
            error: None,
        };
        match request.operation.as_str() {
            "inspect" => {
                let info = service.inspect_archive(world, &path, actor)?;
                event(
                    world,
                    "ARCHIVE_OPENED",
                    &path,
                    None,
                    Vec::new(),
                    Some(info.format),
                );
                response.inspection = Some(info);
            }
            "create" => {
                // File Manager selections share a parent. Store names relative
                // to that parent instead of embedding /home/kali in every ZIP.
                let mut base = request
                    .inputs
                    .first()
                    .map(|p| crate::vfs::parent(p).to_string())
                    .unwrap_or_else(|| HOME.into());
                while base != "/"
                    && request
                        .inputs
                        .iter()
                        .any(|p| !p.starts_with(&format!("{base}/")))
                {
                    base = crate::vfs::parent(&base).to_string();
                }
                let prefix = format!("{}/", base.trim_end_matches('/'));
                let inputs: Vec<String> = request
                    .inputs
                    .iter()
                    .map(|p| p.strip_prefix(&prefix).unwrap_or(p).to_string())
                    .collect();
                world.terminal.cwd = base;
                response.inspection = Some(
                    service.create_archive(
                        world,
                        &path,
                        &inputs,
                        request
                            .format
                            .ok_or_else(|| domain("archive: format required"))?,
                        true,
                        request.password.as_deref(),
                        actor,
                    )?,
                );
            }
            "extract" => {
                response.entries = service.extract_archive(
                    world,
                    &path,
                    &request
                        .options
                        .ok_or_else(|| domain("archive: extraction options required"))?,
                    request.password.as_deref(),
                    actor,
                )?;
            }
            "test" => {
                response.entries = service
                    .test_archive(world, &path, request.password.as_deref(), actor)?
                    .into_iter()
                    .map(|e| e.path)
                    .collect();
            }
            "decompress" => {
                let format = service
                    .inspect_archive(world, &path, actor)?
                    .format
                    .stream();
                let dest = request
                    .options
                    .as_ref()
                    .map(|o| normalize(&o.destination, HOME))
                    .transpose()?;
                service.decompress_file_to(
                    world,
                    &path,
                    format,
                    true,
                    request
                        .options
                        .as_ref()
                        .is_some_and(|o| matches!(o.overwrite, Overwrite::Replace)),
                    false,
                    actor,
                    dest.as_deref(),
                )?;
            }
            _ => return Err(domain("archive: unknown operation")),
        }
        Ok(response)
    })();
    world.terminal = saved;
    match result {
        Ok(response) => response,
        Err(e) => {
            let error = e.to_string();
            record_failure(world, &request.path, &error, request.format);
            Response {
                inspection: None,
                entries: Vec::new(),
                error: Some(error),
            }
        }
    }
}

#[tauri::command]
pub async fn archive_operation(request: Request, app: tauri::AppHandle) -> GameResult<Response> {
    tauri::async_runtime::spawn_blocking(move || {
        app.state::<Mutex<GameService>>()
            .lock()
            .mutate(|_, w, _| Ok(operate(w, request)), false)
    })
    .await
    .map_err(|e| domain(e.to_string()))?
}
pub fn input(world: &mut WorldState, value: &str, cancel: bool) -> terminal::CommandResult {
    let result = if world.terminal.package_pending.is_some() {
        crate::packages::cli::respond(world, value, cancel)
    } else {
        cli::respond(world, value, cancel)
    };
    let mut output = terminal::execute_parts(world, Ok(Vec::new()));
    match result {
        Ok(result) => {
            output.stdout = result.stdout;
            output.stderr = result.stderr;
            output.exit_code = result.status;
            output.archive_job = result.archive_job;
        }
        Err(error) => {
            output.stderr = format!("{error}\n");
            output.exit_code = 1;
        }
    }
    output.archive_prompt = world
        .terminal
        .archive_pending
        .as_ref()
        .map(|p| p.prompt.clone())
        .or_else(|| {
            world
                .terminal
                .package_pending
                .as_ref()
                .map(|_| cli::Prompt {
                    message: "Do you want to continue? [Y/n] ".into(),
                    secret: false,
                })
        });
    world.terminal.last_status = output.exit_code;
    output
}
#[tauri::command]
pub async fn archive_terminal_input(
    value: String,
    cancel: bool,
    session_id: Option<String>,
    app: tauri::AppHandle,
) -> GameResult<terminal::CommandResult> {
    tauri::async_runtime::spawn_blocking(move || {
        app.state::<Mutex<GameService>>().lock().mutate(
            |_, w, _| {
                crate::terminal_sessions::with_session(w, session_id.as_deref(), |w| {
                    Ok(input(w, &value, cancel))
                })
            },
            false,
        )
    })
    .await
    .map_err(|e| domain(e.to_string()))?
}
