mod claude;
mod gemini;
mod openai;

use crate::state::{redact_secrets, AppState, ProviderKind};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ChatMessageInput {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct ChatCompletionResult {
    pub content: String,
}

pub async fn send_message(
    state: &AppState,
    messages: Vec<ChatMessageInput>,
) -> Result<ChatCompletionResult, String> {
    if messages.is_empty() {
        return Err("Er is geen bericht om te versturen.".to_string());
    }

    let provider = *state
        .provider
        .lock()
        .map_err(|_| "Interne state is vergrendeld.".to_string())?;
    let model = state
        .model
        .lock()
        .map_err(|_| "Interne state is vergrendeld.".to_string())?
        .clone();
    let api_key = state
        .api_key_snapshot()?
        .ok_or_else(|| "Configureer eerst een AI provider in Settings.".to_string())?;

    let result = match provider {
        ProviderKind::Openai => openai::send(&api_key, &model, &messages).await,
        ProviderKind::Gemini => gemini::send(&api_key, &model, &messages).await,
        ProviderKind::Claude => claude::send(&api_key, &model, &messages).await,
    };

    result.map_err(|err| {
        let safe = redact_secrets(&err, Some(&api_key));
        if safe.to_ascii_lowercase().contains("connect")
            || safe.to_ascii_lowercase().contains("dns")
            || safe.to_ascii_lowercase().contains("timed out")
        {
            "AI provider kon niet worden bereikt.".to_string()
        } else {
            safe
        }
    })
}

pub fn public_error_from_status(status: u16, body: &str) -> String {
    match status {
        401 | 403 => "API key is ongeldig of heeft geen toegang.".to_string(),
        429 => "Te veel verzoeken. Probeer later opnieuw.".to_string(),
        400 | 404 | 422 => {
            if let Some(msg) = extract_provider_error(body) {
                if msg.len() < 180 {
                    return format!("Het verzoek werd afgewezen: {msg}");
                }
            }
            "Het verzoek werd afgewezen door de provider.".to_string()
        }
        _ => "AI provider kon niet worden bereikt.".to_string(),
    }
}

fn extract_provider_error(body: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    value
        .pointer("/error/message")
        .or_else(|| value.pointer("/error/status"))
        .or_else(|| value.get("message"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}
