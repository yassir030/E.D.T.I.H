use crate::ai::{self, ChatCompletionResult, ChatMessageInput};
use crate::state::{AppState, ProviderKind};
use crate::tools::{ToolRequest, ToolResult};
use crate::storage::{Conversation, MemoryEntry};
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

#[derive(Debug, Deserialize)]
pub struct SaveConversationPayload {
    pub conversation: Conversation,
}

#[derive(Debug, Deserialize)]
pub struct SaveMemoryPayload {
    pub memory: MemoryEntry,
}

#[derive(Debug, Serialize)]
pub struct SystemStatus {
    pub app_name: String,
    pub version: String,
    pub platform: String,
    pub filesystem_ready: bool,
    pub memory_backend: String,
    pub voice_ready: bool,
    pub desktop_control_ready: bool,
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
        *model_guard = model.clone();
    }

    // Save to persistent storage
    if let Ok(storage) = state.storage.lock() {
        let _ = storage.save_setting("provider", provider.as_str());
        let _ = storage.save_setting("model", &model);
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
pub async fn test_ai_connection(state: State<'_, AppState>) -> Result<(), String> {
    ai::test_connection(&state).await
}

#[tauri::command]
pub fn get_system_status() -> SystemStatus {
    SystemStatus {
        app_name: "E.D.I.T.H.".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        platform: std::env::consts::OS.to_string(),
        filesystem_ready: true,
        memory_backend: "sqlite".to_string(),
        voice_ready: false,
        desktop_control_ready: cfg!(not(target_os = "linux")),
    }
}

#[tauri::command]
pub fn list_desktop_tools(state: State<'_, AppState>) -> Vec<DesktopTool> {
    let registry = state.tool_registry.lock().unwrap();
    let tools = registry.list_tools();

    tools.into_iter().map(|tool| {
        let tool_id = tool.id.clone();
        DesktopTool {
            id: tool_id.clone(),
            name: tool.name,
            description: tool.description,
            input: "JSON arguments".to_string(),
            output: "Tool result".to_string(),
            requires_confirmation: registry.requires_confirmation(&tool_id),
            enabled: tool.enabled,
        }
    }).collect()
}

#[tauri::command]
pub fn get_filesystem_status() -> FilesystemStatus {
    FilesystemStatus {
        available: true,
        message: "Filesystem integration is active. Access is limited to user-approved directories.".to_string(),
    }
}

#[tauri::command]
pub fn invoke_desktop_tool(state: State<'_, AppState>, payload: ToolRequest) -> Result<ToolResult, String> {
    let registry = state.tool_registry.lock().unwrap();

    // Check if tool exists
    if registry.get_tool(&payload.tool_id).is_none() {
        return Err(format!("Onbekende tool: {}", payload.tool_id));
    }

    // Execute the tool
    match registry.execute_tool(&payload.tool_id, &payload.arguments) {
        Ok(result) => {
            // Log the action
            if let Ok(storage) = state.storage.lock() {
                let _ = storage.log_action(
                    &format!("Tool execution: {}", payload.tool_id),
                    &payload.tool_id,
                    if result.success { "Success" } else { "Failed" }
                );
            }
            Ok(result)
        }
        Err(e) => Ok(ToolResult {
            success: false,
            message: format!("Tool execution failed: {}", e),
            data: None,
        }),
    }
}

#[tauri::command]
pub fn save_conversation(state: State<'_, AppState>, payload: SaveConversationPayload) -> Result<(), String> {
    state.storage.lock()
        .map_err(|_| "Storage is vergrendeld.".to_string())?
        .save_conversation(&payload.conversation)
        .map_err(|e| format!("Failed to save conversation: {}", e))
}

#[tauri::command]
pub fn get_conversations(state: State<'_, AppState>) -> Result<Vec<Conversation>, String> {
    state.storage.lock()
        .map_err(|_| "Storage is vergrendeld.".to_string())?
        .list_conversations()
        .map_err(|e| format!("Failed to get conversations: {}", e))
}

#[tauri::command]
pub fn delete_conversation(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.storage.lock()
        .map_err(|_| "Storage is vergrendeld.".to_string())?
        .delete_conversation(&id)
        .map_err(|e| format!("Failed to delete conversation: {}", e))
}

#[tauri::command]
pub fn save_memory(state: State<'_, AppState>, payload: SaveMemoryPayload) -> Result<(), String> {
    state.storage.lock()
        .map_err(|_| "Storage is vergrendeld.".to_string())?
        .save_memory(&payload.memory)
        .map_err(|e| format!("Failed to save memory: {}", e))
}

#[tauri::command]
pub fn get_memory(state: State<'_, AppState>) -> Result<Vec<MemoryEntry>, String> {
    state.storage.lock()
        .map_err(|_| "Storage is vergrendeld.".to_string())?
        .list_memory()
        .map_err(|e| format!("Failed to get memory: {}", e))
}

#[tauri::command]
pub fn delete_memory(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.storage.lock()
        .map_err(|_| "Storage is vergrendeld.".to_string())?
        .delete_memory(&id)
        .map_err(|e| format!("Failed to delete memory: {}", e))
}

#[tauri::command]
pub fn clear_all_memory(state: State<'_, AppState>) -> Result<(), String> {
    state.storage.lock()
        .map_err(|_| "Storage is vergrendeld.".to_string())?
        .clear_all_memory()
        .map_err(|e| format!("Failed to clear memory: {}", e))
}

#[tauri::command]
pub fn get_action_log(state: State<'_, AppState>, limit: Option<i32>) -> Result<Vec<(i64, String, String, String)>, String> {
    state.storage.lock()
        .map_err(|_| "Storage is vergrendeld.".to_string())?
        .get_action_log(limit.unwrap_or(50))
        .map_err(|e| format!("Failed to get action log: {}", e))
}

#[tauri::command]
pub fn clear_action_log(state: State<'_, AppState>) -> Result<(), String> {
    state.storage.lock()
        .map_err(|_| "Storage is vergrendeld.".to_string())?
        .clear_action_log()
        .map_err(|e| format!("Failed to clear action log: {}", e))
}

#[tauri::command]
pub fn load_persistent_settings(state: State<'_, AppState>) -> Result<(), String> {
    state.load_persistent_settings()
}
