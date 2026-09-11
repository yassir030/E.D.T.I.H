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

    let registry = (*state
        .tool_registry
        .lock()
        .map_err(|_| "Interne state is vergrendeld.".to_string())?)
        .clone();

    let result = match provider {
        ProviderKind::Openai => openai::send(&api_key, &model, &messages).await,
        ProviderKind::Gemini => gemini::send(&api_key, &model, &messages, &registry).await,
        ProviderKind::Claude => claude::send(&api_key, &model, &messages).await,
    };

    result.map_err(|err| redact_secrets(&err, Some(&api_key)))
}

pub async fn test_connection(state: &AppState) -> Result<(), String> {
    let provider = *state
        .provider
        .lock()
        .map_err(|_| "Interne state is vergrendeld.".to_string())?;
    let api_key = state
        .api_key_snapshot()?
        .ok_or_else(|| "Configureer eerst een AI provider in Settings.".to_string())?;

    let result = match provider {
        ProviderKind::Openai => openai::test_connection(&api_key).await,
        ProviderKind::Gemini => gemini::test_connection(&api_key).await,
        ProviderKind::Claude => claude::test_connection(&api_key).await,
    };

    result.map_err(|err| redact_secrets(&err, Some(&api_key)))
}

pub fn public_error_from_status(status: u16, body: &str) -> String {
    match status {
        400 => {
            if let Some(msg) = extract_provider_error(body) {
                if msg.len() < 180 {
                    return format!("Ongeldig verzoek: {msg}");
                }
            }
            "Ongeldig verzoek: controleer model en parameters.".to_string()
        }
        401 => {
            if let Some(msg) = extract_provider_error(body) {
                if msg.len() < 180 {
                    return format!("Authenticatiefout: {msg}");
                }
            }
            "API key is ongeldig of ontbreekt.".to_string()
        }
        403 => {
            if let Some(msg) = extract_provider_error(body) {
                if msg.len() < 180 {
                    return format!("Toegang geweigerd: {msg}");
                }
            }
            "API key heeft geen toegang tot deze resource of model.".to_string()
        }
        404 => {
            if let Some(msg) = extract_provider_error(body) {
                if msg.len() < 180 {
                    return format!("Niet gevonden: {msg}");
                }
            }
            "Model of resource niet gevonden. Controleer de modelnaam.".to_string()
        }
        429 => {
            if let Some(msg) = extract_provider_error(body) {
                if msg.len() < 180 {
                    return format!("Rate limit: {msg}");
                }
            }
            "Te veel verzoeken. Probeer later opnieuw.".to_string()
        }
        500..=599 => {
            if let Some(msg) = extract_provider_error(body) {
                if msg.len() < 180 {
                    return format!("Providerfout ({status}): {msg}");
                }
            }
            format!("Providerfout ({status}): de AI provider ondervindt problemen.")
        }
        _ => {
            if let Some(msg) = extract_provider_error(body) {
                if msg.len() < 180 {
                    return format!("Fout ({status}): {msg}");
                }
            }
            format!("Onverwachte fout ({status}).")
        }
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
