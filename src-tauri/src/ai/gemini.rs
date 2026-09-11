use super::{public_error_from_status, ChatCompletionResult, ChatMessageInput};
use crate::tools::ToolRegistry;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicUsize, Ordering};

static REQUEST_COUNTER: AtomicUsize = AtomicUsize::new(1);

pub async fn send(
    api_key: &str,
    model: &str,
    messages: &[ChatMessageInput],
    registry: &ToolRegistry,
) -> Result<ChatCompletionResult, String> {
    let req_num = REQUEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let req_id = format!("EDITH-REQUEST-{:03}", req_num);

    eprintln!("[EDITH][AI][REQUEST] id={} type=normal started", req_id);

    let mut system_chunks: Vec<String> = Vec::new();
    let mut contents: Vec<Value> = Vec::new();

    // Tools definition for Gemini
    let tools = json!([{
        "function_declarations": registry.list_tools().into_iter().map(|t| json!({
            "name": t.id,
            "description": t.description,
            "parameters": {
                "type": "OBJECT",
                "properties": {},
            }
        })).collect::<Vec<_>>()
    }]);

    for message in messages {
        match message.role.trim().to_ascii_lowercase().as_str() {
            "system" => system_chunks.push(message.content.clone()),
            "assistant" => {
                let parts = if message.content.is_empty() {
                    json!([])
                } else {
                    json!([{ "text": message.content }])
                };
                contents.push(json!({
                    "role": "model",
                    "parts": parts,
                }));
            },
            "tool" => contents.push(json!({
                "role": "function",
                "parts": [{ "functionResponse": { "name": "tool", "response": { "result": message.content } } }],
            })),
            _ => contents.push(json!({
                "role": "user",
                "parts": [{ "text": message.content }],
            })),
        }
    }

    if contents.is_empty() {
        eprintln!("[EDITH][AI][REQUEST] id={} failed status=400", req_id);
        return Err("Er is geen bericht om te versturen.".to_string());
    }

    let mut current_messages = contents;
    let max_tool_rounds = 5;

    for round in 1..=max_tool_rounds {
        eprintln!("[EDITH][AI][TOOL_LOOP] round={}", round);

        let mut payload = json!({ "contents": current_messages, "tools": tools });
        if !system_chunks.is_empty() {
            payload["systemInstruction"] = json!({
                "parts": [{ "text": system_chunks.join("\n") }],
            });
        }

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent"
        );

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(45))
            .build()
            .map_err(|_| "AI provider kon niet worden bereikt.".to_string())?;

        let mut attempt = 0;
        let max_attempts = 2; // 1 normal + 1 controlled retry for 429
        let mut response_result = None;

        while attempt < max_attempts {
            attempt += 1;
            let resp = client
                .post(&url)
                .header("x-goog-api-key", api_key)
                .json(&payload)
                .send()
                .await;

            match resp {
                Ok(res) => {
                    let status = res.status().as_u16();
                    if status == 400 {
                        let err_body = res.text().await.unwrap_or_default();
                        eprintln!("[EDITH][GEMINI][HTTP_ERROR]\nstatus=400\nbody={}", sanitize_error_body(&err_body));
                        eprintln!("[EDITH][AI][REQUEST] id={} failed status=400", req_id);
                        return Err(public_error_from_status(400, &err_body));
                    }
                    if status == 429 && attempt < max_attempts {
                        eprintln!("[EDITH][AI][REQUEST] id={} rate limited (429), retrying once...", req_id);
                        std::thread::sleep(std::time::Duration::from_secs(2));
                        continue;
                    }
                    response_result = Some(res);
                    break;
                }
                Err(err) => {
                    if attempt >= max_attempts {
                        eprintln!("[EDITH][AI][REQUEST] id={} failed network error", req_id);
                        return Err(map_network_error(&err));
                    }
                }
            }
        }

        let response = match response_result {
            Some(r) => r,
            None => {
                eprintln!("[EDITH][AI][REQUEST] id={} failed no response", req_id);
                return Err("AI provider kon niet worden bereikt.".to_string());
            }
        };

        let status = response.status().as_u16();
        let body = response
            .text()
            .await
            .map_err(|_| "Ongeldig antwoord van de provider.".to_string())?;

        if status >= 400 {
            eprintln!("[EDITH][AI][REQUEST] id={} failed status={}", req_id, status);
            return Err(public_error_from_status(status, &body));
        }

        let parsed: Value =
            serde_json::from_str(&body).map_err(|_| "Ongeldig antwoord van de provider.".to_string())?;

        let candidate = parsed.pointer("/candidates/0/content").ok_or_else(|| {
            eprintln!("[EDITH][AI][REQUEST] id={} failed no candidate", req_id);
            "Geen antwoord gevonden.".to_string()
        })?;
        
        current_messages.push(candidate.clone());

        // Check for function call
        if let Some(parts) = candidate.get("parts") {
            if let Some(first_part) = parts.get(0) {
                if let Some(function_call) = first_part.get("functionCall") {
                    let name = function_call["name"].as_str().ok_or_else(|| "Geen functienaam.".to_string())?;
                    let args = function_call.get("args").cloned().unwrap_or_else(|| json!({}));

                    eprintln!("[EDITH][TOOL] name={}", name);
                    let tool_result = registry.execute_tool(name, &args).map_err(|e| {
                        eprintln!("[EDITH][TOOL] error={}", e);
                        e.to_string()
                    })?;
                    eprintln!("[EDITH][TOOL] completed");

                    let response_json = json!({ "result": tool_result.message });

                    // Push function response as role: "function"
                    current_messages.push(json!({
                        "role": "function",
                        "parts": [{
                            "functionResponse": {
                                "name": name,
                                "response": response_json
                            }
                        }]
                    }));

                    // If tool returned an image (e.g., screenshot), attach it as a separate user content turn
                    if name == "get_screen_screenshot" {
                        eprintln!("[EDITH][VISION] screenshot captured");
                        if let Some(ref data) = tool_result.data {
                            if let Some(b64) = data.get("image").and_then(|v| v.as_str()) {
                                eprintln!("[EDITH][VISION] image attached to Gemini request");
                                current_messages.push(json!({
                                    "role": "user",
                                    "parts": [
                                        {
                                            "inlineData": {
                                                "mimeType": "image/png",
                                                "data": b64
                                            }
                                        },
                                        {
                                            "text": "Here is the screenshot captured by get_screen_screenshot."
                                        }
                                    ]
                                }));
                            }
                        }
                    }

                    continue;
                }
            }
        }

        // Return final text
        let content = candidate
            .pointer("/parts/0/text")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                eprintln!("[EDITH][AI][REQUEST] id={} failed empty response", req_id);
                "De provider gaf een leeg antwoord.".to_string()
            })?;

        eprintln!("[EDITH][AI][REQUEST] id={} completed", req_id);
        return Ok(ChatCompletionResult {
            content: content.to_string(),
        });
    }

    eprintln!("[EDITH][AI][REQUEST] id={} failed max tool rounds", req_id);
    Err("Too many tool execution steps (max 5 tool rounds reached).".to_string())
}

fn sanitize_error_body(body: &str) -> String {
    if body.len() > 1000 {
        format!("{}... [truncated]", &body[..1000])
    } else {
        body.to_string()
    }
}

pub async fn test_connection(api_key: &str) -> Result<(), String> {
    #[cfg(debug_assertions)]
    {
        eprintln!("[EDITH][AI] Connection test started");
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
        eprintln!("[EDITH][AI] Connection test HTTP Status: {}", status);
        if status >= 400 {
            eprintln!("[EDITH][AI] Connection test error: {}", &body[..body.len().min(500)]);
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
