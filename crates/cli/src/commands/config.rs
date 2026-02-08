use anyhow::Result;
use console::Style;
use proompt_core::config::{self as cfg};

use crate::output;

pub fn show() -> Result<()> {
    let config = cfg::load_config()?;

    let accent = Style::new().cyan();
    let muted = Style::new().dim();
    let bold = Style::new().bold();
    let val = Style::new().white();

    eprintln!();
    output::section_header("Configuration");
    eprintln!();

    eprintln!(
        "  {} {}",
        muted.apply_to("mode:            "),
        bold.apply_to(format!("{:?}", config.mode).to_lowercase())
    );
    eprintln!(
        "  {} {}",
        muted.apply_to("provider:        "),
        accent.apply_to(&config.byok.provider)
    );
    eprintln!(
        "  {} {}",
        muted.apply_to("model:           "),
        val.apply_to(&config.byok.model)
    );
    eprintln!(
        "  {} {}",
        muted.apply_to("default platform:"),
        val.apply_to(config.default_platform.to_string())
    );
    eprintln!(
        "  {} {}",
        muted.apply_to("image platform:  "),
        val.apply_to(config.default_image_platform.to_string())
    );

    eprintln!();
    output::section_header("Integrations");
    eprintln!();

    let sm_status = if config.supermemory.enabled {
        Style::new().green().apply_to("enabled").to_string()
    } else {
        Style::new().dim().apply_to("disabled").to_string()
    };
    eprintln!(
        "  {} {}",
        muted.apply_to("supermemory:     "),
        sm_status
    );

    eprintln!();
    output::dim("  Tip: proompt config set byok.api_key YOUR_KEY to store API keys");
    output::dim("  Tip: proompt config set byok.provider <openai|anthropic|google> to switch");
    eprintln!();

    Ok(())
}

pub fn set(key: &str, value: &str) -> Result<()> {
    // Handle API key storage in keychain
    if key.ends_with(".api_key") || key == "api_key" {
        let service = match key {
            "byok.api_key" | "api_key" => {
                let config = cfg::load_config()?;
                config.byok.provider.clone()
            }
            "supermemory.api_key" => "supermemory".to_string(),
            "openai.api_key" => "openai".to_string(),
            "anthropic.api_key" => "anthropic".to_string(),
            "google.api_key" => "google".to_string(),
            other => other.trim_end_matches(".api_key").to_string(),
        };
        cfg::set_api_key(&service, value)?;
        output::success(&format!("API key for '{}' stored in system keychain", service));
        return Ok(());
    }

    let mut config = cfg::load_config()?;

    match key {
        "mode" => {
            config.mode = match value {
                "byok" => cfg::Mode::Byok,
                "hosted" => cfg::Mode::Hosted,
                _ => anyhow::bail!("Invalid mode. Use 'byok' or 'hosted'"),
            };
        }
        "default_platform" => {
            config.default_platform =
                proompt_core::platform::detect_platform(value);
        }
        "default_image_platform" => {
            config.default_image_platform =
                proompt_core::platform::detect_platform(value);
        }
        "byok.provider" => {
            config.byok.provider = value.to_string();
            if config.byok.model.is_empty()
                || !model_matches_provider(&config.byok.model, value)
            {
                config.byok.model = default_model_for_provider(value).to_string();
                output::dim(&format!("  Model auto-set to: {}", config.byok.model));
            }
        }
        "byok.model" => config.byok.model = value.to_string(),
        "supermemory.enabled" => {
            config.supermemory.enabled = value.parse().unwrap_or(false);
        }
        "supermemory.auto_context" => {
            config.supermemory.auto_context = value.parse().unwrap_or(true);
        }
        "supermemory.context_limit" => {
            config.supermemory.context_limit = value.parse().unwrap_or(5);
        }
        "preferences.theme" => config.preferences.theme = value.to_string(),
        "preferences.launch_at_login" => {
            config.preferences.launch_at_login = value.parse().unwrap_or(false);
        }
        "preferences.telemetry" => {
            config.preferences.telemetry = value.parse().unwrap_or(false);
        }
        _ => anyhow::bail!("Unknown config key: {}", key),
    }

    cfg::save_config(&config)?;
    output::success(&format!("Set {} = {}", key, value));

    Ok(())
}

fn default_model_for_provider(provider: &str) -> &str {
    match provider {
        "openai" => "gpt-4o",
        "anthropic" => "claude-sonnet-4-20250514",
        "google" | "gemini" => "gemini-2.0-flash",
        _ => "gpt-4o",
    }
}

fn model_matches_provider(model: &str, provider: &str) -> bool {
    match provider {
        "openai" => model.starts_with("gpt") || model.starts_with("o1") || model.starts_with("o3"),
        "anthropic" => model.starts_with("claude"),
        "google" | "gemini" => model.starts_with("gemini"),
        _ => true,
    }
}
