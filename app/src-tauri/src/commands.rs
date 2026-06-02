use std::collections::HashMap;

use proompt_core::{
    config::{self as cfg, Mode},
    enhance::{self, EnhanceOptions, EnhanceRequest},
    integrations::supermemory::SuperMemoryClient,
    platform::{EnhanceType, Platform, parse_platform},
    templates::{Template, TemplateFilter, TemplateManager},
};
use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;

#[tauri::command]
pub async fn enhance_prompt(
    prompt: String,
    platform: String,
    enhance_type: String,
    include_memory: bool,
    style_hints: Option<Vec<String>>,
) -> Result<String, String> {
    let config = cfg::load_config().map_err(|e| e.to_string())?;

    let platform = parse_platform(&platform).ok_or_else(|| {
        "Invalid platform. Use claude, openai, gemini, generic, midjourney, dalle, or sd."
            .to_string()
    })?;
    let enhancement_type = match enhance_type.as_str() {
        "image" => EnhanceType::Image,
        _ => EnhanceType::Text,
    };

    let api_key = match config.mode {
        Mode::Byok => cfg::get_api_key(&config.byok.provider).map_err(|e| e.to_string())?,
        Mode::Hosted => {
            return Err("Hosted mode not yet implemented".to_string());
        }
    };

    let normalized_style_hints = style_hints
        .map(|hints| {
            hints
                .into_iter()
                .map(|hint| hint.trim().to_string())
                .filter(|hint| !hint.is_empty())
                .collect::<Vec<_>>()
        })
        .filter(|hints| !hints.is_empty());

    let supermemory_context = if include_memory && config.supermemory.enabled {
        match fetch_supermemory_context(&prompt, config.supermemory.context_limit).await {
            Ok(context) if !context.is_empty() => Some(context),
            _ => None,
        }
    } else {
        None
    };

    let request = EnhanceRequest {
        prompt,
        platform,
        enhancement_type,
        options: EnhanceOptions {
            include_supermemory: include_memory,
            style_hints: if enhancement_type == EnhanceType::Image {
                normalized_style_hints
            } else {
                None
            },
            max_tokens: None,
        },
    };

    let result = enhance::enhance(
        request,
        &config.byok.provider,
        &api_key,
        Some(config.byok.model),
        supermemory_context,
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(result.enhanced_prompt)
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
    if let Some(m) = model {
        config.byok.model = m;
    }

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
pub async fn test_api_connection() -> Result<String, String> {
    let config = cfg::load_config().map_err(|e| e.to_string())?;
    let api_key = cfg::get_api_key(&config.byok.provider).map_err(|e| e.to_string())?;

    let request = proompt_core::integrations::llm::LlmRequest {
        system_prompt: "Respond with only: OK".to_string(),
        user_prompt: "Test".to_string(),
        max_tokens: 10,
    };

    let result = match config.byok.provider.as_str() {
        "openai" => {
            let client = proompt_core::integrations::llm::openai::OpenAIClient::new(
                api_key,
                Some(config.byok.model),
            );
            client.complete(request).await
        }
        "openrouter" => {
            let client = proompt_core::integrations::llm::openai::OpenAIClient::openrouter(
                api_key,
                Some(config.byok.model),
            );
            client.complete(request).await
        }
        "google" | "gemini" => {
            let client = proompt_core::integrations::llm::google::GoogleClient::new(
                api_key,
                Some(config.byok.model),
            );
            client.complete(request).await
        }
        _ => {
            let client = proompt_core::integrations::llm::anthropic::AnthropicClient::new(
                api_key,
                Some(config.byok.model),
            );
            client.complete(request).await
        }
    };

    result
        .map(|_| "Connection successful!".to_string())
        .map_err(|e| e.to_string())
}

async fn fetch_supermemory_context(prompt: &str, limit: u32) -> Result<Vec<String>, String> {
    let api_key = cfg::get_api_key("supermemory").map_err(|e| e.to_string())?;
    let client = SuperMemoryClient::new(api_key);
    let memories = client
        .search(prompt, limit)
        .await
        .map_err(|e| e.to_string())?;
    Ok(memories.into_iter().map(|m| m.content).collect())
}

#[tauri::command]
pub fn copy_to_clipboard(app: AppHandle, text: String) -> Result<(), String> {
    app.clipboard().write_text(text).map_err(|e| e.to_string())
}
