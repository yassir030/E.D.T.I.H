# E.D.I.T.H. AI Provider Architecture

## Provider Adapters Location
Provider-specific implementations are located in `src-tauri/src/ai/`:
- `gemini.rs` - Google Gemini API implementation
- `openai.rs` - OpenAI API implementation  
- `claude.rs` - Anthropic Claude API implementation
- `mod.rs` - Shared provider interface and error handling

## Gemini Authentication

### How It Works
1. User enters API key in Settings UI (React frontend)
2. Key is sent via Tauri command to Rust backend
3. Key is stored in runtime memory only (`AppState.api_key: Mutex<Option<String>>`)
4. Key is never written to disk, localStorage, or configuration files
5. Each API request includes the key in the `x-goog-api-key` header

### Security Measures
- Keys are masked before being returned to the UI (shows only last 4 characters)
- Debug logging never includes API keys or sensitive headers
- Error messages are redacted to remove any accidentally leaked keys
- Keys are stored only in Rust runtime memory, cleared on app restart

## Adding a New Provider

### 1. Create Provider Module
Add a new file in `src-tauri/src/ai/your_provider.rs`:

```rust
use super::{public_error_from_status, ChatCompletionResult, ChatMessageInput};
use serde_json::{json, Value};

pub async fn send(
    api_key: &str,
    model: &str,
    messages: &[ChatMessageInput],
) -> Result<ChatCompletionResult, String> {
    // 1. Transform messages to provider format
    // 2. Build HTTP request to provider API
    // 3. Include authentication header
    // 4. Handle response and errors
    // 5. Return ChatCompletionResult with content
}

pub async fn test_connection(api_key: &str) -> Result<(), String> {
    // 1. Make a simple API call to verify connectivity
    // 2. Use provider's models endpoint or a minimal request
    // 3. Return Ok(()) if successful, Err with user-friendly message if not
}
```

### 2. Register Provider
In `src-tauri/src/ai/mod.rs`:
- Add `mod your_provider;` at the top
- Add variant to `ProviderKind` enum in `state.rs`
- Add `default_model()` implementation in `state.rs`
- Add match arm in `send_message()` and `test_connection()`

### 3. Add Frontend Support
In `src/views/SettingsView.tsx`:
- Add provider to the `providers` array
- Add default model to `defaultModels` object

## Connection Testing

### How It Works
1. User clicks "Test Connection" in Settings
2. Frontend calls `testAssistantConnection()` → `testAiConnection()` → Tauri command
3. Rust backend calls provider's `test_connection()` function
4. Provider makes a real API call (e.g., to `/models` endpoint)
5. Success/failure is returned with specific error messages
6. UI shows ✓ or ✗ with appropriate status message

### Test Endpoints
- **Gemini**: `GET https://generativelanguage.googleapis.com/v1beta/models`
- **OpenAI**: `GET https://api.openai.com/v1/models`
- **Claude**: `GET https://api.anthropic.com/v1/models`

## Error Handling

### Status Code Mapping
Error responses are mapped to user-friendly messages in `public_error_from_status()`:
- **400**: Invalid request (model/parameter validation)
- **401**: Authentication failed (invalid API key)
- **403**: Permission denied (key lacks access)
- **404**: Model/resource not found
- **429**: Rate limit exceeded
- **5xx**: Provider server errors

### Error Redaction
All error messages pass through `redact_secrets()` which removes:
- OpenAI keys: `sk-[A-Za-z0-9_-]+`
- Gemini keys: `AIza[A-Za-z0-9_-]+`
- Claude keys: `sk-ant-[A-Za-z0-9_-]+`

## Development Logging

In debug builds only (`#[cfg(debug_assertions)]`), the following safe information is logged:
- Provider name
- API endpoint hostname/path
- Model name
- HTTP status codes
- Provider error codes and messages (truncated)

**Never logged:**
- API keys
- Authorization headers
- Full request headers containing secrets
- Stored credentials

## Model Selection

### Current Defaults
- **Gemini**: `gemini-3.1-flash-lite` (replaces deprecated `gemini-2.0-flash`)
- **OpenAI**: `gpt-4o-mini`
- **Claude**: `claude-sonnet-4-20250514`

### Model Discovery
For extensibility, consider implementing model discovery by:
1. Calling provider's `/models` endpoint
2. Parsing available models
3. Filtering for models that support `generateContent`
4. Allowing user selection in Settings UI

## Request Flow

### Message Sending
1. User types message in Assistant view and clicks Send
2. Frontend calls `sendAssistantMessage(messages)` → Tauri command
3. Rust backend determines current provider from `AppState`
4. Calls provider's `send()` function with API key, model, and messages
5. Provider transforms messages to API format and makes HTTP request
6. Response is parsed and content extracted
7. Result returned to frontend and displayed in chat

### Configuration
- Provider selection and model are persisted in UI state
- API key is in Rust runtime memory only
- No credentials stored in frontend bundle or configuration files
