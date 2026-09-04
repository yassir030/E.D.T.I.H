use super::{public_error_from_status, ChatCompletionResult, ChatMessageInput};
use serde_json::{json, Value};

pub async fn send(
    api_key: &str,
    model: &str,
    messages: &[ChatMessageInput],
) -> Result<ChatCompletionResult, String> {
    let mut system_chunks: Vec<String> = Vec::new();
    let mut payload_messages: Vec<Value> = Vec::new();

    for message in messages {
        match message.role.trim().to_ascii_lowercase().as_str() {
            "system" => system_chunks.push(message.content.clone()),
            "assistant" => payload_messages.push(json!({
                "role": "assistant",
                "content": message.content,
            })),
            _ => payload_messages.push(json!({
                "role": "user",
                "content": message.content,
            })),
        }
    }

    if payload_messages.is_empty() {
        return Err("Er is geen bericht om te versturen.".to_string());
    }

    let mut payload = json!({
        "model": model,
        "max_tokens": 4096,
        "messages": payload_messages,
    });
    if !system_chunks.is_empty() {
        payload["system"] = json!(system_chunks.join("\n"));
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(90))
        .build()
        .map_err(|_| "AI provider kon niet worden bereikt.".to_string())?;

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&payload)
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
        .pointer("/content/0/text")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "De provider gaf een leeg antwoord.".to_string())?;

    Ok(ChatCompletionResult {
        content: content.to_string(),
    })
}
