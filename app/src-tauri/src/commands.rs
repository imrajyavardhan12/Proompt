use std::{
    collections::HashMap,
    fs,
    sync::atomic::{AtomicBool, Ordering},
};

#[cfg(target_os = "macos")]
use core_foundation::{
    base::{CFRelease, CFTypeRef, TCFType},
    boolean::CFBoolean,
    dictionary::{CFDictionary, CFDictionaryRef},
    string::{CFString, CFStringRef},
};
#[cfg(target_os = "macos")]
use core_graphics::{
    event::{CGEvent, CGEventFlags, CGEventTapLocation},
    event_source::{CGEventSource, CGEventSourceStateID},
};
#[cfg(target_os = "macos")]
use std::process::Command;

use proompt_core::{
    config::{self as cfg, Mode},
    enhance::{ConfiguredEnhanceRequest, enhance_with_config, enhance_with_loaded_config},
    history::{self, NewPromptHistoryRecord, PromptHistoryRecord},
    platform::{EnhanceType, Platform, parse_platform},
    routing::{
        ActiveApp, BrowserContext, EnvironmentSnapshot, TargetResolution,
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

static QUICK_ENHANCE_IN_FLIGHT: AtomicBool = AtomicBool::new(false);

struct QuickEnhanceInFlightGuard;

impl QuickEnhanceInFlightGuard {
    fn try_acquire() -> anyhow::Result<Self> {
        QUICK_ENHANCE_IN_FLIGHT
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .map(|_| Self)
            .map_err(|_| {
                anyhow::anyhow!(
                    "Quick Enhance is already running. Wait for the current enhancement to finish."
                )
            })
    }
}

impl Drop for QuickEnhanceInFlightGuard {
    fn drop(&mut self) {
        QUICK_ENHANCE_IN_FLIGHT.store(false, Ordering::Release);
    }
}

#[derive(Debug)]
struct QuickEnhanceOutcome {
    enhanced_prompt: String,
    resolution: TargetResolution,
    input_source: QuickEnhanceInputSource,
    delivery: QuickEnhanceDelivery,
    /// Why selected text was copied instead of replaced, when known.
    delivery_note: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum QuickEnhanceInputSource {
    SelectedText,
    Clipboard,
}

impl QuickEnhanceInputSource {
    fn label(&self) -> &'static str {
        match self {
            QuickEnhanceInputSource::SelectedText => "selected text",
            QuickEnhanceInputSource::Clipboard => "clipboard prompt",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum QuickEnhanceDelivery {
    ReplacedSelection,
    CopiedToClipboard,
}

#[derive(Debug)]
struct QuickEnhanceInput {
    prompt: String,
    source: QuickEnhanceInputSource,
    original_clipboard: Option<String>,
    accessibility_untrusted: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SelectionCaptureDiagnostics {
    timestamp_ms: u64,
    selected_text_enabled: bool,
    outcome: String,
    steps: Vec<String>,
}

impl SelectionCaptureDiagnostics {
    fn new(selected_text_enabled: bool) -> Self {
        Self {
            timestamp_ms: now_ms(),
            selected_text_enabled,
            outcome: "not_started".to_string(),
            steps: Vec::new(),
        }
    }

    fn step(&mut self, message: impl Into<String>) {
        self.steps.push(message.into());
    }
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
    selected_text_enabled: bool,
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
    pub api_key_status: String,
    pub api_key_error: Option<String>,
    pub env_var: String,
    pub cli_command: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickEnhanceRouteInspection {
    pub prompt_preview: Option<String>,
    pub environment: Option<EnvironmentSnapshot>,
    pub resolution: Option<TargetResolution>,
    pub error: Option<String>,
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
            if event.state != ShortcutState::Released {
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
        routing: None,
    });

    Ok(enhanced_prompt)
}

#[tauri::command]
pub async fn quick_enhance_clipboard(app: AppHandle) -> Result<String, String> {
    quick_enhance_from_available_input(app, false)
        .await
        .map(|outcome| outcome.enhanced_prompt)
        .map_err(|e| friendly_quick_enhance_error(&e.to_string()))
}

async fn quick_enhance_clipboard_with_notifications(app: AppHandle) -> anyhow::Result<String> {
    // Let physical hotkey modifiers settle before sending synthetic Cmd+C/Cmd+V.
    // Without this, Cmd+Shift+E can leak Shift into capture as Cmd+Shift+C.
    tokio::time::sleep(std::time::Duration::from_millis(120)).await;
    let outcome = quick_enhance_from_available_input(app.clone(), true).await?;
    notify(&app, "Proompt", &quick_enhance_success_message(&outcome));
    Ok(outcome.enhanced_prompt)
}

async fn quick_enhance_from_available_input(
    app: AppHandle,
    notify_progress: bool,
) -> anyhow::Result<QuickEnhanceOutcome> {
    let _in_flight = QuickEnhanceInFlightGuard::try_acquire()?;
    let config = cfg::load_config()?;
    let environment = collect_environment_snapshot();
    let captured = capture_quick_enhance_input(&app, &config).await?;

    if notify_progress {
        let progress = if captured.source == QuickEnhanceInputSource::Clipboard
            && captured.accessibility_untrusted
        {
            "Selected text needs Accessibility permission (Settings > Selected-text diagnostics). Enhancing clipboard prompt instead...".to_string()
        } else if captured.source == QuickEnhanceInputSource::Clipboard
            && config.quick_enhance.selected_text_enabled
        {
            "No selected text detected; enhancing clipboard prompt...".to_string()
        } else {
            format!("Enhancing {}...", captured.source.label())
        };
        notify(&app, "Proompt", &progress);
    }

    let resolved = resolve_quick_enhance_input_with_environment(
        &config,
        &captured.prompt,
        environment.as_ref(),
    )?;
    let original_prompt = resolved.prompt.clone();
    let resolution = resolved.resolution.clone();
    let result = enhance_with_loaded_config(
        ConfiguredEnhanceRequest {
            prompt: resolved.prompt,
            platform: Some(resolution.platform.to_string()),
            enhancement_type: Some(EnhanceType::Text),
            include_memory: false,
            style_hints: None,
            max_tokens: None,
        },
        config,
    )
    .await?;

    let enhanced_prompt = result.response.enhanced_prompt.clone();
    let (delivery, delivery_note) =
        deliver_quick_enhance_output(&app, &enhanced_prompt, &captured, environment.as_ref())
            .await?;
    record_prompt_history_if_enabled(NewPromptHistoryRecord {
        original_prompt,
        enhanced_prompt: enhanced_prompt.clone(),
        enhancement_type: result.enhancement_type,
        platform: result.response.platform,
        provider: result.provider,
        model: result.model,
        routing: Some(resolution.clone().into()),
    });
    Ok(QuickEnhanceOutcome {
        enhanced_prompt,
        resolution,
        input_source: captured.source,
        delivery,
        delivery_note,
    })
}

async fn capture_quick_enhance_input(
    app: &AppHandle,
    config: &cfg::Config,
) -> anyhow::Result<QuickEnhanceInput> {
    let original_clipboard = app.clipboard().read_text().ok();
    let accessibility_untrusted =
        config.quick_enhance.selected_text_enabled && accessibility_trusted() == Some(false);
    let mut diagnostics =
        SelectionCaptureDiagnostics::new(config.quick_enhance.selected_text_enabled);
    if let Some((active_app, window_title)) = collect_active_app() {
        diagnostics.step(format!(
            "active app: {} ({})",
            active_app.name,
            active_app
                .bundle_id
                .unwrap_or_else(|| "no bundle id".to_string())
        ));
        diagnostics.step(format!(
            "active window: {}",
            window_title.unwrap_or_else(|| "unavailable".to_string())
        ));
    } else {
        diagnostics.step("active app: unavailable");
    }
    diagnostics.step(format!(
        "original clipboard: {}",
        original_clipboard
            .as_ref()
            .map(|text| format!("{} chars", text.chars().count()))
            .unwrap_or_else(|| "unavailable".to_string())
    ));

    if config.quick_enhance.selected_text_enabled {
        if let Some(selected_text) = capture_selected_text_via_accessibility(&mut diagnostics) {
            diagnostics.outcome = format!(
                "selected_text_via_accessibility:{} chars",
                selected_text.chars().count()
            );
            write_selection_capture_diagnostics(&diagnostics);
            return Ok(QuickEnhanceInput {
                prompt: selected_text,
                source: QuickEnhanceInputSource::SelectedText,
                original_clipboard,
                accessibility_untrusted,
            });
        }

        if original_clipboard.is_some() {
            if let Some(selected_text) = capture_selected_text_via_clipboard(
                app,
                original_clipboard.as_ref(),
                &mut diagnostics,
            )
            .await
            {
                diagnostics.outcome = format!(
                    "selected_text_via_clipboard:{} chars",
                    selected_text.chars().count()
                );
                write_selection_capture_diagnostics(&diagnostics);
                return Ok(QuickEnhanceInput {
                    prompt: selected_text,
                    source: QuickEnhanceInputSource::SelectedText,
                    original_clipboard,
                    accessibility_untrusted,
                });
            }
        } else {
            diagnostics.step(
                "clipboard fallback skipped: original text clipboard unavailable; preserving non-text clipboard",
            );
        }
    } else {
        diagnostics.step("selected text capture disabled by config");
    }

    let prompt = original_clipboard.clone().unwrap_or_default();
    if prompt.trim().is_empty() {
        diagnostics.outcome = "empty_input".to_string();
        write_selection_capture_diagnostics(&diagnostics);
        anyhow::bail!(
            "No selected text found and clipboard text is empty or unavailable. Select rough text, grant Accessibility permission if prompted, or copy the prompt first."
        );
    }

    diagnostics.outcome = format!("clipboard_fallback:{} chars", prompt.chars().count());
    write_selection_capture_diagnostics(&diagnostics);

    Ok(QuickEnhanceInput {
        prompt,
        source: QuickEnhanceInputSource::Clipboard,
        original_clipboard,
        accessibility_untrusted,
    })
}

async fn deliver_quick_enhance_output(
    app: &AppHandle,
    enhanced_prompt: &str,
    captured: &QuickEnhanceInput,
    initial_environment: Option<&EnvironmentSnapshot>,
) -> anyhow::Result<(QuickEnhanceDelivery, Option<&'static str>)> {
    if captured.source == QuickEnhanceInputSource::SelectedText {
        if !active_app_matches_current(initial_environment) {
            app.clipboard().write_text(enhanced_prompt)?;
            return Ok((
                QuickEnhanceDelivery::CopiedToClipboard,
                Some("the focused app or window changed"),
            ));
        }

        if replace_selected_text(app, enhanced_prompt, captured.original_clipboard.as_ref()).await {
            return Ok((QuickEnhanceDelivery::ReplacedSelection, None));
        }

        app.clipboard().write_text(enhanced_prompt)?;
        return Ok((
            QuickEnhanceDelivery::CopiedToClipboard,
            Some("pasting back into the app failed"),
        ));
    }

    app.clipboard().write_text(enhanced_prompt)?;
    Ok((QuickEnhanceDelivery::CopiedToClipboard, None))
}

fn quick_enhance_success_message(outcome: &QuickEnhanceOutcome) -> String {
    let action = match (outcome.input_source, outcome.delivery) {
        (QuickEnhanceInputSource::SelectedText, QuickEnhanceDelivery::ReplacedSelection) => {
            "Replaced selection"
        }
        (QuickEnhanceInputSource::SelectedText, QuickEnhanceDelivery::CopiedToClipboard) => {
            "Copied enhanced selection"
        }
        (QuickEnhanceInputSource::Clipboard, QuickEnhanceDelivery::CopiedToClipboard) => {
            "Copied enhanced prompt"
        }
        (QuickEnhanceInputSource::Clipboard, QuickEnhanceDelivery::ReplacedSelection) => {
            "Enhanced prompt"
        }
    };

    let mut message = format!(
        "{} for {} — {}.",
        action,
        outcome.resolution.platform.label(),
        outcome.resolution.reason
    );
    if let Some(note) = outcome.delivery_note {
        message.push_str(&format!(
            " Couldn't replace the selection: {}. Paste to use it.",
            note
        ));
    }
    message
}

#[cfg(target_os = "macos")]
fn capture_selected_text_via_accessibility(
    diagnostics: &mut SelectionCaptureDiagnostics,
) -> Option<String> {
    let trusted = ax_is_process_trusted();
    diagnostics.step(format!("AX trusted: {}", trusted));
    if !trusted {
        let prompted = request_accessibility_permission_prompt();
        diagnostics.step(format!("AX permission prompt requested: {}", prompted));
        return None;
    }
    let focused = match focused_ax_element(diagnostics) {
        Ok(focused) => focused,
        Err(e) => {
            diagnostics.step(format!("AX focused element failed: {}", e));
            return None;
        }
    };

    match copy_ax_string_attribute(focused.as_ref(), AX_ROLE) {
        Ok(Some(role)) => diagnostics.step(format!("AX focused role: {}", role)),
        Ok(None) => diagnostics.step("AX focused role unavailable"),
        Err(e) => diagnostics.step(format!("AX focused role failed: {}", e)),
    }

    match copy_ax_string_attribute(focused.as_ref(), AX_SELECTED_TEXT) {
        Ok(Some(selected_text)) if !selected_text.trim().is_empty() => {
            diagnostics.step(format!(
                "AXSelectedText captured: {} chars",
                selected_text.chars().count()
            ));
            Some(selected_text)
        }
        Ok(Some(_)) => {
            diagnostics.step("AXSelectedText returned empty text");
            None
        }
        Ok(None) => {
            diagnostics.step("AXSelectedText unavailable");
            None
        }
        Err(e) => {
            diagnostics.step(format!("AXSelectedText failed: {}", e));
            None
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn capture_selected_text_via_accessibility(
    diagnostics: &mut SelectionCaptureDiagnostics,
) -> Option<String> {
    diagnostics.step("AX capture unavailable on this OS");
    None
}

#[cfg(target_os = "macos")]
async fn capture_selected_text_via_clipboard(
    app: &AppHandle,
    original_clipboard: Option<&String>,
    diagnostics: &mut SelectionCaptureDiagnostics,
) -> Option<String> {
    diagnostics.step("clipboard fallback: starting sentinel copy probe");
    let sentinel = selection_clipboard_sentinel();
    if let Err(e) = app.clipboard().write_text(&sentinel) {
        diagnostics.step(format!("clipboard fallback: sentinel write failed: {}", e));
        return None;
    }

    tokio::time::sleep(std::time::Duration::from_millis(40)).await;

    if let Err(e) = send_system_edit_command(SystemEditCommand::Copy) {
        diagnostics.step(format!("clipboard fallback: copy command failed: {}", e));
        restore_clipboard_text(app, original_clipboard);
        return None;
    }

    for _ in 0..20 {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let current = match app.clipboard().read_text() {
            Ok(current) => current,
            Err(e) => {
                diagnostics.step(format!("clipboard fallback: read failed: {}", e));
                restore_clipboard_text(app, original_clipboard);
                return None;
            }
        };
        if current != sentinel {
            diagnostics.step(format!(
                "clipboard fallback: clipboard changed after copy: {} chars",
                current.chars().count()
            ));
            restore_clipboard_text(app, original_clipboard);
            return (!current.trim().is_empty()).then_some(current);
        }
    }

    diagnostics.step("clipboard fallback: clipboard did not change after copy command");
    restore_clipboard_text(app, original_clipboard);
    None
}

#[cfg(not(target_os = "macos"))]
async fn capture_selected_text_via_clipboard(
    _app: &AppHandle,
    _original_clipboard: Option<&String>,
    diagnostics: &mut SelectionCaptureDiagnostics,
) -> Option<String> {
    diagnostics.step("clipboard fallback capture unavailable on this OS");
    None
}

#[cfg(target_os = "macos")]
async fn replace_selected_text(
    app: &AppHandle,
    enhanced_prompt: &str,
    original_clipboard: Option<&String>,
) -> bool {
    if app.clipboard().write_text(enhanced_prompt).is_err() {
        return false;
    }

    if send_system_edit_command(SystemEditCommand::Paste).is_err() {
        return false;
    }

    tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    restore_clipboard_text(app, original_clipboard);
    true
}

#[cfg(not(target_os = "macos"))]
async fn replace_selected_text(
    _app: &AppHandle,
    _enhanced_prompt: &str,
    _original_clipboard: Option<&String>,
) -> bool {
    false
}

#[cfg(target_os = "macos")]
const AX_FOCUSED_APPLICATION: &str = "AXFocusedApplication";
#[cfg(target_os = "macos")]
const AX_FOCUSED_UI_ELEMENT: &str = "AXFocusedUIElement";
#[cfg(target_os = "macos")]
const AX_ROLE: &str = "AXRole";
#[cfg(target_os = "macos")]
const AX_SELECTED_TEXT: &str = "AXSelectedText";
#[cfg(target_os = "macos")]
const AX_ERROR_SUCCESS: i32 = 0;

#[cfg(target_os = "macos")]
type AXUIElementRef = *const std::ffi::c_void;
#[cfg(target_os = "macos")]
type AXError = i32;

#[cfg(target_os = "macos")]
struct AxElement(AXUIElementRef);

#[cfg(target_os = "macos")]
impl AxElement {
    fn as_ref(&self) -> AXUIElementRef {
        self.0
    }
}

#[cfg(target_os = "macos")]
impl Drop for AxElement {
    fn drop(&mut self) {
        unsafe { CFRelease(self.0.cast()) }
    }
}

#[cfg(target_os = "macos")]
fn focused_ax_element(diagnostics: &mut SelectionCaptureDiagnostics) -> anyhow::Result<AxElement> {
    let system = unsafe { AXUIElementCreateSystemWide() };
    if system.is_null() {
        anyhow::bail!("could not create system-wide accessibility element");
    }
    let system = AxElement(system);

    match copy_ax_element_attribute(system.as_ref(), AX_FOCUSED_APPLICATION) {
        Ok(focused_app) => {
            diagnostics.step("AX focused application captured");
            match copy_ax_element_attribute(focused_app.as_ref(), AX_FOCUSED_UI_ELEMENT) {
                Ok(focused_element) => {
                    diagnostics.step("AX focused UI element captured from focused application");
                    return Ok(focused_element);
                }
                Err(e) => diagnostics.step(format!(
                    "AX focused UI element from focused app failed: {}",
                    e
                )),
            }
        }
        Err(e) => diagnostics.step(format!("AX focused application failed: {}", e)),
    }

    let focused = copy_ax_element_attribute(system.as_ref(), AX_FOCUSED_UI_ELEMENT)?;
    diagnostics.step("AX focused UI element captured from system-wide element");
    Ok(focused)
}

#[cfg(target_os = "macos")]
fn copy_ax_element_attribute(
    element: AXUIElementRef,
    attribute: &str,
) -> anyhow::Result<AxElement> {
    let attribute_name = attribute;
    let attribute = CFString::new(attribute_name);
    let mut value: CFTypeRef = std::ptr::null();
    let error = unsafe {
        AXUIElementCopyAttributeValue(element, attribute.as_concrete_TypeRef(), &mut value)
    };
    if error != AX_ERROR_SUCCESS || value.is_null() {
        anyhow::bail!(
            "AX attribute '{}' copy failed with error {}",
            attribute_name,
            error
        );
    }
    Ok(AxElement(value.cast()))
}

#[cfg(target_os = "macos")]
fn copy_ax_string_attribute(
    element: AXUIElementRef,
    attribute: &str,
) -> anyhow::Result<Option<String>> {
    let attribute_name = attribute;
    let attribute = CFString::new(attribute_name);
    let mut value: CFTypeRef = std::ptr::null();
    let error = unsafe {
        AXUIElementCopyAttributeValue(element, attribute.as_concrete_TypeRef(), &mut value)
    };
    if error != AX_ERROR_SUCCESS || value.is_null() {
        anyhow::bail!(
            "AX attribute '{}' copy failed with error {}",
            attribute_name,
            error
        );
    }

    let value = unsafe { CFString::wrap_under_create_rule(value as CFStringRef) };
    Ok(Some(value.to_string()))
}

#[cfg(target_os = "macos")]
#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    static kAXTrustedCheckOptionPrompt: CFStringRef;

    fn AXIsProcessTrusted() -> bool;
    fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> bool;
    fn AXUIElementCreateSystemWide() -> AXUIElementRef;
    fn AXUIElementCopyAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRef,
        value: *mut CFTypeRef,
    ) -> AXError;
}

#[cfg(target_os = "macos")]
fn ax_is_process_trusted() -> bool {
    unsafe { AXIsProcessTrusted() }
}

#[cfg(target_os = "macos")]
fn request_accessibility_permission_prompt() -> bool {
    let prompt_key = unsafe { CFString::wrap_under_get_rule(kAXTrustedCheckOptionPrompt) };
    let prompt_value = CFBoolean::true_value();
    let options = CFDictionary::from_CFType_pairs(&[(prompt_key, prompt_value)]);
    unsafe { AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef()) }
}

/// `Some(trusted)` on macOS, `None` where Accessibility does not apply.
fn accessibility_trusted() -> Option<bool> {
    #[cfg(target_os = "macos")]
    {
        Some(ax_is_process_trusted())
    }
    #[cfg(not(target_os = "macos"))]
    {
        None
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessibilityStatus {
    pub platform_supported: bool,
    pub accessibility_trusted: Option<bool>,
    pub selected_text_enabled: bool,
    pub diagnostics_path: String,
    pub last_capture: Option<SelectionCaptureDiagnostics>,
}

#[tauri::command]
pub fn get_accessibility_status() -> Result<AccessibilityStatus, String> {
    let config = cfg::load_config().map_err(|e| e.to_string())?;
    let path = cfg::config_dir()
        .map_err(|e| e.to_string())?
        .join("selection-diagnostics.json");
    let last_capture = fs::read_to_string(&path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok());

    Ok(AccessibilityStatus {
        platform_supported: cfg!(target_os = "macos"),
        accessibility_trusted: accessibility_trusted(),
        selected_text_enabled: config.quick_enhance.selected_text_enabled,
        diagnostics_path: path.display().to_string(),
        last_capture,
    })
}

#[tauri::command]
pub fn open_accessibility_settings() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let status = Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
            .status()
            .map_err(|e| e.to_string())?;
        if status.success() {
            Ok(())
        } else {
            Err("Could not open System Settings".to_string())
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err("Accessibility settings are only available on macOS".to_string())
    }
}

fn write_selection_capture_diagnostics(diagnostics: &SelectionCaptureDiagnostics) {
    let Ok(dir) = cfg::config_dir() else {
        return;
    };
    let _ = fs::create_dir_all(&dir);
    let path = dir.join("selection-diagnostics.json");
    if let Ok(content) = serde_json::to_string_pretty(diagnostics)
        && fs::write(&path, content).is_ok()
    {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
        }
    }
}

fn now_ms() -> u64 {
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    millis.min(u128::from(u64::MAX)) as u64
}

#[cfg(target_os = "macos")]
#[derive(Debug, Clone, Copy)]
enum SystemEditCommand {
    Copy,
    Paste,
}

#[cfg(target_os = "macos")]
impl SystemEditCommand {
    fn menu_item(&self) -> &'static str {
        match self {
            SystemEditCommand::Copy => "Copy",
            SystemEditCommand::Paste => "Paste",
        }
    }

    fn key_code(&self) -> u8 {
        match self {
            // ANSI C / V key codes. Prefer menu items first because physical
            // hotkey modifiers can leak into synthetic keystrokes.
            SystemEditCommand::Copy => 8,
            SystemEditCommand::Paste => 9,
        }
    }
}

#[cfg(target_os = "macos")]
fn send_system_edit_command(command: SystemEditCommand) -> anyhow::Result<()> {
    send_quartz_command_key_code(command).or_else(|quartz_error| {
        send_system_edit_menu_item(command).or_else(|menu_error| {
            send_osascript_command_key_code(command).map_err(|key_error| {
                anyhow::anyhow!(
                    "CoreGraphics event failed: {}; Edit menu failed: {}; osascript key fallback failed: {}",
                    quartz_error,
                    menu_error,
                    key_error
                )
            })
        })
    })
}

#[cfg(target_os = "macos")]
fn send_quartz_command_key_code(command: SystemEditCommand) -> anyhow::Result<()> {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| anyhow::anyhow!("could not create CGEventSource"))?;
    let key_down = CGEvent::new_keyboard_event(source.clone(), command.key_code().into(), true)
        .map_err(|_| anyhow::anyhow!("could not create key-down event"))?;
    key_down.set_flags(CGEventFlags::CGEventFlagCommand);
    key_down.post(CGEventTapLocation::HID);

    std::thread::sleep(std::time::Duration::from_millis(20));

    let key_up = CGEvent::new_keyboard_event(source, command.key_code().into(), false)
        .map_err(|_| anyhow::anyhow!("could not create key-up event"))?;
    key_up.set_flags(CGEventFlags::CGEventFlagCommand);
    key_up.post(CGEventTapLocation::HID);

    Ok(())
}

#[cfg(target_os = "macos")]
fn send_system_edit_menu_item(command: SystemEditCommand) -> anyhow::Result<()> {
    let script = format!(
        r#"
        tell application "System Events"
            set frontApp to first application process whose frontmost is true
            tell frontApp
                click menu item "{}" of menu "Edit" of menu bar 1
            end tell
        end tell
        "#,
        command.menu_item()
    );
    run_osascript(&script)
        .map_err(|e| anyhow::anyhow!("{} menu item failed: {}", command.menu_item(), e))
}

#[cfg(target_os = "macos")]
fn send_osascript_command_key_code(command: SystemEditCommand) -> anyhow::Result<()> {
    let script = format!(
        r#"tell application "System Events" to key code {} using command down"#,
        command.key_code()
    );
    run_osascript(&script)
        .map_err(|e| anyhow::anyhow!("key code {} failed: {}", command.key_code(), e))
}

#[cfg(target_os = "macos")]
fn run_osascript(script: &str) -> anyhow::Result<()> {
    let output = Command::new("osascript").arg("-e").arg(script).output()?;
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    anyhow::bail!("{}{}", stderr, stdout);
}

#[cfg(target_os = "macos")]
fn restore_clipboard_text(app: &AppHandle, original_clipboard: Option<&String>) {
    if let Some(original_clipboard) = original_clipboard {
        let _ = app.clipboard().write_text(original_clipboard);
    }
}

fn active_app_matches_current(initial_environment: Option<&EnvironmentSnapshot>) -> bool {
    let Some(initial_environment) = initial_environment else {
        return false;
    };
    let Some(initial_app) = initial_environment.active_app.as_ref() else {
        return false;
    };
    let Some((current_app, current_window_title)) = collect_active_app() else {
        return false;
    };

    same_active_app(initial_app, &current_app)
        && same_window_title_when_initial_available(
            initial_environment.window_title.as_deref(),
            current_window_title.as_deref(),
        )
}

fn same_active_app(left: &ActiveApp, right: &ActiveApp) -> bool {
    left.name == right.name && left.bundle_id == right.bundle_id
}

fn same_window_title_when_initial_available(
    initial_window_title: Option<&str>,
    current_window_title: Option<&str>,
) -> bool {
    let Some(initial_window_title) = initial_window_title
        .map(str::trim)
        .filter(|title| !title.is_empty())
    else {
        return true;
    };

    current_window_title
        .map(str::trim)
        .filter(|title| !title.is_empty())
        .is_some_and(|current_window_title| current_window_title == initial_window_title)
}

#[cfg(target_os = "macos")]
fn selection_clipboard_sentinel() -> String {
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!(
        "__PROOMPT_SELECTION_CAPTURE_{}_{}__",
        std::process::id(),
        millis
    )
}

#[tauri::command]
pub fn inspect_quick_enhance_route(app: AppHandle) -> Result<QuickEnhanceRouteInspection, String> {
    let config = cfg::load_config().map_err(|e| e.to_string())?;
    let environment = collect_environment_snapshot();
    let prompt = match app.clipboard().read_text() {
        Ok(prompt) => prompt,
        Err(e) => {
            return Ok(QuickEnhanceRouteInspection {
                prompt_preview: None,
                environment,
                resolution: None,
                error: Some(format!("Could not read clipboard: {}", e)),
            });
        }
    };

    let prompt = prompt.trim();
    if prompt.is_empty() {
        return Ok(QuickEnhanceRouteInspection {
            prompt_preview: None,
            environment,
            resolution: None,
            error: Some("Clipboard is empty. Copy a rough prompt to preview routing.".to_string()),
        });
    }

    let prompt_preview = truncate_chars(prompt, 120);
    match resolve_quick_enhance_input_with_environment(&config, prompt, environment.as_ref()) {
        Ok(resolved) => Ok(QuickEnhanceRouteInspection {
            prompt_preview: Some(prompt_preview),
            environment,
            resolution: Some(resolved.resolution),
            error: None,
        }),
        Err(e) => Ok(QuickEnhanceRouteInspection {
            prompt_preview: Some(prompt_preview),
            environment,
            resolution: None,
            error: Some(e.to_string()),
        }),
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

fn truncate_chars(s: &str, max: usize) -> String {
    match s.char_indices().nth(max) {
        Some((idx, _)) => format!("{}...", &s[..idx]),
        None => s.to_string(),
    }
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
    if lower.contains("quick enhance is already running") {
        "Already enhancing. Wait for the current Quick Enhance to finish.".to_string()
    } else if lower.contains("api key not configured")
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

    let (api_key_configured, api_key_status, api_key_error) = match config.mode {
        Mode::Byok if cfg::get_api_key_from_env(&provider).is_some() => {
            (true, "env_configured".to_string(), None)
        }
        Mode::Byok => (
            false,
            "deferred".to_string(),
            Some(
                "Keychain lookup is deferred until Enhance, Quick Enhance, or Test to avoid macOS prompts on app launch."
                    .to_string(),
            ),
        ),
        Mode::Hosted => (
            false,
            "hosted_unavailable".to_string(),
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
        api_key_status,
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
    config.quick_enhance.selected_text_enabled = input.selected_text_enabled;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_window_title_allows_match_when_initial_title_unavailable() {
        assert!(same_window_title_when_initial_available(
            None,
            Some("Other window")
        ));
    }

    #[test]
    fn same_window_title_requires_current_title_to_match_when_initial_available() {
        assert!(same_window_title_when_initial_available(
            Some("Editor — project"),
            Some("Editor — project")
        ));
        assert!(!same_window_title_when_initial_available(
            Some("Editor — project"),
            Some("Different document")
        ));
        assert!(!same_window_title_when_initial_available(
            Some("Editor — project"),
            None
        ));
    }
}
