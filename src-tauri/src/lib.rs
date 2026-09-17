mod app_settings;
mod archive;
mod binary;
#[cfg(test)]
mod binary_tests;
mod browser;
#[cfg(test)]
mod campaign_tests;
#[cfg(test)]
mod cli_compat_tests;
pub mod cli_contract;
#[cfg(test)]
mod cli_tooling_bridge;
mod command_registry;
mod commands;
mod completion;
mod coreutils;
mod db;
mod desktop;
mod domains;
mod error;
mod forum;
mod host_power;
mod installer;
mod investigation;
mod mission;
#[cfg(test)]
mod mission_persistence_tests;
mod mission_runtime;
pub mod nano;
mod network;
#[cfg(test)]
mod opening_fixture;
mod packages;
mod save;
mod service;
mod shell;
mod shell_pipeline;
#[cfg(test)]
mod shell_tests;
mod software;
mod system_info;
mod task_manager;
mod terminal;
mod terminal_io;
#[cfg(test)]
mod terminal_io_tests;
mod terminal_query;
#[cfg(test)]
mod terminal_query_tests;
mod terminal_remove;
mod terminal_sessions;
mod terminal_text;
mod terminal_transfer;
mod vfs;
mod world;

use parking_lot::Mutex;
use service::GameService;
use tauri::{Emitter, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let result = tauri::Builder::default()
        .setup(|app| {
            #[cfg(debug_assertions)]
            if std::env::var("CYBER_WAR_ARCHIVE_QA").as_deref() == Ok("1") {
                app.manage(Mutex::new(archive::qa::game()?));
                if let Some(window) = app.get_webview_window("main") {
                    window.set_fullscreen(false)?;
                    window.set_size(tauri::LogicalSize::new(1280, 800))?;
                    window.navigate("http://localhost:1420/#archive/qa".parse()?)?;
                }
                return Ok(());
            }
            let data = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data)?;
            let game = GameService::new(rusqlite::Connection::open(data.join("game-hacker.db"))?)?;
            app.manage(Mutex::new(game));
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if let Some(state) = window.try_state::<Mutex<GameService>>() {
                    let mut game = state.lock();
                    if game.active.is_some() {
                        if let Err(error) = game.end_session() {
                            api.prevent_close();
                            let _ = window.emit("save-error", error.to_string());
                        }
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            host_power::host_battery_status,
            packages::ipc::package_inspect,
            packages::ipc::package_plan,
            packages::ipc::package_confirm,
            packages::ipc::package_launch,
            archive::ipc::archive_operation,
            archive::ipc::archive_terminal_input,
            archive::ipc::archive_job_start,
            archive::ipc::archive_job_cancel,
            archive::ipc::archive_jobs_tick,
            commands::attachment_download,
            commands::system_health,
            commands::task_manager_snapshot,
            commands::task_manager_terminate,
            commands::wireless_scan,
            commands::wireless_inspect,
            commands::traffic_read,
            commands::traffic_follow,
            commands::settings_global_get,
            commands::settings_global_save,
            commands::quit_game,
            commands::world_get,
            commands::new_game,
            commands::list_save_slots,
            commands::save_slot,
            commands::autosave,
            commands::session_start,
            commands::end_session,
            commands::terminal_open,
            commands::terminal_close,
            commands::terminal_input,
            commands::terminal_output_ack,
            commands::terminal_cancel_all,
            commands::system_setup_complete,
            commands::load_slot,
            commands::create_mission_checkpoint,
            commands::list_checkpoints,
            commands::restore_checkpoint,
            commands::execute_terminal,
            commands::terminal_complete,
            commands::nano_write,
            commands::nano_read,
            commands::nano_close,
            commands::vfs_list,
            commands::desktop_create,
            commands::vfs_read,
            commands::vfs_read_bytes,
            commands::vfs_import_bytes,
            commands::vfs_write,
            commands::vfs_copy,
            commands::vfs_stat,
            commands::vfs_create_file,
            commands::vfs_create_directory,
            commands::vfs_move,
            commands::vfs_remove,
            commands::vfs_trash_restore,
            commands::vfs_trash_empty,
            commands::mission_get_state,
            commands::mission_list_available,
            commands::mission_start,
            commands::mission_abort_attempt,
            commands::mission_choose,
            commands::messages_read,
            commands::message_reply,
            commands::forum_action,
            commands::setting_update,
            commands::launcher_open,
            commands::launcher_favorite,
            commands::tool_run,
            commands::browser_navigate,
            commands::browser_preferences_save,
            commands::browser_action,
            commands::domains_search,
            commands::domains_whois,
            commands::domains_onion_inspect,
            commands::domain_register,
            commands::domain_renew,
            commands::domain_set_primary,
            commands::domain_create_subdomain,
            commands::domain_set_redirect,
            commands::domain_list_for_sale,
            commands::domain_cancel_sale,
            commands::domain_offer_respond
        ])
        .run(tauri::generate_context!());
    if let Err(error) = result {
        eprintln!("Game Hacker could not start: {error}");
    }
}
