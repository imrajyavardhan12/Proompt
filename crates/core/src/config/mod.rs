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
pub struct PreferencesConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default)]
    pub launch_at_login: bool,
    #[serde(default)]
    pub telemetry: bool,
}

impl Default for PreferencesConfig {
    fn default() -> Self {
        Self {
            theme: "system".to_string(),
            launch_at_login: false,
            telemetry: false,
        }
    }
}

fn default_theme() -> String {
    "system".to_string()
}
