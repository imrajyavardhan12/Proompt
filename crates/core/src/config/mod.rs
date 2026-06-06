mod manager;

pub use manager::*;

use serde::{Deserialize, Serialize};

use crate::platform::Platform;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_mode")]
    pub mode: Mode,
    #[serde(default = "default_platform")]
    pub default_platform: Platform,
    #[serde(default = "default_image_platform")]
    pub default_image_platform: Platform,
    #[serde(default)]
    pub byok: ByokConfig,
    #[serde(default)]
    pub hosted: HostedConfig,
    #[serde(default)]
    pub supermemory: SuperMemoryConfig,
    #[serde(default)]
    pub hotkeys: HotkeyConfig,
    #[serde(default)]
    pub quick_enhance: QuickEnhanceConfig,
    #[serde(default)]
    pub preferences: PreferencesConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: Mode::Byok,
            default_platform: Platform::Claude,
            default_image_platform: Platform::Midjourney,
            byok: ByokConfig::default(),
            hosted: HostedConfig::default(),
            supermemory: SuperMemoryConfig::default(),
            hotkeys: HotkeyConfig::default(),
            quick_enhance: QuickEnhanceConfig::default(),
            preferences: PreferencesConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Byok,
    Hosted,
}

fn default_mode() -> Mode {
    Mode::Byok
}
fn default_platform() -> Platform {
    Platform::Claude
}
fn default_image_platform() -> Platform {
    Platform::Midjourney
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByokConfig {
    #[serde(default = "default_provider")]
    pub provider: String,
    #[serde(default = "default_model")]
    pub model: String,
}

impl Default for ByokConfig {
    fn default() -> Self {
        Self {
            provider: default_provider(),
            model: default_model(),
        }
    }
}

fn default_provider() -> String {
    "openai".to_string()
}
fn default_model() -> String {
    "gpt-4o".to_string()
}

pub const OPENAI_PROVIDER: &str = "openai";
pub const ANTHROPIC_PROVIDER: &str = "anthropic";
pub const GOOGLE_PROVIDER: &str = "google";
pub const OPENROUTER_PROVIDER: &str = "openrouter";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderUpdate {
    pub provider: String,
    pub model_changed: bool,
}

pub fn normalize_provider(provider: &str) -> Option<&'static str> {
    match provider.trim().to_lowercase().as_str() {
        "openai" | "gpt" | "chatgpt" => Some(OPENAI_PROVIDER),
        "anthropic" | "claude" => Some(ANTHROPIC_PROVIDER),
        "google" | "gemini" => Some(GOOGLE_PROVIDER),
        "openrouter" | "open-router" | "or" => Some(OPENROUTER_PROVIDER),
        _ => None,
    }
}

pub fn api_key_service_name(service: &str) -> String {
    normalize_provider(service)
        .unwrap_or_else(|| service.trim())
        .to_string()
}

pub fn api_key_env_vars_for_service(service: &str) -> &'static [&'static str] {
    let service = api_key_service_name(service);
    match service.as_str() {
        OPENAI_PROVIDER => &["OPENAI_API_KEY"],
        ANTHROPIC_PROVIDER => &["ANTHROPIC_API_KEY"],
        GOOGLE_PROVIDER => &["GEMINI_API_KEY", "GOOGLE_API_KEY"],
        OPENROUTER_PROVIDER => &["OPENROUTER_API_KEY"],
        "supermemory" => &["SUPERMEMORY_API_KEY"],
        _ => &[],
    }
}

pub fn preferred_api_key_env_var(service: &str) -> Option<&'static str> {
    api_key_env_vars_for_service(service).first().copied()
}

pub fn default_model_for_provider(provider: &str) -> Option<&'static str> {
    let provider = normalize_provider(provider)?;
    Some(match provider {
        OPENAI_PROVIDER => "gpt-4o",
        ANTHROPIC_PROVIDER => "claude-sonnet-4-20250514",
        GOOGLE_PROVIDER => "gemini-2.0-flash",
        OPENROUTER_PROVIDER => "openai/gpt-4o-mini",
        _ => return None,
    })
}

pub fn model_matches_provider(model: &str, provider: &str) -> bool {
    let Some(provider) = normalize_provider(provider) else {
        return false;
    };
    let model = model.trim().to_lowercase();

    match provider {
        OPENAI_PROVIDER => {
            model.starts_with("gpt")
                || model.starts_with("chatgpt")
                || model.starts_with("o1")
                || model.starts_with("o3")
                || model.starts_with("o4")
        }
        ANTHROPIC_PROVIDER => model.starts_with("claude"),
        GOOGLE_PROVIDER => model.starts_with("gemini"),
        OPENROUTER_PROVIDER => model
            .split_once('/')
            .is_some_and(|(provider, model)| !provider.is_empty() && !model.is_empty()),
        _ => false,
    }
}

pub fn set_byok_provider(config: &mut Config, provider: &str) -> anyhow::Result<ProviderUpdate> {
    let normalized = normalize_provider(provider).ok_or_else(|| {
        anyhow::anyhow!(
            "Invalid provider '{}'. Use openai, anthropic, google, or openrouter.",
            provider
        )
    })?;

    let previous_model = config.byok.model.clone();
    config.byok.provider = normalized.to_string();

    if config.byok.model.trim().is_empty()
        || !model_matches_provider(&config.byok.model, normalized)
    {
        let default_model = default_model_for_provider(normalized).ok_or_else(|| {
            anyhow::anyhow!("Provider '{}' does not have a default model", normalized)
        })?;
        config.byok.model = default_model.to_string();
    }

    Ok(ProviderUpdate {
        provider: config.byok.provider.clone(),
        model_changed: config.byok.model != previous_model,
    })
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HostedConfig {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuperMemoryConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub auto_context: bool,
    #[serde(default = "default_context_limit")]
    pub context_limit: u32,
}

impl Default for SuperMemoryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            auto_context: true,
            context_limit: 5,
        }
    }
}

fn default_true() -> bool {
    true
}
fn default_context_limit() -> u32 {
    5
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    #[serde(default = "default_quick_enhance")]
    pub quick_enhance: String,
    #[serde(default = "default_open_window")]
    pub open_window: String,
    #[serde(default = "default_open_templates")]
    pub open_templates: String,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            quick_enhance: default_quick_enhance(),
            open_window: default_open_window(),
            open_templates: default_open_templates(),
        }
    }
}

fn default_quick_enhance() -> String {
    "CmdOrCtrl+Shift+E".to_string()
}
fn default_open_window() -> String {
    "CmdOrCtrl+Shift+P".to_string()
}
fn default_open_templates() -> String {
    "CmdOrCtrl+Shift+T".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickEnhanceConfig {
    #[serde(default = "default_auto_detect_target")]
    pub auto_detect_target: bool,
    #[serde(default = "default_selected_text_enabled")]
    pub selected_text_enabled: bool,
    #[serde(default)]
    pub terminal_platform: Option<Platform>,
}

impl Default for QuickEnhanceConfig {
    fn default() -> Self {
        Self {
            auto_detect_target: true,
            selected_text_enabled: true,
            terminal_platform: None,
        }
    }
}

fn default_auto_detect_target() -> bool {
    true
}
fn default_selected_text_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferencesConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default)]
    pub launch_at_login: bool,
    #[serde(default)]
    pub telemetry: bool,
    #[serde(default = "default_save_history")]
    pub save_history: bool,
}

impl Default for PreferencesConfig {
    fn default() -> Self {
        Self {
            theme: "system".to_string(),
            launch_at_login: false,
            telemetry: false,
            save_history: true,
        }
    }
}

fn default_theme() -> String {
    "system".to_string()
}
fn default_save_history() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_provider_accepts_common_aliases() {
        assert_eq!(normalize_provider("openai"), Some(OPENAI_PROVIDER));
        assert_eq!(normalize_provider("GPT"), Some(OPENAI_PROVIDER));
        assert_eq!(normalize_provider("claude"), Some(ANTHROPIC_PROVIDER));
        assert_eq!(normalize_provider("gemini"), Some(GOOGLE_PROVIDER));
        assert_eq!(normalize_provider("open-router"), Some(OPENROUTER_PROVIDER));
        assert_eq!(normalize_provider("unknown"), None);
    }

    #[test]
    fn set_byok_provider_normalizes_alias_and_updates_incompatible_model() {
        let mut config = Config::default();
        config.byok.model = "gpt-4o".to_string();

        let update = set_byok_provider(&mut config, "claude").unwrap();

        assert_eq!(update.provider, ANTHROPIC_PROVIDER);
        assert!(update.model_changed);
        assert_eq!(config.byok.provider, ANTHROPIC_PROVIDER);
        assert_eq!(config.byok.model, "claude-sonnet-4-20250514");
    }

    #[test]
    fn set_byok_provider_preserves_compatible_model() {
        let mut config = Config::default();
        config.byok.model = "gpt-4o-mini".to_string();

        let update = set_byok_provider(&mut config, "chatgpt").unwrap();

        assert_eq!(update.provider, OPENAI_PROVIDER);
        assert!(!update.model_changed);
        assert_eq!(config.byok.model, "gpt-4o-mini");
    }

    #[test]
    fn set_byok_provider_supports_openrouter_defaults() {
        let mut config = Config::default();
        config.byok.model = "gpt-4o".to_string();

        let update = set_byok_provider(&mut config, "openrouter").unwrap();

        assert_eq!(update.provider, OPENROUTER_PROVIDER);
        assert!(update.model_changed);
        assert_eq!(config.byok.model, "openai/gpt-4o-mini");
    }

    #[test]
    fn set_byok_provider_preserves_openrouter_model_ids() {
        let mut config = Config::default();
        config.byok.provider = OPENROUTER_PROVIDER.to_string();
        config.byok.model = "anthropic/claude-3.5-sonnet".to_string();

        let update = set_byok_provider(&mut config, "or").unwrap();

        assert_eq!(update.provider, OPENROUTER_PROVIDER);
        assert!(!update.model_changed);
        assert_eq!(config.byok.model, "anthropic/claude-3.5-sonnet");
    }

    #[test]
    fn set_byok_provider_rejects_unknown_provider() {
        let mut config = Config::default();

        let result = set_byok_provider(&mut config, "local-llm");

        assert!(result.is_err());
    }

    #[test]
    fn api_key_service_name_normalizes_provider_aliases() {
        assert_eq!(api_key_service_name("gemini"), GOOGLE_PROVIDER);
        assert_eq!(api_key_service_name("open-router"), OPENROUTER_PROVIDER);
        assert_eq!(api_key_service_name("supermemory"), "supermemory");
    }

    #[test]
    fn preferences_default_to_saving_history_locally() {
        let config = Config::default();

        assert!(config.preferences.save_history);
    }

    #[test]
    fn api_key_env_vars_include_provider_specific_fallbacks() {
        assert_eq!(
            api_key_env_vars_for_service("openrouter"),
            &["OPENROUTER_API_KEY"]
        );
        assert_eq!(
            api_key_env_vars_for_service("gemini"),
            &["GEMINI_API_KEY", "GOOGLE_API_KEY"]
        );
        assert_eq!(
            preferred_api_key_env_var("supermemory"),
            Some("SUPERMEMORY_API_KEY")
        );
        assert!(api_key_env_vars_for_service("unknown").is_empty());
    }
}
