fn main() {
    tauri::Builder::default()
        .manage(bdl_tauri::state::AppState::new().expect("failed to initialize BDL app state"))
        .invoke_handler(tauri::generate_handler![
            bdl_tauri::commands::parse_create_source,
            bdl_tauri::commands::parse_load_more,
            bdl_tauri::commands::parse_load_all,
            bdl_tauri::commands::parse_close_source,
            bdl_tauri::commands::parse_refresh_source,
            bdl_tauri::commands::selection_create_tasks,
            bdl_tauri::commands::queue_list,
            bdl_tauri::commands::queue_logs,
            bdl_tauri::commands::queue_pause,
            bdl_tauri::commands::queue_resume,
            bdl_tauri::commands::queue_cancel,
            bdl_tauri::commands::queue_retry,
            bdl_tauri::commands::queue_remove,
            bdl_tauri::commands::queue_open_file,
            bdl_tauri::commands::queue_open_dir,
            bdl_tauri::commands::settings_get,
            bdl_tauri::commands::settings_update,
            bdl_tauri::commands::account_get,
            bdl_tauri::commands::account_login_qr_start,
            bdl_tauri::commands::account_login_qr_poll,
            bdl_tauri::commands::account_import_cookie,
            bdl_tauri::commands::account_logout,
            bdl_tauri::commands::account_verify,
        ])
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("failed to run BDL desktop app");
}
