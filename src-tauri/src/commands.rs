use crate::{
    error::GameResult,
    mission::CheckpointEvent,
    save,
    service::GameService,
    terminal,
    vfs::{domain, normalize, VfsNode, HOME},
    world::WorldState,
};
use parking_lot::Mutex;
use serde::Serialize;
use tauri::{Manager, State};

type Service<'a> = State<'a, Mutex<GameService>>;

#[tauri::command]
pub fn attachment_download(
    message_id: String,
    index: usize,
    service: Service<'_>,
) -> GameResult<String> {
    service.lock().mutate(
        |_, w, _| crate::archive::downloads::attachment(w, &message_id, index),
        false,
    )
}

#[tauri::command]
pub fn task_manager_snapshot(service: Service<'_>) -> GameResult<crate::task_manager::Snapshot> {
    Ok(crate::task_manager::snapshot(service.lock().world()?))
}

#[tauri::command]
pub fn task_manager_terminate(
    pid: u32,
    expected_name: String,
    force: bool,
    service: Service<'_>,
) -> GameResult<()> {
    service.lock().mutate(
        |_, world, _| crate::task_manager::terminate(world, pid, &expected_name, force),
        false,
    )
}

#[tauri::command]
pub fn wireless_scan(service: Service<'_>) -> GameResult<Vec<crate::network::VirtualWifi>> {
    crate::investigation::scan(service.lock().world()?)
}
#[tauri::command]
pub fn wireless_inspect(
    bssid: String,
    service: Service<'_>,
) -> GameResult<crate::investigation::SignalEvidence> {
    service
        .lock()
        .mutate(|_, w, _| crate::investigation::inspect(w, &bssid), false)
}
#[tauri::command]
pub fn traffic_read(
    path: String,
    service: Service<'_>,
) -> GameResult<Vec<crate::investigation::Packet>> {
    crate::investigation::packets(service.lock().world()?, &path)
}
#[tauri::command]
pub fn traffic_follow(
    path: String,
    stream: u32,
    service: Service<'_>,
) -> GameResult<Vec<crate::investigation::Packet>> {
    service.lock().mutate(
        |_, w, _| crate::investigation::follow_stream(w, &path, stream),
        false,
    )
}

#[tauri::command]
pub fn settings_global_get(service: Service<'_>) -> GameResult<crate::app_settings::AppSettings> {
    crate::app_settings::load(&service.lock().connection)
}
#[tauri::command]
pub fn settings_global_save(
    settings: crate::app_settings::AppSettings,
    service: Service<'_>,
) -> GameResult<crate::app_settings::AppSettings> {
    crate::app_settings::save(&service.lock().connection, &settings)
}
#[tauri::command]
pub fn quit_game(app: tauri::AppHandle, service: Service<'_>) -> GameResult<()> {
    crate::shell::control::cancel_all();
    service.lock().end_session()?;
    app.exit(0);
    Ok(())
}

#[tauri::command]
pub fn system_health() -> &'static str {
    "LifeOS · Rust core online · virtual network only"
}
#[tauri::command]
pub fn world_get(service: Service<'_>) -> GameResult<WorldState> {
    Ok(service.lock().world()?.clone())
}
#[tauri::command]
pub fn new_game(
    slot_index: i64,
    nickname: String,
    hostname: String,
    overwrite: bool,
    service: Service<'_>,
) -> GameResult<WorldState> {
    service
        .lock()
        .new_game(slot_index, &nickname, &hostname, overwrite)
}
#[tauri::command]
pub fn list_save_slots(service: Service<'_>) -> GameResult<Vec<save::SaveSlotSummary>> {
    save::list(&service.lock().connection)
}
#[tauri::command]
pub fn save_slot(service: Service<'_>) -> GameResult<()> {
    service.lock().save(true)
}
#[tauri::command]
pub fn autosave(service: Service<'_>) -> GameResult<()> {
    service.lock().save(false)
}
#[tauri::command]
pub fn session_start(service: Service<'_>) -> GameResult<WorldState> {
    service.lock().start_session()
}
#[tauri::command]
pub fn end_session(service: Service<'_>) -> GameResult<()> {
    crate::shell::control::cancel_all();
    service.lock().end_session()
}
#[tauri::command]
pub fn terminal_open(
    session_id: String,
    cwd: Option<String>,
    as_root: Option<bool>,
    service: Service<'_>,
) -> GameResult<crate::world::TerminalSession> {
    let mut game = service.lock();
    let active = game
        .active
        .as_mut()
        .ok_or_else(|| domain("no active campaign"))?;
    if active.session_ended {
        return Err(domain(
            "Sessão encerrada. Entre novamente para abrir um terminal.",
        ));
    }
    crate::terminal_sessions::open_at(
        &mut active.world,
        &session_id,
        cwd.as_deref(),
        as_root.unwrap_or(false),
    )
}
#[tauri::command]
pub fn terminal_close(session_id: String, service: Service<'_>) -> GameResult<()> {
    crate::shell::control::cancel(&session_id);
    let mut game = service.lock();
    if let Some(active) = game.active.as_mut() {
        crate::terminal_sessions::close(&mut active.world, &session_id)?;
    }
    Ok(())
}
#[tauri::command]
pub fn system_setup_complete(service: Service<'_>) -> GameResult<()> {
    service.lock().commit_system_setup()
}
#[tauri::command]
pub fn load_slot(slot_index: i64, manual: bool, service: Service<'_>) -> GameResult<WorldState> {
    crate::shell::control::cancel_all();
    service.lock().load(slot_index, manual)
}
#[tauri::command]
pub fn list_checkpoints(
    slot_index: i64,
    service: Service<'_>,
) -> GameResult<Vec<save::CheckpointSummary>> {
    save::checkpoints(&service.lock().connection, slot_index)
}
#[tauri::command]
pub fn restore_checkpoint(checkpoint_id: String, service: Service<'_>) -> GameResult<WorldState> {
    crate::shell::control::cancel_all();
    service.lock().restore(&checkpoint_id)
}
#[tauri::command]
pub fn create_mission_checkpoint(mission_id: String, service: Service<'_>) -> GameResult<()> {
    service.lock().mutate(
        |_, world, events| {
            if !world.missions.contains_key(&mission_id) {
                return Err(domain("mission not found"));
            }
            events.push(CheckpointEvent {
                kind: "autosave",
                mission: mission_id,
                label: "Checkpoint manual".into(),
                state: world.clone(),
            });
            Ok(())
        },
        false,
    )
}
#[tauri::command]
pub async fn execute_terminal(
    command: String,
    session_id: Option<String>,
    presentation: Option<crate::system_info::Presentation>,
    output: Option<tauri::ipc::JavaScriptChannelId>,
    webview: tauri::Webview,
    app: tauri::AppHandle,
) -> GameResult<terminal::CommandResult> {
    let output = output.map(|channel| channel.channel_on(webview));
    let control =
        crate::shell::control::register_output(session_id.as_deref().unwrap_or("default"), output);
    tauri::async_runtime::spawn_blocking(move || {
        crate::shell::control::run(&control, || {
            app.state::<Mutex<GameService>>().lock().mutate(
                |_, world, _| {
                    crate::terminal_sessions::with_session(world, session_id.as_deref(), |world| {
                        if let Some(presentation) = presentation {
                            world.terminal.presentation = presentation;
                        }
                        Ok(terminal::execute_interactive(world, &command))
                    })
                },
                false,
            )
        })
    })
    .await
    .map_err(|e| domain(e.to_string()))?
}
#[tauri::command]
pub async fn terminal_input(session_id: String, value: Option<String>, cancel: bool) -> bool {
    if cancel {
        crate::shell::control::cancel(&session_id);
        true
    } else {
        crate::shell::control::input(&session_id, value)
    }
}
#[tauri::command]
pub async fn terminal_output_ack(session_id: String, bytes: usize) {
    crate::shell::control::acknowledge(&session_id, bytes);
}
#[tauri::command]
pub async fn terminal_cancel_all() {
    crate::shell::control::cancel_all();
}
#[tauri::command]
pub fn terminal_complete(
    line: String,
    cursor: usize,
    session_id: Option<String>,
    service: Service<'_>,
) -> GameResult<crate::completion::Completion> {
    let mut world = service.lock().world()?.clone();
    crate::terminal_sessions::with_session(&mut world, session_id.as_deref(), |world| {
        crate::completion::complete(world, &line, cursor)
    })
}

#[tauri::command]
pub fn nano_write(
    path: String,
    content: String,
    expected_content: Option<String>,
    session_id: Option<String>,
    service: Service<'_>,
) -> GameResult<()> {
    service.lock().mutate(
        |_, world, _| {
            crate::terminal_sessions::with_session(world, session_id.as_deref(), |world| {
                let session = world
                    .terminal
                    .nano
                    .clone()
                    .ok_or_else(|| domain("nano: no active editor"))?;
                let path = normalize(
                    &path,
                    session
                        .options
                        .operating_dir
                        .as_deref()
                        .unwrap_or(&world.terminal.cwd),
                )?;
                if session.options.view {
                    return Err(domain("nano: file is read-only"));
                }
                let path = world.fs()?.resolve_missing(
                    &path,
                    &session.actor,
                    crate::vfs::Follow::Yes,
                    true,
                )?;
                crate::nano::validate_access(&session, &path)?;
                if session.options.trim_blanks {
                    let normalized = content
                        .lines()
                        .map(str::trim_end)
                        .collect::<Vec<_>>()
                        .join("\n");
                    let content = if content.ends_with('\n') {
                        format!("{normalized}\n")
                    } else {
                        normalized
                    };
                    let expected = expected_for_save(world, &session, &path, expected_content)?;
                    return write_nano_buffer(
                        world,
                        &session,
                        &path,
                        &content,
                        expected.as_deref(),
                    );
                }
                let expected = expected_for_save(world, &session, &path, expected_content)?;
                write_nano_buffer(world, &session, &path, &content, expected.as_deref())
            })
        },
        false,
    )
}

fn expected_for_save(
    world: &WorldState,
    session: &crate::nano::NanoSession,
    path: &str,
    expected: Option<String>,
) -> GameResult<Option<String>> {
    if path == session.path {
        return Ok(expected);
    }
    match world.fs()?.read(path, &session.actor) {
        Ok(content) => Ok(Some(content)),
        Err(crate::error::GameError::Vfs(crate::vfs::Errno::NotFound)) => Ok(None),
        Err(error) => Err(error),
    }
}

fn write_nano_buffer(
    world: &mut WorldState,
    session: &crate::nano::NanoSession,
    path: &str,
    content: &str,
    expected: Option<&str>,
) -> GameResult<()> {
    let actor = session.actor.clone();
    if session.options.view {
        return Err(domain("nano: file is read-only"));
    }
    crate::nano::validate_access(session, path)?;
    let content = if !session.options.no_newlines && !content.is_empty() && !content.ends_with('\n')
    {
        format!("{content}\n")
    } else {
        content.to_owned()
    };
    if session.options.backup && !session.options.restricted {
        let backup_path = if let Some(directory) = session.options.backup_dir.as_deref() {
            let root = normalize(
                directory,
                session
                    .options
                    .operating_dir
                    .as_deref()
                    .unwrap_or(&world.terminal.cwd),
            )?;
            world.fs()?.directory(&root, &actor)?;
            let root = world
                .fs()?
                .resolve(&root, &actor, crate::vfs::Follow::Yes)?;
            let stem = format!(
                "{}/{}",
                root.trim_end_matches('/'),
                path.rsplit('/').next().unwrap_or("buffer")
            );
            (1..=10000)
                .map(|n| format!("{stem}~{n}~"))
                .find(|p| !world.fs().is_ok_and(|fs| fs.nodes.contains_key(p)))
                .ok_or_else(|| domain("nano: backup directory limit exceeded"))?
        } else {
            format!("{path}~")
        };
        let backup_path =
            world
                .fs()?
                .resolve_missing(&backup_path, &actor, crate::vfs::Follow::Yes, true)?;
        crate::nano::validate_access(session, &backup_path)?;
        let previous = if path == session.path {
            session.original_content.clone()
        } else {
            world.fs()?.read(path, &actor).ok()
        };
        if let Some(previous) = previous {
            world.fs_mut()?.write(&backup_path, &previous, &actor)?;
        }
    }
    world
        .fs_mut()?
        .write_checked(path, &content, expected, &actor)?;
    if let Some(active) = &mut world.terminal.nano {
        active.path = path.into();
        active.display_name = active.path.clone();
        active.original_content = Some(content);
    }
    Ok(())
}

#[tauri::command]
pub fn nano_read(
    path: String,
    session_id: Option<String>,
    service: Service<'_>,
) -> GameResult<String> {
    let mut world = service.lock().world()?.clone();
    crate::terminal_sessions::with_session(&mut world, session_id.as_deref(), |world| {
        read_nano_file(world, &path)
    })
}
fn read_nano_file(world: &WorldState, path: &str) -> GameResult<String> {
    let session = world
        .terminal
        .nano
        .as_ref()
        .ok_or_else(|| domain("nano: no active editor"))?;
    let path = normalize(
        path,
        session
            .options
            .operating_dir
            .as_deref()
            .unwrap_or(&world.terminal.cwd),
    )?;
    let path = world
        .fs()?
        .resolve(&path, &session.actor, crate::vfs::Follow::Yes)?;
    crate::nano::validate_access(session, &path)?;
    world.fs()?.read(&path, &session.actor)
}

#[tauri::command]
pub fn nano_close(session_id: Option<String>, service: Service<'_>) -> GameResult<()> {
    service.lock().mutate(
        |_, world, _| {
            crate::terminal_sessions::with_session(world, session_id.as_deref(), |world| {
                world.terminal.nano = None;
                world.terminal.foreground = None;
                Ok(())
            })
        },
        false,
    )
}
#[tauri::command]
pub fn desktop_create(
    kind: crate::desktop::ItemKind,
    name: String,
    target: Option<String>,
    service: Service<'_>,
) -> GameResult<()> {
    service.lock().mutate(
        |_, w, _| crate::desktop::create(w, kind, &name, target.as_deref().unwrap_or("")),
        false,
    )
}

// Root here is a virtual actor, never the host account or a native privilege request.
fn file_actor(as_root: Option<bool>) -> &'static str {
    if as_root == Some(true) {
        "root"
    } else {
        "kali"
    }
}

#[tauri::command]
pub fn vfs_list(
    path: String,
    as_root: Option<bool>,
    service: Service<'_>,
) -> GameResult<Vec<VfsNode>> {
    service
        .lock()
        .world()?
        .vfs
        .list(&normalize(&path, HOME)?, file_actor(as_root))
}
#[tauri::command]
pub fn vfs_read(path: String, as_root: Option<bool>, service: Service<'_>) -> GameResult<String> {
    let path = normalize(&path, HOME)?;
    if crate::vfs::VirtualFileSystem::is_trash_path(&path) {
        return Err(domain(
            "itens na lixeira precisam ser restaurados antes de abrir",
        ));
    }
    service.lock().world()?.vfs.read(&path, file_actor(as_root))
}
#[tauri::command]
pub fn vfs_read_bytes(
    path: String,
    as_root: Option<bool>,
    service: Service<'_>,
) -> GameResult<crate::binary::BinaryRead> {
    crate::binary::read(service.lock().world()?, &path, file_actor(as_root))
}
#[tauri::command]
pub fn vfs_import_bytes(
    path: String,
    base64: String,
    mime: String,
    expected_modified: Option<u64>,
    as_root: Option<bool>,
    service: Service<'_>,
) -> GameResult<()> {
    service.lock().mutate(
        |_, world, _| {
            crate::binary::import(
                world,
                &path,
                &base64,
                &mime,
                expected_modified,
                file_actor(as_root),
            )
        },
        false,
    )
}
#[tauri::command]
pub fn vfs_stat(path: String, as_root: Option<bool>, service: Service<'_>) -> GameResult<VfsNode> {
    Ok(service
        .lock()
        .world()?
        .vfs
        .stat(&normalize(&path, HOME)?, file_actor(as_root))?
        .clone())
}
#[tauri::command]
pub fn vfs_write(
    path: String,
    content: String,
    expected_content: Option<String>,
    as_root: Option<bool>,
    service: Service<'_>,
) -> GameResult<()> {
    service.lock().mutate(
        |_, w, _| {
            w.vfs.write_checked(
                &normalize(&path, HOME)?,
                &content,
                expected_content.as_deref(),
                file_actor(as_root),
            )
        },
        false,
    )
}
#[tauri::command]
pub fn vfs_create_file(
    path: String,
    as_root: Option<bool>,
    service: Service<'_>,
) -> GameResult<()> {
    service.lock().mutate(
        |_, w, _| {
            let path = normalize(&path, HOME)?;
            if w.vfs.nodes.contains_key(&path) {
                return Err(domain("EEXIST: arquivo já existe"));
            }
            w.vfs.write(&path, "", file_actor(as_root))
        },
        false,
    )
}
#[tauri::command]
pub fn vfs_copy(
    source: String,
    destination: String,
    as_root: Option<bool>,
    service: Service<'_>,
) -> GameResult<()> {
    service.lock().mutate(
        |_, w, _| {
            w.vfs.copy_unique(
                &normalize(&source, HOME)?,
                &normalize(&destination, HOME)?,
                file_actor(as_root),
            )
        },
        false,
    )
}
#[tauri::command]
pub fn vfs_create_directory(
    path: String,
    as_root: Option<bool>,
    service: Service<'_>,
) -> GameResult<()> {
    service.lock().mutate(
        |_, w, _| w.vfs.mkdir(&normalize(&path, HOME)?, file_actor(as_root)),
        false,
    )
}
#[tauri::command]
pub fn vfs_move(
    source: String,
    destination: String,
    as_root: Option<bool>,
    service: Service<'_>,
) -> GameResult<()> {
    service.lock().mutate(
        |_, w, _| {
            w.vfs.transfer(
                &normalize(&source, HOME)?,
                &normalize(&destination, HOME)?,
                file_actor(as_root),
                true,
                true,
            )
        },
        false,
    )
}
#[tauri::command]
pub fn vfs_remove(
    path: String,
    recursive: bool,
    as_root: Option<bool>,
    service: Service<'_>,
) -> GameResult<()> {
    let path = normalize(&path, HOME)?;
    if crate::vfs::VirtualFileSystem::is_trash_container(&path) {
        return Err(domain("a estrutura da lixeira não pode ser removida"));
    }
    service.lock().mutate(
        |_, w, _| {
            if crate::vfs::VirtualFileSystem::is_trash_path(&path) {
                w.vfs.remove(&path, file_actor(as_root), recursive)
            } else {
                w.vfs.move_to_trash(&path, file_actor(as_root), recursive)
            }
        },
        false,
    )
}
#[tauri::command]
pub fn vfs_trash_restore(
    path: String,
    as_root: Option<bool>,
    service: Service<'_>,
) -> GameResult<()> {
    let path = normalize(&path, HOME)?;
    service.lock().mutate(
        |_, w, _| w.vfs.restore_from_trash(&path, file_actor(as_root)),
        false,
    )
}
#[tauri::command]
pub fn vfs_trash_empty(as_root: Option<bool>, service: Service<'_>) -> GameResult<()> {
    service
        .lock()
        .mutate(|_, w, _| w.vfs.empty_trash(file_actor(as_root)), false)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionView {
    id: String,
    title: String,
    description: String,
    contact: String,
    status: String,
    objective: String,
    hint: String,
    choices: Vec<ChoiceView>,
}
#[derive(Serialize)]
pub struct ChoiceView {
    id: String,
    label: String,
}
#[tauri::command]
pub fn mission_get_state(service: Service<'_>) -> GameResult<Vec<MissionView>> {
    let game = service.lock();
    let world = game.world()?;
    Ok(game
        .engine
        .definitions
        .iter()
        .filter_map(|m| {
            let progress = world.missions.get(&m.id);
            if progress.is_none() && !game.engine.unlocked(world, m) {
                return None;
            }
            let stage = progress
                .filter(|p| p.status == "active")
                .and_then(|p| m.stages.get(p.stage));
            Some(MissionView {
                id: m.id.clone(),
                title: m.title.clone(),
                description: m.description.clone(),
                contact: m.contact.clone(),
                status: progress
                    .map(|p| p.status.clone())
                    .unwrap_or_else(|| "available".into()),
                objective: stage.map(|s| s.objective.clone()).unwrap_or_default(),
                hint: stage.map(|s| s.hint.clone()).unwrap_or_default(),
                choices: stage
                    .map(|s| {
                        s.choices
                            .iter()
                            .map(|c| ChoiceView {
                                id: c.id.clone(),
                                label: c.label.clone(),
                            })
                            .collect()
                    })
                    .unwrap_or_default(),
            })
        })
        .collect())
}
#[tauri::command]
pub fn mission_list_available(service: Service<'_>) -> GameResult<Vec<MissionView>> {
    Ok(mission_get_state(service)?
        .into_iter()
        .filter(|m| m.status == "available")
        .collect())
}
#[tauri::command]
pub fn mission_start(mission_id: String, service: Service<'_>) -> GameResult<()> {
    service.lock().mutate(
        |engine, w, events| {
            events.push(engine.start(w, &mission_id)?);
            Ok(())
        },
        false,
    )
}
#[tauri::command]
pub fn mission_abort_attempt(mission_id: String, service: Service<'_>) -> GameResult<()> {
    service
        .lock()
        .mutate(|engine, w, _| engine.abort(w, &mission_id), false)
}
#[tauri::command]
pub fn mission_choose(
    mission_id: String,
    choice_id: String,
    service: Service<'_>,
) -> GameResult<()> {
    service.lock().mutate(
        |engine, w, events| {
            if let Some(event) = engine.choose(w, &mission_id, &choice_id)? {
                events.push(event);
            }
            Ok(())
        },
        false,
    )
}
#[tauri::command]
pub fn messages_read(contact: String, service: Service<'_>) -> GameResult<()> {
    service.lock().mutate(
        |_, w, _| {
            for m in &mut w.messages {
                if m.contact == contact {
                    m.read = true;
                }
            }
            Ok(())
        },
        false,
    )
}
#[tauri::command]
pub fn message_reply(contact: String, text: String, service: Service<'_>) -> GameResult<()> {
    service.lock().mutate(
        |_, w, _| {
            if !w.contacts.contains(&contact) || text.is_empty() || text.len() > 1000 {
                return Err(domain("invalid message"));
            }
            w.notify(&contact, &format!("Você: {text}"));
            Ok(())
        },
        false,
    )
}
#[tauri::command]
pub fn forum_action(
    action: crate::forum::ForumAction,
    service: Service<'_>,
) -> GameResult<Option<String>> {
    service
        .lock()
        .mutate(|_, world, _| crate::forum::apply(world, action), false)
}
#[tauri::command]
pub fn setting_update(key: String, value: String, service: Service<'_>) -> GameResult<()> {
    service.lock().mutate(
        |_, w, _| {
            let valid = match key.as_str() {
                "fontSize" => value.parse::<u8>().is_ok_and(|n| (12..=24).contains(&n)),
                "wallpaper" => ["waves", "maze"].contains(&value.as_str()),
                "volume" => value.parse::<u8>().is_ok_and(|n| n <= 100),
                "networkEnabled" => ["true", "false"].contains(&value.as_str()),
                "fileSort" | "desktopSort" => {
                    ["name", "modified", "kind"].contains(&value.as_str())
                }
                "fileSortDirection" => ["asc", "desc"].contains(&value.as_str()),
                "fileFoldersFirst" => ["true", "false"].contains(&value.as_str()),
                "autostartApps" => serde_json::from_str::<Vec<String>>(&value).is_ok_and(|apps| {
                    apps.len() <= 16
                        && apps.iter().all(|app| {
                            ["terminal", "files", "editor", "browser", "messages"]
                                .contains(&app.as_str())
                        })
                }),
                "language" => value == "pt-BR",
                "location" => value == "Brasil",
                "keyboard" => ["br-abnt2", "br-abnt", "us-intl"].contains(&value.as_str()),
                "network" => ["eth0", "wlan0"].contains(&value.as_str()),
                "networkSsid" => w.network.wifi.iter().any(|ap| ap.ssid == value),
                "storage" => {
                    ["plain", "lvm", "encrypted", "raid1", "iscsi"].contains(&value.as_str())
                }
                "partitions" => crate::installer::validate(w, &value).is_ok(),
                "desktopEnvironment" => {
                    value.is_empty()
                        || value
                            .split(',')
                            .all(|s| ["xfce", "gnome", "kde"].contains(&s))
                }
                "softwareTools" => {
                    value.is_empty() || value.split(',').all(|s| ["top10", "default"].contains(&s))
                }
                "disk" => ["nvme0n1", "sda"].contains(&value.as_str()),
                "partitionScheme" => {
                    ["all", "home", "var-tmp", "server", "small"].contains(&value.as_str())
                }
                "writeChanges" => ["true", "false"].contains(&value.as_str()),
                "softwareProfile" => ["standard", "minimal"].contains(&value.as_str()),
                "partition" => [
                    "guided-largest",
                    "guided-disk",
                    "guided-lvm",
                    "guided-encrypted",
                    "manual",
                ]
                .contains(&value.as_str()),
                "fullName" => !value.trim().is_empty() && value.len() <= 80,
                "timezone" => ["America/Sao_Paulo", "America/Manaus", "America/Belem"]
                    .contains(&value.as_str()),
                "domain" => {
                    !value.is_empty()
                        && value.len() <= 48
                        && value
                            .chars()
                            .all(|c| c.is_ascii_alphanumeric() || ".-".contains(c))
                }
                "displayMode" => ["fullscreen", "windowed"].contains(&value.as_str()),
                "displayResolution" => {
                    ["1024x640", "1280x720", "1440x900", "1920x1080"].contains(&value.as_str())
                }
                "loginUsername" => {
                    !value.is_empty()
                        && value.len() <= 24
                        && value
                            .chars()
                            .all(|c| c.is_ascii_alphanumeric() || "_-".contains(c))
                }
                "loginPassword" => (4..=128).contains(&value.len()),
                _ => false,
            };
            if !valid {
                return Err(domain("invalid setting"));
            }
            if key == "networkEnabled" {
                w.network.connected = value == "true";
            }
            if key == "partitions" {
                crate::installer::apply(w, &value)?;
            }
            if key == "desktopEnvironment" || key == "softwareTools" {
                w.vfs
                    .seed(&format!("/etc/kali-{key}"), "file", &value, "root");
            }
            w.settings.insert(key, value);
            Ok(())
        },
        false,
    )
}

#[tauri::command]
pub fn launcher_open(id: String, service: Service<'_>) -> GameResult<()> {
    service
        .lock()
        .mutate(|_, world, _| crate::software::open(world, &id), false)
}
#[tauri::command]
pub fn launcher_favorite(id: String, service: Service<'_>) -> GameResult<()> {
    service
        .lock()
        .mutate(|_, world, _| crate::software::favorite(world, &id), false)
}
#[tauri::command]
pub fn tool_run(
    id: String,
    target: String,
    save_path: Option<String>,
    service: Service<'_>,
) -> GameResult<crate::software::ToolReport> {
    service.lock().mutate(
        |_, world, _| crate::software::run(world, &id, &target, save_path.as_deref()),
        false,
    )
}

#[tauri::command]
pub fn browser_navigate(
    address: String,
    service: Service<'_>,
) -> GameResult<crate::browser::BrowserPage> {
    service
        .lock()
        .mutate(|_, w, _| crate::browser::navigate(w, &address), false)
}

#[tauri::command]
pub fn browser_preferences_save(
    preferences: crate::browser::Preferences,
    service: Service<'_>,
) -> GameResult<()> {
    service.lock().mutate(
        |_, world, _| crate::browser::save_preferences(world, preferences),
        false,
    )
}
#[tauri::command]
pub fn browser_action(action: String, value: String, service: Service<'_>) -> GameResult<()> {
    service
        .lock()
        .mutate(|_, w, _| crate::browser::action(w, &action, &value), false)
}

#[tauri::command]
pub fn domains_search(
    query: String,
    business_type: String,
    service: Service<'_>,
) -> GameResult<crate::domains::DomainSearchResult> {
    crate::domains::search(service.lock().world()?, &query, &business_type)
}

#[tauri::command]
pub fn domains_whois(
    address: String,
    service: Service<'_>,
) -> GameResult<crate::domains::DomainWhois> {
    crate::domains::whois(service.lock().world()?, &address)
}

#[tauri::command]
pub fn domains_onion_inspect(
    address: String,
    service: Service<'_>,
) -> GameResult<crate::domains::OnionServiceInfo> {
    crate::domains::onion_service(service.lock().world()?, &address)
}

#[tauri::command]
pub fn domain_register(
    address: String,
    organization: String,
    business_type: String,
    service: Service<'_>,
) -> GameResult<crate::domains::DomainRecord> {
    service.lock().mutate(
        |_, world, _| crate::domains::register(world, &address, &organization, &business_type),
        false,
    )
}

#[tauri::command]
pub fn domain_renew(
    address: String,
    service: Service<'_>,
) -> GameResult<crate::domains::DomainRecord> {
    service
        .lock()
        .mutate(|_, world, _| crate::domains::renew(world, &address), false)
}

#[tauri::command]
pub fn domain_set_primary(address: String, service: Service<'_>) -> GameResult<()> {
    service.lock().mutate(
        |_, world, _| crate::domains::set_primary(world, &address),
        false,
    )
}

#[tauri::command]
pub fn domain_create_subdomain(
    address: String,
    label: String,
    service: Service<'_>,
) -> GameResult<crate::domains::SubdomainRecord> {
    service.lock().mutate(
        |_, world, _| crate::domains::create_subdomain(world, &address, &label),
        false,
    )
}

#[tauri::command]
pub fn domain_set_redirect(
    address: String,
    target: Option<String>,
    service: Service<'_>,
) -> GameResult<crate::domains::DomainRecord> {
    service.lock().mutate(
        |_, world, _| crate::domains::set_redirect(world, &address, target.as_deref()),
        false,
    )
}

#[tauri::command]
pub fn domain_list_for_sale(
    address: String,
    price: i64,
    service: Service<'_>,
) -> GameResult<crate::domains::DomainMarketOffer> {
    service.lock().mutate(
        |_, world, _| crate::domains::list_for_sale(world, &address, price),
        false,
    )
}

#[tauri::command]
pub fn domain_cancel_sale(address: String, service: Service<'_>) -> GameResult<()> {
    service.lock().mutate(
        |_, world, _| crate::domains::cancel_sale(world, &address),
        false,
    )
}

#[tauri::command]
pub fn domain_offer_respond(
    offer_id: String,
    action: String,
    counter_price: Option<i64>,
    service: Service<'_>,
) -> GameResult<()> {
    service.lock().mutate(
        |_, world, _| crate::domains::respond_offer(world, &offer_id, &action, counter_price),
        false,
    )
}

#[cfg(test)]
mod nano_tests {
    use super::*;
    use crate::{nano, terminal_sessions};

    #[test]
    fn isolated_editors_detect_conflicts_and_preserve_existing_final_newlines() {
        let mut game = GameService::new(rusqlite::Connection::open_in_memory().unwrap()).unwrap();
        game.new_game(1, "neo", "pc", false).unwrap();
        game.mutate(
            |_, w, _| {
                w.vfs.write("/home/kali/shared.txt", "old\n\n", "kali")?;
                for id in ["A", "B"] {
                    terminal_sessions::open(w, id)?;
                    terminal_sessions::with_session(w, Some(id), |w| {
                        nano::command(w, &["-L".into(), "shared.txt".into()], "kali")
                    })?;
                }
                Ok(())
            },
            false,
        )
        .unwrap();
        game.mutate(
            |_, w, _| {
                terminal_sessions::with_session(w, Some("A"), |w| {
                    let session = w.terminal.nano.clone().unwrap();
                    write_nano_buffer(
                        w,
                        &session,
                        "/home/kali/shared.txt",
                        "new\n\n",
                        Some("old\n\n"),
                    )
                })
            },
            false,
        )
        .unwrap();
        assert!(game
            .mutate(
                |_, w, _| terminal_sessions::with_session(w, Some("B"), |w| {
                    let session = w.terminal.nano.clone().unwrap();
                    write_nano_buffer(
                        w,
                        &session,
                        "/home/kali/shared.txt",
                        "stale",
                        Some("old\n\n"),
                    )
                }),
                false
            )
            .is_err());
        assert_eq!(
            game.world()
                .unwrap()
                .vfs
                .read("/home/kali/shared.txt", "kali")
                .unwrap(),
            "new\n\n"
        );
        assert_eq!(
            game.world().unwrap().terminal_sessions["B"]
                .nano
                .as_ref()
                .unwrap()
                .original_content
                .as_deref(),
            Some("old\n\n")
        );
    }
    #[test]
    fn restrictions_apply_to_reads_writes_save_as_and_numbered_backups() {
        let mut w = WorldState::new("neo", "pc").unwrap();
        w.vfs
            .write("/home/kali/Documents/note.txt", "old\n", "kali")
            .unwrap();
        nano::command(&mut w, &["-R".into(), "Documents/note.txt".into()], "kali").unwrap();
        let session = w.terminal.nano.clone().unwrap();
        assert!(read_nano_file(&w, "/etc/os-release").is_err());
        assert!(write_nano_buffer(&mut w, &session, "/home/kali/another.txt", "x", None).is_err());
        nano::command(
            &mut w,
            &[
                "-o".into(),
                "Documents".into(),
                "-B".into(),
                "-C".into(),
                ".".into(),
                "note.txt".into(),
            ],
            "kali",
        )
        .unwrap();
        assert!(read_nano_file(&w, "../outside.txt").is_err());
        let session = w.terminal.nano.clone().unwrap();
        assert_eq!(session.path, "/home/kali/Documents/note.txt");
        assert!(write_nano_buffer(&mut w, &session, "/home/kali/out.txt", "x", None).is_err());
        write_nano_buffer(&mut w, &session, &session.path, "one", Some("old\n")).unwrap();
        let session = w.terminal.nano.clone().unwrap();
        write_nano_buffer(&mut w, &session, &session.path, "two", Some("one\n")).unwrap();
        assert_eq!(
            w.vfs
                .read("/home/kali/Documents/note.txt~1~", "kali")
                .unwrap(),
            "old\n"
        );
        assert_eq!(
            w.vfs
                .read("/home/kali/Documents/note.txt~2~", "kali")
                .unwrap(),
            "one\n"
        );
        assert_eq!(w.vfs.read(&session.path, "kali").unwrap(), "two\n");
    }
}
