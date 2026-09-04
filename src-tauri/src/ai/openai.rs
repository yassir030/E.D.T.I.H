use super::{public_error_from_status, ChatCompletionResult, ChatMessageInput};
use serde_json::{json, Value};

pub async fn send(
    api_key: &str,
    model: &str,
    messages: &[ChatMessageInput],
) -> Result<ChatCompletionResult, String> {
    let payload_messages: Vec<Value> = messages
        .iter()
        .map(|m| {
            json!({
                "role": normalize_role(&m.role),
                "content": m.content,
            })
        })
        .collect();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(90))
        .build()
        .map_err(|_| "AI provider kon niet worden bereikt.".to_string())?;

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&json!({
            "model": model,
            "messages": payload_messages,
        }))
        .send()
        .await
        .map_err(|_| "AI provider kon niet worden bereikt.".to_string())?;

    let status = response.status().as_u16();
    let body = response
        .text()
        .await
        .map_err(|_| "Ongeldig antwoord van de provider.".to_string())?;

    if status >= 400 {
        return Err(public_error_from_status(status, &body));
    }

    let parsed: Value =
        serde_json::from_str(&body).map_err(|_| "Ongeldig antwoord van de provider.".to_string())?;
    let content = parsed
        .pointer("/choices/0/message/content")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "De provider gaf een leeg antwoord.".to_string())?;

    Ok(ChatCompletionResult {
        content: content.to_string(),
    })
}

fn normalize_role(role: &str) -> &'static str {
    match role.trim().to_ascii_lowercase().as_str() {
        "assistant" => "assistant",
        "system" => "system",
        _ => "user",
    }
}
