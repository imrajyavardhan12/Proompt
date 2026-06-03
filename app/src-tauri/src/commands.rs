use std::collections::HashMap;

use proompt_core::{
    config::{self as cfg, Mode},
    enhance::{ConfiguredEnhanceRequest, enhance_with_config},
    platform::{EnhanceType, Platform, parse_platform},
    templates::{Template, TemplateFilter, TemplateManager},
};
use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;

#[derive(Debug, Serialize)]
pub struct ProviderSetupStatus {
    pub mode: String,
    pub provider: String,
    pub model: String,
    pub api_key_configured: bool,
    pub api_key_error: Option<String>,
    pub env_var: String,
    pub cli_command: String,
}

#[tauri::command]
pub async fn enhance_prompt(
    prompt: String,
    platform: String,
    enhance_type: String,
    include_memory: bool,
    style_hints: Option<Vec<String>>,
) -> Result<String, String> {
    let enhancement_type = match enhance_type.as_str() {
        "image" => EnhanceType::Image,
        _ => EnhanceType::Text,
    };

    let result = enhance_with_config(ConfiguredEnhanceRequest {
        prompt,
        platform: Some(platform),
        enhancement_type: Some(enhancement_type),
        include_memory,
        style_hints,
        max_tokens: None,
    })
    .await
    .map_err(|e| e.to_string())?;

    Ok(result.response.enhanced_prompt)
}

#[tauri::command]
pub fn list_templates() -> Result<Vec<Template>, String> {
    let manager = TemplateManager::new();
    let templates = manager.list(&TemplateFilter::default());
    Ok(templates.into_iter().cloned().collect())
}

#[tauri::command]
pub fn apply_template(
    template_id: String,
    fields: HashMap<String, String>,
) -> Result<String, String> {
    let manager = TemplateManager::new();
    manager
        .apply(&template_id, &fields)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_config() -> Result<cfg::Config, String> {
    cfg::load_config().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_provider_setup_status() -> Result<ProviderSetupStatus, String> {
    let config = cfg::load_config().map_err(|e| e.to_string())?;
    let provider = cfg::normalize_provider(&config.byok.provider)
        .unwrap_or(cfg::OPENAI_PROVIDER)
        .to_string();
    let model = if cfg::model_matches_provider(&config.byok.model, &provider) {
        config.byok.model.clone()
    } else {
        cfg::default_model_for_provider(&provider)
            .unwrap_or("gpt-4o")
            .to_string()
    };
    let env_var = cfg::preferred_api_key_env_var(&provider)
        .unwrap_or("<PROVIDER>_API_KEY")
        .to_string();
    let cli_command = format!("proompt config set {}.api_key YOUR_KEY", provider);

    let (api_key_configured, api_key_error) = match config.mode {
        Mode::Byok => match cfg::get_api_key(&provider) {
            Ok(key) if !key.trim().is_empty() => (true, None),
            Ok(_) => (
                false,
                Some(format!("Empty API key configured for '{}'", provider)),
            ),
            Err(e) => (false, Some(e.to_string())),
        },
        Mode::Hosted => (
            false,
            Some("Hosted mode is not implemented yet. Switch to BYOK mode.".to_string()),
        ),
    };

    Ok(ProviderSetupStatus {
        mode: match config.mode {
            Mode::Byok => "byok".to_string(),
            Mode::Hosted => "hosted".to_string(),
        },
        provider,
        model,
        api_key_configured,
        api_key_error,
        env_var,
        cli_command,
    })
}

#[tauri::command]
pub fn save_settings(
    mode: String,
    provider: String,
    model: Option<String>,
    default_platform: String,
    default_image_platform: Option<String>,
    supermemory_enabled: bool,
) -> Result<(), String> {
    let mut config = cfg::load_config().map_err(|e| e.to_string())?;
    config.mode = match mode.as_str() {
        "hosted" => Mode::Hosted,
        _ => Mode::Byok,
    };
    cfg::set_byok_provider(&mut config, &provider).map_err(|e| e.to_string())?;
    let model = model.unwrap_or_else(|| config.byok.model.clone());
    let model = model.trim().to_string();
    if model.is_empty() {
        return Err("Model is required".to_string());
    }
    if !cfg::model_matches_provider(&model, &config.byok.provider) {
        return Err(model_validation_message(&config.byok.provider));
    }
    config.byok.model = model;

    let default_platform = parse_platform(&default_platform)
        .ok_or_else(|| "Default platform must be claude, openai, gemini, or generic".to_string())?;
    if !default_platform.is_text_platform() {
        return Err("Default platform must be claude, openai, gemini, or generic".to_string());
    }
    config.default_platform = default_platform;

    if let Some(default_image_platform) = default_image_platform {
        let default_image_platform = parse_platform(&default_image_platform).ok_or_else(|| {
            "Default image platform must be midjourney, dalle, sd, or generic".to_string()
        })?;
        if !default_image_platform.is_image_platform()
            && default_image_platform != Platform::Generic
        {
            return Err(
                "Default image platform must be midjourney, dalle, sd, or generic".to_string(),
            );
        }
        config.default_image_platform = default_image_platform;
    }

    config.supermemory.enabled = supermemory_enabled;
    cfg::save_config(&config).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_api_key(service: String, key: String) -> Result<(), String> {
    cfg::set_api_key(&service, &key).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn test_api_connection(
    provider: Option<String>,
    model: Option<String>,
    api_key: Option<String>,
) -> Result<String, String> {
    let config = cfg::load_config().map_err(|e| e.to_string())?;
    let provider = match provider {
        Some(provider) => cfg::normalize_provider(&provider)
            .ok_or_else(|| "Provider must be openai, anthropic, google, or openrouter".to_string())?
            .to_string(),
        None => config.byok.provider.clone(),
    };

    let model = model.unwrap_or_else(|| {
        if cfg::model_matches_provider(&config.byok.model, &provider) {
            config.byok.model.clone()
        } else {
            cfg::default_model_for_provider(&provider)
                .unwrap_or("gpt-4o")
                .to_string()
        }
    });
    let model = model.trim().to_string();
    if model.is_empty() {
        return Err("Model is required".to_string());
    }
    if !cfg::model_matches_provider(&model, &provider) {
        return Err(model_validation_message(&provider));
    }

    let api_key = match api_key.map(|key| key.trim().to_string()) {
        Some(key) if !key.is_empty() => key,
        _ => cfg::get_api_key(&provider).map_err(|e| e.to_string())?,
    };

    let request = proompt_core::integrations::llm::LlmRequest {
        system_prompt: "Respond with only: OK".to_string(),
        user_prompt: "Test".to_string(),
        max_tokens: 10,
    };

    let result = match provider.as_str() {
        "openai" => {
            let client = proompt_core::integrations::llm::openai::OpenAIClient::new(
                api_key,
                Some(model.clone()),
            );
            client.complete(request).await
        }
        "openrouter" => {
            let client = proompt_core::integrations::llm::openai::OpenAIClient::openrouter(
                api_key,
                Some(model.clone()),
            );
            client.complete(request).await
        }
        "google" => {
            let client = proompt_core::integrations::llm::google::GoogleClient::new(
                api_key,
                Some(model.clone()),
            );
            client.complete(request).await
        }
        "anthropic" => {
            let client = proompt_core::integrations::llm::anthropic::AnthropicClient::new(
                api_key,
                Some(model.clone()),
            );
            client.complete(request).await
        }
        _ => unreachable!("provider was normalized before matching"),
    };

    result
        .map(|_| format!("Connection successful via {} / {}", provider, model))
        .map_err(|e| e.to_string())
}

fn model_validation_message(provider: &str) -> String {
    match cfg::normalize_provider(provider) {
        Some(cfg::OPENAI_PROVIDER) => {
            "OpenAI model must start with gpt, chatgpt, o1, o3, or o4".to_string()
        }
        Some(cfg::ANTHROPIC_PROVIDER) => "Anthropic model must start with claude".to_string(),
        Some(cfg::GOOGLE_PROVIDER) => "Google model must start with gemini".to_string(),
        Some(cfg::OPENROUTER_PROVIDER) => {
            "OpenRouter model must use provider/model-id format".to_string()
        }
        _ => "Unsupported provider".to_string(),
    }
}

#[tauri::command]
pub fn copy_to_clipboard(app: AppHandle, text: String) -> Result<(), String> {
    app.clipboard().write_text(text).map_err(|e| e.to_string())
}
