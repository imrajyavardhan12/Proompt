use std::collections::HashMap;

#[cfg(target_os = "macos")]
use std::process::Command;

use proompt_core::{
    config::{self as cfg, Mode},
    enhance::{ConfiguredEnhanceRequest, enhance_with_config, enhance_with_loaded_config},
    history::{self, NewPromptHistoryRecord, PromptHistoryRecord},
    platform::{EnhanceType, Platform, parse_platform},
    routing::{
        ActiveApp, BrowserContext, EnvironmentSnapshot, ResolutionSource, TargetResolution,
        resolve_quick_enhance_input_with_environment,
    },
    templates::{Template, TemplateFilter, TemplateManager},
};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
use tauri_plugin_notification::NotificationExt;

const DEFAULT_QUICK_ENHANCE_HOTKEY: &str = "CmdOrCtrl+Shift+E";
const TEXT_PLATFORM_HELP: &str =
    "claude, claude-code, openai, gemini, cursor, codex, coding-agent, or generic";
const IMAGE_PLATFORM_HELP: &str = "midjourney, dalle, sd, or generic";

#[derive(Debug)]
struct QuickEnhanceOutcome {
    enhanced_prompt: String,
    resolution: TargetResolution,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSettingsInput {
    mode: String,
    provider: String,
    model: Option<String>,
    default_platform: String,
    default_image_platform: Option<String>,
    auto_detect_target: bool,
    terminal_platform: Option<String>,
    supermemory_enabled: bool,
    save_history_enabled: bool,
}

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

pub fn register_quick_enhance_shortcut(app: &AppHandle) {
    let hotkey = cfg::load_config()
        .map(|config| config.hotkeys.quick_enhance)
        .unwrap_or_else(|_| DEFAULT_QUICK_ENHANCE_HOTKEY.to_string());
    let hotkey = if hotkey.trim().is_empty() {
        DEFAULT_QUICK_ENHANCE_HOTKEY.to_string()
    } else {
        hotkey
    };

    let result = app
        .global_shortcut()
        .on_shortcut(hotkey.as_str(), |app, _shortcut, event| {
            if event.state != ShortcutState::Pressed {
                return;
            }

            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = quick_enhance_clipboard_with_notifications(app.clone()).await {
                    notify(
                        &app,
                        "Proompt quick enhance failed",
                        &friendly_quick_enhance_error(&e.to_string()),
                    );
                }
            });
        });

    if let Err(e) = result {
        notify(
            app,
            "Proompt hotkey unavailable",
            &format!("Could not register {}: {}", hotkey, e),
        );
    }
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

    let original_prompt = prompt.clone();
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

    let enhanced_prompt = result.response.enhanced_prompt.clone();
    record_prompt_history_if_enabled(NewPromptHistoryRecord {
        original_prompt,
        enhanced_prompt: enhanced_prompt.clone(),
        enhancement_type: result.enhancement_type,
        platform: result.response.platform,
        provider: result.provider,
        model: result.model,
    });

    Ok(enhanced_prompt)
}

#[tauri::command]
pub async fn quick_enhance_clipboard(app: AppHandle) -> Result<String, String> {
    quick_enhance_clipboard_inner(app)
        .await
        .map(|outcome| outcome.enhanced_prompt)
        .map_err(|e| friendly_quick_enhance_error(&e.to_string()))
}

async fn quick_enhance_clipboard_with_notifications(app: AppHandle) -> anyhow::Result<String> {
    notify(&app, "Proompt", "Enhancing clipboard prompt...");
    let outcome = quick_enhance_clipboard_inner(app.clone()).await?;
    notify(
        &app,
        "Proompt",
        &quick_enhance_success_message(&outcome.resolution),
    );
    Ok(outcome.enhanced_prompt)
}

async fn quick_enhance_clipboard_inner(app: AppHandle) -> anyhow::Result<QuickEnhanceOutcome> {
    let prompt = app.clipboard().read_text()?;
    if prompt.trim().is_empty() {
        anyhow::bail!("Clipboard is empty. Copy a rough prompt first.");
    }

    let config = cfg::load_config()?;
    let environment = collect_environment_snapshot();
    let resolved =
        resolve_quick_enhance_input_with_environment(&config, &prompt, environment.as_ref())?;
    let result = enhance_with_loaded_config(
        ConfiguredEnhanceRequest {
            prompt: resolved.prompt.clone(),
            platform: Some(resolved.resolution.platform.to_string()),
            enhancement_type: Some(EnhanceType::Text),
            include_memory: false,
            style_hints: None,
            max_tokens: None,
        },
        config,
    )
    .await?;

    let enhanced_prompt = result.response.enhanced_prompt.clone();
    app.clipboard().write_text(&enhanced_prompt)?;
    record_prompt_history_if_enabled(NewPromptHistoryRecord {
        original_prompt: resolved.prompt,
        enhanced_prompt: enhanced_prompt.clone(),
        enhancement_type: result.enhancement_type,
        platform: result.response.platform,
        provider: result.provider,
        model: result.model,
    });
    Ok(QuickEnhanceOutcome {
        enhanced_prompt,
        resolution: resolved.resolution,
    })
}

fn quick_enhance_success_message(resolution: &TargetResolution) -> String {
    match resolution.source {
        ResolutionSource::ConfigDefault => format!("Enhanced for {}.", resolution.platform.label()),
        _ => format!(
            "Enhanced for {} ({}).",
            resolution.platform.label(),
            resolution.reason
        ),
    }
}

fn collect_environment_snapshot() -> Option<EnvironmentSnapshot> {
    collect_active_app().map(|(active_app, window_title)| {
        let browser_context = collect_browser_context(&active_app);
        EnvironmentSnapshot {
            active_app: Some(active_app),
            window_title,
            browser_context,
            terminal_context: None,
        }
    })
}

#[cfg(target_os = "macos")]
fn collect_active_app() -> Option<(ActiveApp, Option<String>)> {
    let script = r#"
        tell application "System Events"
            set frontApp to first application process whose frontmost is true
            set appName to name of frontApp
            set bundleId to bundle identifier of frontApp
            set windowTitle to ""
            try
                set windowTitle to name of front window of frontApp
            end try
            return appName & linefeed & bundleId & linefeed & windowTitle
        end tell
    "#;

    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut lines = stdout.lines();
    let app_name = lines.next()?.trim();
    if app_name.is_empty() {
        return None;
    }
    let bundle_id = lines
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let window_title = lines
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    let mut app = ActiveApp::new(app_name);
    if let Some(bundle_id) = bundle_id {
        app = app.with_bundle_id(bundle_id);
    }

    Some((app, window_title.map(str::to_string)))
}

#[cfg(target_os = "macos")]
fn collect_browser_context(active_app: &ActiveApp) -> Option<BrowserContext> {
    let app_name = active_app.name.to_ascii_lowercase();
    let script = if app_name.contains("safari") {
        r#"
            tell application "Safari"
                set activeTab to current tab of front window
                return URL of activeTab & linefeed & name of activeTab
            end tell
        "#
    } else if app_name.contains("chrome") {
        chromium_browser_script("Google Chrome")
    } else if app_name.contains("brave") {
        chromium_browser_script("Brave Browser")
    } else if app_name.contains("edge") {
        chromium_browser_script("Microsoft Edge")
    } else if app_name.contains("arc") {
        chromium_browser_script("Arc")
    } else {
        return None;
    };

    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut lines = stdout.lines();
    let url = lines
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let title = lines
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    if url.is_none() && title.is_none() {
        return None;
    }

    Some(BrowserContext {
        url: url.map(str::to_string),
        title: title.map(str::to_string),
    })
}

#[cfg(target_os = "macos")]
fn chromium_browser_script(app_name: &str) -> &'static str {
    match app_name {
        "Google Chrome" => {
            r#"
                tell application "Google Chrome"
                    set activeTab to active tab of front window
                    return URL of activeTab & linefeed & title of activeTab
                end tell
            "#
        }
        "Brave Browser" => {
            r#"
                tell application "Brave Browser"
                    set activeTab to active tab of front window
                    return URL of activeTab & linefeed & title of activeTab
                end tell
            "#
        }
        "Microsoft Edge" => {
            r#"
                tell application "Microsoft Edge"
                    set activeTab to active tab of front window
                    return URL of activeTab & linefeed & title of activeTab
                end tell
            "#
        }
        "Arc" => {
            r#"
                tell application "Arc"
                    set activeTab to active tab of front window
                    return URL of activeTab & linefeed & title of activeTab
                end tell
            "#
        }
        _ => unreachable!("only known Chromium browser names are used"),
    }
}

#[cfg(not(target_os = "macos"))]
fn collect_active_app() -> Option<(ActiveApp, Option<String>)> {
    None
}

#[cfg(not(target_os = "macos"))]
fn collect_browser_context(_active_app: &ActiveApp) -> Option<BrowserContext> {
    None
}

fn record_prompt_history_if_enabled(record: NewPromptHistoryRecord) {
    let save_history = cfg::load_config()
        .map(|config| config.preferences.save_history)
        .unwrap_or(true);
    if save_history {
        let _ = history::append_history_record(record);
    }
}

fn notify(app: &AppHandle, title: &str, body: &str) {
    let _ = app.notification().builder().title(title).body(body).show();
}

fn friendly_quick_enhance_error(message: &str) -> String {
    let lower = message.to_lowercase();
    if lower.contains("api key not configured")
        || lower.contains("failed to get api key")
        || lower.contains("api key not found")
    {
        "Add a provider API key in Settings before using quick enhance.".to_string()
    } else if lower.contains("hosted mode") {
        "Hosted mode is coming soon. Switch to BYOK mode in Settings.".to_string()
    } else {
        message.to_string()
    }
}

#[tauri::command]
pub fn list_history(
    limit: Option<usize>,
    favorites_only: Option<bool>,
) -> Result<Vec<PromptHistoryRecord>, String> {
    let mut records = history::load_history().map_err(|e| e.to_string())?;
    if favorites_only.unwrap_or(false) {
        records.retain(|record| record.favorite);
    }
    if let Some(limit) = limit {
        records.truncate(limit);
    }
    Ok(records)
}

#[tauri::command]
pub fn set_history_favorite(id: String, favorite: bool) -> Result<PromptHistoryRecord, String> {
    history::set_history_favorite(&id, favorite).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_history_record(id: String) -> Result<bool, String> {
    history::delete_history_record(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_prompt_history() -> Result<usize, String> {
    history::clear_history().map_err(|e| e.to_string())
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
pub fn save_settings(input: SaveSettingsInput) -> Result<(), String> {
    let mut config = cfg::load_config().map_err(|e| e.to_string())?;
    config.mode = match input.mode.as_str() {
        "hosted" => Mode::Hosted,
        _ => Mode::Byok,
    };
    cfg::set_byok_provider(&mut config, &input.provider).map_err(|e| e.to_string())?;
    let model = input.model.unwrap_or_else(|| config.byok.model.clone());
    let model = model.trim().to_string();
    if model.is_empty() {
        return Err("Model is required".to_string());
    }
    if !cfg::model_matches_provider(&model, &config.byok.provider) {
        return Err(model_validation_message(&config.byok.provider));
    }
    config.byok.model = model;

    let default_platform = parse_platform(&input.default_platform)
        .ok_or_else(|| format!("Default platform must be {}", TEXT_PLATFORM_HELP))?;
    if !default_platform.is_text_platform() {
        return Err(format!("Default platform must be {}", TEXT_PLATFORM_HELP));
    }
    config.default_platform = default_platform;

    if let Some(default_image_platform) = input.default_image_platform {
        let default_image_platform = parse_platform(&default_image_platform)
            .ok_or_else(|| format!("Default image platform must be {}", IMAGE_PLATFORM_HELP))?;
        if !default_image_platform.is_image_platform()
            && default_image_platform != Platform::Generic
        {
            return Err(format!(
                "Default image platform must be {}",
                IMAGE_PLATFORM_HELP
            ));
        }
        config.default_image_platform = default_image_platform;
    }

    config.quick_enhance.auto_detect_target = input.auto_detect_target;
    config.quick_enhance.terminal_platform = match input.terminal_platform.as_deref().map(str::trim)
    {
        Some("") | None => None,
        Some("none" | "off" | "default") => None,
        Some(platform) => {
            let platform = parse_platform(platform)
                .ok_or_else(|| format!("Terminal platform must be {}", TEXT_PLATFORM_HELP))?;
            if !platform.is_text_platform() {
                return Err(format!("Terminal platform must be {}", TEXT_PLATFORM_HELP));
            }
            Some(platform)
        }
    };

    config.supermemory.enabled = input.supermemory_enabled;
    config.preferences.save_history = input.save_history_enabled;
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
