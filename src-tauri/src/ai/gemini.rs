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

    #[cfg(debug_assertions)]
    {
        eprintln!("[Gemini Debug] Endpoint: {}", url);
        eprintln!("[Gemini Debug] Model: {}", model);
        eprintln!("[Gemini Debug] Messages count: {}", messages.len());
    }

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
        .map_err(|err| map_network_error(&err))?;

    let status = response.status().as_u16();
    let body = response
        .text()
        .await
        .map_err(|_| "Ongeldig antwoord van de provider.".to_string())?;

    #[cfg(debug_assertions)]
    {
        eprintln!("[Gemini Debug] HTTP Status: {}", status);
        if status >= 400 {
            eprintln!("[Gemini Debug] Error body: {}", &body[..body.len().min(500)]);
        }
    }

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

pub async fn test_connection(api_key: &str) -> Result<(), String> {
    #[cfg(debug_assertions)]
    {
        eprintln!("[Gemini Debug] Testing connection to models endpoint");
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|_| "AI provider kon niet worden bereikt.".to_string())?;

    let response = client
        .get("https://generativelanguage.googleapis.com/v1beta/models")
        .header("x-goog-api-key", api_key)
        .send()
        .await
        .map_err(|err| map_network_error(&err))?;

    let status = response.status().as_u16();
    let body = response
        .text()
        .await
        .map_err(|_| "Ongeldig antwoord van de provider.".to_string())?;

    #[cfg(debug_assertions)]
    {
        eprintln!("[Gemini Debug] Connection test HTTP Status: {}", status);
        if status >= 400 {
            eprintln!("[Gemini Debug] Connection test error: {}", &body[..body.len().min(500)]);
        }
    }

    if status >= 400 {
        return Err(public_error_from_status(status, &body));
    }

    Ok(())
}

fn map_network_error(err: &reqwest::Error) -> String {
    if err.is_timeout() {
        "De verbinding met de provider is verlopen.".to_string()
    } else {
        "AI provider kon niet worden bereikt.".to_string()
    }
}
