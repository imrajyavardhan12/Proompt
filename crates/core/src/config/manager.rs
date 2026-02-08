use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

use super::Config;

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

    // Fix empty values from stale config files by applying defaults
    if config.byok.provider.is_empty() {
        config.byok.provider = "openai".to_string();
    }
    if config.byok.model.is_empty() {
        config.byok.model = "gpt-4o".to_string();
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
    if service.is_empty() {
        anyhow::bail!("Provider not configured. Run: proompt config set byok.provider openai");
    }

    // 1. Check environment variable first (no keychain prompt)
    let env_key = match service {
        "openai" => std::env::var("OPENAI_API_KEY").ok(),
        "anthropic" => std::env::var("ANTHROPIC_API_KEY").ok(),
        "google" | "gemini" => std::env::var("GEMINI_API_KEY")
            .or_else(|_| std::env::var("GOOGLE_API_KEY"))
            .ok(),
        "supermemory" => std::env::var("SUPERMEMORY_API_KEY").ok(),
        _ => None,
    };

    if let Some(key) = env_key {
        if !key.is_empty() {
            return Ok(key);
        }
    }

    // 2. Fall back to OS keychain
    let entry = keyring::Entry::new(APP_NAME, service)?;
    entry
        .get_password()
        .map_err(|e| anyhow::anyhow!("Failed to get API key for '{}': {}", service, e))
}

pub fn set_api_key(service: &str, key: &str) -> Result<()> {
    if service.is_empty() {
        anyhow::bail!("Provider not configured. Run: proompt config set byok.provider openai");
    }
    let entry = keyring::Entry::new(APP_NAME, service)?;
    entry
        .set_password(key)
        .map_err(|e| anyhow::anyhow!("Failed to store API key for '{}': {}", service, e))
}

pub fn delete_api_key(service: &str) -> Result<()> {
    if service.is_empty() {
        anyhow::bail!("Provider not configured");
    }
    let entry = keyring::Entry::new(APP_NAME, service)?;
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
        assert_eq!(
            config.default_platform,
            crate::platform::Platform::Claude
        );
        assert!(!config.supermemory.enabled);
    }

    #[test]
    fn test_config_roundtrip() {
        let config = Config::default();
        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.mode, config.mode);
        assert_eq!(
            deserialized.default_platform,
            config.default_platform
        );
    }
}
