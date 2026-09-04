mod ai;
mod commands;
mod state;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::get_public_ai_settings,
            commands::save_provider_settings,
            commands::save_api_key,
            commands::clear_api_key,
            commands::send_ai_message,
            commands::get_system_status,
            commands::list_desktop_tools,
            commands::get_filesystem_status,
            commands::invoke_desktop_tool,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
