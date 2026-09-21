#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let download_dir = app.path().download_dir()?.join("BDL");
            app.manage(bdl_tauri::state::AppState::new(data_dir, download_dir)?);
            bdl_tauri::commands::start_account_startup_verification(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            bdl_tauri::commands::parse_create_source,
            bdl_tauri::commands::parse_load_more,
            bdl_tauri::commands::parse_cancel,
            bdl_tauri::commands::parse_progress,
            bdl_tauri::commands::parse_load_all,
            bdl_tauri::commands::parse_close_source,
            bdl_tauri::commands::parse_refresh_source,
            bdl_tauri::commands::selection_create_tasks,
            bdl_tauri::commands::queue_list,
            bdl_tauri::commands::queue_startup_recovery,
            bdl_tauri::commands::queue_dismiss_startup_recovery,
            bdl_tauri::commands::queue_logs,
            bdl_tauri::commands::queue_pause,
            bdl_tauri::commands::queue_resume,
            bdl_tauri::commands::queue_schedule,
            bdl_tauri::commands::queue_unschedule,
            bdl_tauri::commands::queue_set_speed_limit,
            bdl_tauri::commands::queue_cancel,
            bdl_tauri::commands::queue_retry,
            bdl_tauri::commands::queue_refresh_urls_and_retry,
            bdl_tauri::commands::queue_bulk_pause,
            bdl_tauri::commands::queue_bulk_cancel,
            bdl_tauri::commands::queue_bulk_resume,
            bdl_tauri::commands::queue_bulk_retry,
            bdl_tauri::commands::queue_bulk_refresh_urls_and_retry,
            bdl_tauri::commands::queue_bulk_remove,
            bdl_tauri::commands::queue_clear_completed,
            bdl_tauri::commands::queue_remove,
            bdl_tauri::commands::queue_open_file,
            bdl_tauri::commands::queue_open_dir,
            bdl_tauri::commands::open_external_url,
            bdl_tauri::commands::settings_get,
            bdl_tauri::commands::settings_update,
            bdl_tauri::commands::environment_health,
            bdl_tauri::commands::environment_create_download_directory,
            bdl_tauri::commands::maintenance_cleanup_cache,
            bdl_tauri::commands::maintenance_cleanup_temp,
            bdl_tauri::commands::diagnostics_export,
            bdl_tauri::commands::account_get,
            bdl_tauri::commands::account_login_qr_start,
            bdl_tauri::commands::account_login_qr_poll,
            bdl_tauri::commands::account_import_cookie,
            bdl_tauri::commands::account_logout,
            bdl_tauri::commands::account_verify,
            bdl_tauri::commands::account_library_list,
        ])
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .run(tauri::generate_context!())
        .expect("failed to run BDL desktop app");
}
