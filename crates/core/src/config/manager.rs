use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

use super::{
    Config, api_key_service_name, default_model_for_provider, model_matches_provider,
    normalize_provider,
};

const APP_NAME: &str = "proompt";
const CONFIG_FILE: &str = "config.toml";

pub fn config_dir() -> Result<PathBuf> {
    let base = dirs::config_dir().context("Could not determine config directory")?;
    Ok(base.join(APP_NAME))
}

pub fn config_path() -> Result<PathBuf> {
    Ok(config_dir()?.join(CONFIG_FILE))
}

pub fn load_config() -> Result<Config> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(Config::default());
    }
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config from {}", path.display()))?;
    let mut config: Config =
        toml::from_str(&content).with_context(|| "Failed to parse config file")?;

    // Fix empty/invalid values from stale config files by applying defaults.
    config.byok.provider = normalize_provider(&config.byok.provider)
        .unwrap_or("openai")
        .to_string();
    if config.byok.model.trim().is_empty()
        || !model_matches_provider(&config.byok.model, &config.byok.provider)
    {
        config.byok.model = default_model_for_provider(&config.byok.provider)
            .unwrap_or("gpt-4o")
            .to_string();
    }

    Ok(config)
}

pub fn save_config(config: &Config) -> Result<()> {
    let dir = config_dir()?;
    fs::create_dir_all(&dir)
        .with_context(|| format!("Failed to create config dir: {}", dir.display()))?;

    let path = config_path()?;
    let content = toml::to_string_pretty(config).context("Failed to serialize config")?;
    fs::write(&path, content)
        .with_context(|| format!("Failed to write config to {}", path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o600);
        fs::set_permissions(&path, perms)?;
    }

    Ok(())
}

pub fn get_api_key(service: &str) -> Result<String> {
    let service = api_key_service_name(service);
    if service.is_empty() {
        anyhow::bail!("Provider not configured. Run: proompt config set byok.provider openai");
    }

    // 1. Check environment variable first (no keychain prompt)
    let env_key = match service.as_str() {
        "openai" => std::env::var("OPENAI_API_KEY").ok(),
        "anthropic" => std::env::var("ANTHROPIC_API_KEY").ok(),
        "google" => std::env::var("GEMINI_API_KEY")
            .or_else(|_| std::env::var("GOOGLE_API_KEY"))
            .ok(),
        "openrouter" => std::env::var("OPENROUTER_API_KEY").ok(),
        "supermemory" => std::env::var("SUPERMEMORY_API_KEY").ok(),
        _ => None,
    };

    if let Some(key) = env_key
        && !key.is_empty()
    {
        return Ok(key);
    }

    if keychain_disabled() {
        anyhow::bail!(
            "API key not found for '{}'; keychain lookup disabled by PROOMPT_DISABLE_KEYCHAIN",
            service
        );
    }

    // 2. Fall back to OS keychain. Google previously used both "google" and
    // "gemini" service names, so read both for backwards compatibility.
    let candidates = match service.as_str() {
        "google" => vec!["google", "gemini"],
        _ => vec![service.as_str()],
    };
    let mut last_error = None;

    for candidate in candidates {
        let entry = match keyring::Entry::new(APP_NAME, candidate) {
            Ok(entry) => entry,
            Err(e) => {
                last_error = Some(e.to_string());
                continue;
            }
        };

        match entry.get_password() {
            Ok(password) => return Ok(password),
            Err(e) => last_error = Some(e.to_string()),
        }
    }

    Err(anyhow::anyhow!(
        "Failed to get API key for '{}': {}",
        service,
        last_error.unwrap_or_else(|| "not found".to_string())
    ))
}

fn keychain_disabled() -> bool {
    std::env::var("PROOMPT_DISABLE_KEYCHAIN")
        .map(|value| matches!(value.to_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false)
}

pub fn set_api_key(service: &str, key: &str) -> Result<()> {
    let service = api_key_service_name(service);
    if service.is_empty() {
        anyhow::bail!("Provider not configured. Run: proompt config set byok.provider openai");
    }
    let entry = keyring::Entry::new(APP_NAME, &service)?;
    entry
        .set_password(key)
        .map_err(|e| anyhow::anyhow!("Failed to store API key for '{}': {}", service, e))
}

pub fn delete_api_key(service: &str) -> Result<()> {
    let service = api_key_service_name(service);
    if service.is_empty() {
        anyhow::bail!("Provider not configured");
    }
    let entry = keyring::Entry::new(APP_NAME, &service)?;
    entry
        .delete_credential()
        .map_err(|e| anyhow::anyhow!("Failed to delete API key for '{}': {}", service, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.mode, super::super::Mode::Byok);
        assert_eq!(config.default_platform, crate::platform::Platform::Claude);
        assert!(!config.supermemory.enabled);
    }

    #[test]
    fn test_config_roundtrip() {
        let config = Config::default();
        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.mode, config.mode);
        assert_eq!(deserialized.default_platform, config.default_platform);
    }
}
