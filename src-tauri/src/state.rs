use std::sync::Mutex;

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
            Self::Gemini => "gemini-2.0-flash",
            Self::Claude => "claude-sonnet-4-20250514",
        }
    }
}

pub struct AppState {
    pub provider: Mutex<ProviderKind>,
    pub model: Mutex<String>,
    api_key: Mutex<Option<String>>,
}

impl AppState {
    pub fn new() -> Self {
        let provider = ProviderKind::Openai;
        Self {
            provider: Mutex::new(provider),
            model: Mutex::new(provider.default_model().to_string()),
            api_key: Mutex::new(None),
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
        *guard = Some(trimmed);
        Ok(())
    }

    pub fn clear_api_key(&self) -> Result<(), String> {
        let mut guard = self
            .api_key
            .lock()
            .map_err(|_| "Interne state is vergrendeld.".to_string())?;
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
    let prefix: String = chars.iter().take(7).collect();
    let suffix: String = chars.iter().skip(len.saturating_sub(4)).collect();
    format!("{prefix}••••••••••••{suffix}")
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
