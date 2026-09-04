use crate::ai::{self, ChatCompletionResult, ChatMessageInput};
use crate::state::{AppState, ProviderKind};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize)]
pub struct PublicAiSettings {
    pub provider: String,
    pub model: String,
    pub has_api_key: bool,
    pub masked_api_key: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SaveProviderPayload {
    pub provider: String,
    pub model: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SaveApiKeyPayload {
    pub api_key: String,
}

#[derive(Debug, Deserialize)]
pub struct SendAiMessagePayload {
    pub messages: Vec<ChatMessageInput>,
}

#[derive(Debug, Serialize)]
pub struct SystemStatus {
    pub app_name: String,
    pub version: String,
    pub platform: String,
    pub filesystem_ready: bool,
    pub memory_backend: String,
    pub voice_ready: bool,
}

#[derive(Debug, Serialize)]
pub struct DesktopTool {
    pub id: String,
    pub name: String,
    pub description: String,
    pub input: String,
    pub output: String,
    pub requires_confirmation: bool,
    pub enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct FilesystemStatus {
    pub available: bool,
    pub message: String,
}

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {name}! You've been greeted from Rust!")
}

#[tauri::command]
pub fn get_public_ai_settings(state: State<'_, AppState>) -> Result<PublicAiSettings, String> {
    let provider = *state
        .provider
        .lock()
        .map_err(|_| "Interne state is vergrendeld.".to_string())?;
    let model = state
        .model
        .lock()
        .map_err(|_| "Interne state is vergrendeld.".to_string())?
        .clone();

    Ok(PublicAiSettings {
        provider: provider.as_str().to_string(),
        model,
        has_api_key: state.has_api_key()?,
        masked_api_key: state.masked_api_key()?,
    })
}

#[tauri::command]
pub fn save_provider_settings(
    state: State<'_, AppState>,
    payload: SaveProviderPayload,
) -> Result<PublicAiSettings, String> {
    let provider = ProviderKind::parse(&payload.provider)?;
    let model = payload
        .model
        .map(|m| m.trim().to_string())
        .filter(|m| !m.is_empty())
        .unwrap_or_else(|| provider.default_model().to_string());

    {
        let mut provider_guard = state
            .provider
            .lock()
            .map_err(|_| "Interne state is vergrendeld.".to_string())?;
        *provider_guard = provider;
    }
    {
        let mut model_guard = state
            .model
            .lock()
            .map_err(|_| "Interne state is vergrendeld.".to_string())?;
        *model_guard = model;
    }

    get_public_ai_settings(state)
}

#[tauri::command]
pub fn save_api_key(
    state: State<'_, AppState>,
    payload: SaveApiKeyPayload,
) -> Result<PublicAiSettings, String> {
    state.set_api_key(payload.api_key)?;
    get_public_ai_settings(state)
}

#[tauri::command]
pub fn clear_api_key(state: State<'_, AppState>) -> Result<PublicAiSettings, String> {
    state.clear_api_key()?;
    get_public_ai_settings(state)
}

#[tauri::command]
pub async fn send_ai_message(
    state: State<'_, AppState>,
    payload: SendAiMessagePayload,
) -> Result<ChatCompletionResult, String> {
    ai::send_message(&state, payload.messages).await
}

#[tauri::command]
pub fn get_system_status() -> SystemStatus {
    SystemStatus {
        app_name: "E.D.I.T.H.".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        platform: std::env::consts::OS.to_string(),
        filesystem_ready: false,
        memory_backend: "runtime-memory".to_string(),
        voice_ready: false,
    }
}

#[tauri::command]
pub fn list_desktop_tools() -> Vec<DesktopTool> {
    vec![
        tool("open_file", "Open file", "path", true),
        tool("open_folder", "Open folder", "path", true),
        tool("open_application", "Open application", "app_id", true),
        tool("open_browser", "Open browser", "url", true),
        tool("system_information", "System information", "none", false),
        tool("screenshot", "Screenshot", "none", true),
        tool("keyboard_automation", "Keyboard automation", "sequence", true),
        tool("mouse_automation", "Mouse automation", "action", true),
    ]
}

#[tauri::command]
pub fn get_filesystem_status() -> FilesystemStatus {
    FilesystemStatus {
        available: false,
        message: "Filesystem-integratie is nog niet aangesloten. Er worden geen lokale bestanden getoond of verzonnen.".to_string(),
    }
}

#[tauri::command]
pub fn invoke_desktop_tool(tool_id: String) -> Result<String, String> {
    let known = list_desktop_tools();
    if !known.iter().any(|tool| tool.id == tool_id) {
        return Err("Onbekende tool.".to_string());
    }
    Err("Deze desktop-tool is geregistreerd maar nog niet ingeschakeld. Er wordt niets uitgevoerd.".to_string())
}

fn tool(id: &str, name: &str, input: &str, requires_confirmation: bool) -> DesktopTool {
    DesktopTool {
        id: id.to_string(),
        name: name.to_string(),
        description: format!("{name} via de permission/tool-laag."),
        input: input.to_string(),
        output: "success | error".to_string(),
        requires_confirmation,
        enabled: false,
    }
}
