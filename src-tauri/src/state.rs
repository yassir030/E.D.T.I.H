use std::sync::Mutex;
use crate::tools::ToolRegistry;
use crate::storage::Storage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    Openai,
    Gemini,
    Claude,
}

impl ProviderKind {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_lowercase().as_str() {
            "openai" => Ok(Self::Openai),
            "gemini" => Ok(Self::Gemini),
            "claude" => Ok(Self::Claude),
            _ => Err("Onbekende AI provider.".to_string()),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Openai => "openai",
            Self::Gemini => "gemini",
            Self::Claude => "claude",
        }
    }

    pub fn default_model(self) -> &'static str {
        match self {
            Self::Openai => "gpt-4o-mini",
            Self::Gemini => "gemini-3.1-flash-lite",
            Self::Claude => "claude-sonnet-4-20250514",
        }
    }
}

pub struct AppState {
    pub provider: Mutex<ProviderKind>,
    pub model: Mutex<String>,
    api_key: Mutex<Option<String>>,
    pub tool_registry: Mutex<ToolRegistry>,
    pub storage: Mutex<Storage>,
}

impl AppState {
    pub fn new() -> Self {
        let provider = ProviderKind::Openai;
        Self {
            provider: Mutex::new(provider),
            model: Mutex::new(provider.default_model().to_string()),
            api_key: Mutex::new(None),
            tool_registry: Mutex::new(ToolRegistry::new()),
            storage: Mutex::new(Storage::new().expect("Failed to initialize storage")),
        }
    }

    pub fn set_api_key(&self, key: String) -> Result<(), String> {
        let trimmed = key.trim().to_string();
        if trimmed.is_empty() {
            return Err("Voer een API key in.".to_string());
        }
        let mut guard = self
            .api_key
            .lock()
            .map_err(|_| "Interne state is vergrendeld.".to_string())?;

        // Save to persistent storage
        if let Ok(storage) = self.storage.lock() {
            let _ = storage.save_setting("api_key", &trimmed);
        }

        *guard = Some(trimmed);
        Ok(())
    }

    pub fn clear_api_key(&self) -> Result<(), String> {
        let mut guard = self
            .api_key
            .lock()
            .map_err(|_| "Interne state is vergrendeld.".to_string())?;

        // Clear from persistent storage
        if let Ok(storage) = self.storage.lock() {
            let _ = storage.delete_setting("api_key");
        }

        *guard = None;
        Ok(())
    }

    pub fn api_key_snapshot(&self) -> Result<Option<String>, String> {
        let guard = self
            .api_key
            .lock()
            .map_err(|_| "Interne state is vergrendeld.".to_string())?;
        Ok(guard.clone())
    }

    pub fn masked_api_key(&self) -> Result<Option<String>, String> {
        Ok(self.api_key_snapshot()?.as_deref().map(mask_secret))
    }

    pub fn has_api_key(&self) -> Result<bool, String> {
        Ok(self.api_key_snapshot()?.is_some())
    }

    pub fn load_persistent_settings(&self) -> Result<(), String> {
        if let Ok(storage) = self.storage.lock() {
            // Load API key from storage
            if let Ok(Some(saved_key)) = storage.get_setting("api_key") {
                if !saved_key.is_empty() {
                    let mut guard = self
                        .api_key
                        .lock()
                        .map_err(|_| "Interne state is vergrendeld.".to_string())?;
                    *guard = Some(saved_key);
                }
            }

            // Load provider and model
            if let Ok(Some(saved_provider)) = storage.get_setting("provider") {
                if let Ok(provider) = ProviderKind::parse(&saved_provider) {
                    let mut provider_guard = self
                        .provider
                        .lock()
                        .map_err(|_| "Interne state is vergrendeld.".to_string())?;
                    *provider_guard = provider;
                }
            }

            if let Ok(Some(saved_model)) = storage.get_setting("model") {
                let mut model_guard = self
                    .model
                    .lock()
                    .map_err(|_| "Interne state is vergrendeld.".to_string())?;
                *model_guard = saved_model;
            }
        }
        Ok(())
    }
}

pub fn mask_secret(value: &str) -> String {
    let chars: Vec<char> = value.chars().collect();
    let len = chars.len();
    if len == 0 {
        return String::new();
    }
    if len <= 8 {
        return "•".repeat(len);
    }
    let suffix: String = chars.iter().skip(len.saturating_sub(4)).collect();
    format!("••••••••{suffix}")
}

pub fn redact_secrets(message: &str, secret: Option<&str>) -> String {
    let mut redacted = message.to_string();
    if let Some(key) = secret {
        if !key.is_empty() {
            redacted = redacted.replace(key, "[redacted]");
        }
    }
    redacted
}
