use super::{deb, model::Pending, service, state};
use crate::{
    archive,
    error::GameResult,
    service::GameService,
    vfs::{domain, normalize},
    world::WorldState,
};
use parking_lot::Mutex;
use serde::Serialize;
use tauri::{Manager, State};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Inspection {
    name: String,
    version: String,
    architecture: String,
    description: String,
    installed_size: u64,
    download_size: u64,
    dependencies: Vec<String>,
    origin: String,
    status: String,
    valid: bool,
}
pub fn inspect(world: &WorldState, path: &str) -> GameResult<Inspection> {
    let path = normalize(path, crate::vfs::HOME)?;
    let bytes = archive::bytes(world, &path, "kali")?;
    let p = deb::decode(&bytes)?;
    let installed = world.packages.installed.get(&p.name);
    let origin = super::repository::REPOSITORIES
        .values()
        .flat_map(|r| r.entries.iter())
        .find(|(_, e)| e.checksum == deb::hash(&bytes))
        .map(|(_, e)| e.repository.clone())
        .unwrap_or_else(|| "Local virtual package".into());
    Ok(Inspection {
        name: p.name.clone(),
        version: p.version.clone(),
        architecture: p.architecture.clone(),
        description: p.description.clone(),
        installed_size: p.installed_size(),
        download_size: p.download_size,
        dependencies: p
            .depends
            .iter()
            .map(|clause| {
                clause
                    .iter()
                    .map(|r| format!("{} {} {}", r.name, r.op, r.version))
                    .collect::<Vec<_>>()
                    .join(" | ")
            })
            .collect(),
        origin,
        status: installed
            .map_or("not-installed", |i| i.status.label())
            .into(),
        valid: true,
    })
}
#[tauri::command]
pub fn package_inspect(
    path: String,
    service: State<'_, Mutex<GameService>>,
) -> GameResult<Inspection> {
    inspect(service.lock().world()?, &path)
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanSummary {
    id: String,
    summary: String,
}
#[tauri::command]
pub fn package_plan(
    path: String,
    operation: String,
    service: State<'_, Mutex<GameService>>,
) -> GameResult<PlanSummary> {
    if !["install", "remove", "purge"].contains(&operation.as_str()) {
        return Err(domain("unsupported package operation"));
    }
    service.lock().mutate(
        |_, world, _| {
            let info = inspect(world, &path)?;
            let request = if operation == "install" {
                path.clone()
            } else {
                info.name
            };
            let plan = service::plan(world, &operation, &[request], true)?;
            let response = PlanSummary {
                id: plan.id.clone(),
                summary: service::summary(&plan),
            };
            world.package_lock = Some(plan.id.clone());
            world.terminal.package_pending = Some(Pending {
                plan,
                actor: "root".into(),
            });
            Ok(response)
        },
        false,
    )
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationResult {
    stdout: String,
    stderr: String,
    exit_code: i32,
    job: Option<u32>,
}
#[tauri::command]
pub async fn package_confirm(
    id: String,
    accept: bool,
    app: tauri::AppHandle,
) -> GameResult<OperationResult> {
    tauri::async_runtime::spawn_blocking(move || {
        app.state::<Mutex<GameService>>().lock().mutate(
            |_, world, _| {
                if world
                    .terminal
                    .package_pending
                    .as_ref()
                    .is_none_or(|p| p.plan.id != id)
                {
                    return Err(domain("package plan expired"));
                }
                let result = super::cli::respond(world, if accept { "y" } else { "n" }, false)
                    .unwrap_or_else(|error| super::cli::failure("dpkg", error));
                Ok(OperationResult {
                    stdout: result.stdout,
                    stderr: result.stderr,
                    exit_code: result.status,
                    job: result.archive_job,
                })
            },
            false,
        )
    })
    .await
    .map_err(|e| domain(e.to_string()))?
}
pub fn download(
    world: &mut WorldState,
    url: &str,
    destination: &str,
    actor: &str,
) -> GameResult<String> {
    let entry = download_entry(url).ok_or_else(|| domain("virtual package download not found"))?;
    let bytes = super::repository::artifact(&entry, world)?;
    let path = normalize(destination, &world.terminal.cwd)?;
    let path = world.vfs.available_path(&path, false);
    archive::write_bytes(world, &path, bytes, actor, entry.package.download_size)?;
    service::set_deb_mime(world, &path);
    Ok(path)
}
pub fn download_entry(url: &str) -> Option<super::model::IndexEntry> {
    super::repository::REPOSITORIES
        .values()
        .flat_map(|r| r.entries.iter())
        .find(|(_, e)| url == format!("{}/pool/{}.deb", e.repository, e.package.key()))
        .map(|(_, e)| e.clone())
}
#[tauri::command]
pub fn package_launch(path: String, service: State<'_, Mutex<GameService>>) -> GameResult<String> {
    let service = service.lock();
    let world = service.world()?;
    let node = world.vfs.readable(&path, "kali")?;
    if !path.starts_with("/usr/share/applications/")
        || !path.ends_with(".desktop")
        || state::content_hash(world, &path).as_deref()
            != world
                .packages
                .ownership
                .get(&path)
                .map(|o| o.shipped_hash.as_str())
    {
        return Err(domain("unregistered desktop entry"));
    }
    let exec = node
        .content
        .lines()
        .find_map(|l| l.strip_prefix("Exec="))
        .ok_or_else(|| domain("missing desktop Exec"))?;
    if exec.contains(char::is_whitespace)
        || super::executables::resolve(world, exec, "kali").is_none()
    {
        return Err(domain("application executable is unavailable"));
    }
    Ok(exec.into())
}
