mod ai;
mod commands;
mod state;
mod tools;
mod storage;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState::new();

    // Load persistent settings on startup
    let _ = app_state.load_persistent_settings();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::get_public_ai_settings,
            commands::save_provider_settings,
            commands::save_api_key,
            commands::clear_api_key,
            commands::send_ai_message,
            commands::test_ai_connection,
            commands::get_system_status,
            commands::list_desktop_tools,
            commands::get_filesystem_status,
            commands::invoke_desktop_tool,
            commands::save_conversation,
            commands::get_conversations,
            commands::delete_conversation,
            commands::save_memory,
            commands::get_memory,
            commands::delete_memory,
            commands::clear_all_memory,
            commands::get_action_log,
            commands::clear_action_log,
            commands::load_persistent_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
