use super::{public_error_from_status, ChatCompletionResult, ChatMessageInput};
use serde_json::{json, Value};

pub async fn send(
    api_key: &str,
    model: &str,
    messages: &[ChatMessageInput],
) -> Result<ChatCompletionResult, String> {
    let mut system_chunks: Vec<String> = Vec::new();
    let mut contents: Vec<Value> = Vec::new();

    for message in messages {
        match message.role.trim().to_ascii_lowercase().as_str() {
            "system" => system_chunks.push(message.content.clone()),
            "assistant" => contents.push(json!({
                "role": "model",
                "parts": [{ "text": message.content }],
            })),
            _ => contents.push(json!({
                "role": "user",
                "parts": [{ "text": message.content }],
            })),
        }
    }

    if contents.is_empty() {
        return Err("Er is geen bericht om te versturen.".to_string());
    }

    let mut payload = json!({ "contents": contents });
    if !system_chunks.is_empty() {
        payload["systemInstruction"] = json!({
            "parts": [{ "text": system_chunks.join("\n") }],
        });
    }

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent"
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(90))
        .build()
        .map_err(|_| "AI provider kon niet worden bereikt.".to_string())?;

    let response = client
        .post(url)
        .header("x-goog-api-key", api_key)
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
        .pointer("/candidates/0/content/parts/0/text")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "De provider gaf een leeg antwoord.".to_string())?;

    Ok(ChatCompletionResult {
        content: content.to_string(),
    })
}
