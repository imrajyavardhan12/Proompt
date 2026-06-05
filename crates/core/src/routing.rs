use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{config::Config, platform::Platform};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvironmentSnapshot {
    pub active_app: Option<ActiveApp>,
    pub window_title: Option<String>,
    pub browser_context: Option<BrowserContext>,
    pub terminal_context: Option<TerminalContext>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveApp {
    pub name: String,
    pub bundle_id: Option<String>,
}

impl ActiveApp {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            bundle_id: None,
        }
    }

    pub fn with_bundle_id(mut self, bundle_id: impl Into<String>) -> Self {
        self.bundle_id = Some(bundle_id.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserContext {
    pub url: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalContext {
    pub shell_name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionSource {
    ExplicitPrefix,
    ActiveApp,
    BrowserContext,
    TerminalDefault,
    ConfigDefault,
}

impl ResolutionSource {
    pub fn label(&self) -> &'static str {
        match self {
            ResolutionSource::ExplicitPrefix => "Explicit prefix",
            ResolutionSource::ActiveApp => "Active app",
            ResolutionSource::BrowserContext => "Browser context",
            ResolutionSource::TerminalDefault => "Terminal default",
            ResolutionSource::ConfigDefault => "Quick Enhance fallback",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionConfidence {
    Explicit,
    High,
    Medium,
    Fallback,
}

impl ResolutionConfidence {
    pub fn label(&self) -> &'static str {
        match self {
            ResolutionConfidence::Explicit => "Explicit",
            ResolutionConfidence::High => "High",
            ResolutionConfidence::Medium => "Medium",
            ResolutionConfidence::Fallback => "Fallback",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetResolution {
    pub platform: Platform,
    pub source: ResolutionSource,
    pub confidence: ResolutionConfidence,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedQuickEnhanceInput {
    pub prompt: String,
    pub resolution: TargetResolution,
}

pub fn resolve_quick_enhance_input(
    config: &Config,
    input: &str,
) -> Result<ResolvedQuickEnhanceInput> {
    resolve_quick_enhance_input_with_environment(config, input, None)
}

pub fn resolve_quick_enhance_input_with_environment(
    config: &Config,
    input: &str,
    environment: Option<&EnvironmentSnapshot>,
) -> Result<ResolvedQuickEnhanceInput> {
    let prompt = input.trim();
    if prompt.is_empty() {
        anyhow::bail!("Prompt cannot be empty");
    }

    if let Some(route) = parse_explicit_prefix(prompt) {
        let routed_prompt = route.rest.trim();
        if routed_prompt.is_empty() {
            anyhow::bail!("Prompt cannot be empty after routing prefix");
        }

        return Ok(ResolvedQuickEnhanceInput {
            prompt: routed_prompt.to_string(),
            resolution: TargetResolution {
                platform: route.platform,
                source: ResolutionSource::ExplicitPrefix,
                confidence: ResolutionConfidence::Explicit,
                reason: format!("via {}", route.prefix),
            },
        });
    }

    if config.quick_enhance.auto_detect_target
        && let Some(environment) = environment
        && let Some(resolution) = resolve_from_environment(config, environment)
    {
        return Ok(ResolvedQuickEnhanceInput {
            prompt: prompt.to_string(),
            resolution,
        });
    }

    Ok(ResolvedQuickEnhanceInput {
        prompt: prompt.to_string(),
        resolution: default_resolution(config),
    })
}

fn resolve_from_environment(
    config: &Config,
    environment: &EnvironmentSnapshot,
) -> Option<TargetResolution> {
    let active_app = environment.active_app.as_ref();

    if let Some(app) = active_app {
        if let Some(platform) = platform_for_direct_app(app) {
            return Some(TargetResolution {
                platform,
                source: ResolutionSource::ActiveApp,
                confidence: ResolutionConfidence::High,
                reason: format!("from {} active app", app.name),
            });
        }

        if is_terminal_app(app) {
            return terminal_resolution(config);
        }

        if is_browser_app(app)
            && let Some(platform) = platform_for_browser_context(environment)
        {
            return Some(TargetResolution {
                platform,
                source: ResolutionSource::BrowserContext,
                confidence: ResolutionConfidence::Medium,
                reason: format!("from {} browser context", app.name),
            });
        }
    }

    None
}

fn terminal_resolution(config: &Config) -> Option<TargetResolution> {
    let platform = config.quick_enhance.terminal_platform?;
    if !platform.is_text_platform() {
        return None;
    }

    Some(TargetResolution {
        platform,
        source: ResolutionSource::TerminalDefault,
        confidence: ResolutionConfidence::Medium,
        reason: "using Terminal default".to_string(),
    })
}

fn default_resolution(config: &Config) -> TargetResolution {
    let platform = if config.default_platform.is_text_platform() {
        config.default_platform
    } else {
        Platform::Generic
    };

    TargetResolution {
        platform,
        source: ResolutionSource::ConfigDefault,
        confidence: ResolutionConfidence::Fallback,
        reason: "using Quick Enhance fallback target".to_string(),
    }
}

fn platform_for_direct_app(app: &ActiveApp) -> Option<Platform> {
    let haystack = app_identity(app);

    if haystack.contains("cursor") {
        Some(Platform::Cursor)
    } else if haystack.contains("chatgpt") || haystack.contains("openai") {
        Some(Platform::OpenAI)
    } else if haystack.contains("claude") || haystack.contains("anthropic") {
        Some(Platform::Claude)
    } else {
        None
    }
}

fn platform_for_browser_context(environment: &EnvironmentSnapshot) -> Option<Platform> {
    let mut haystack = String::new();
    if let Some(browser) = &environment.browser_context {
        if let Some(url) = &browser.url {
            haystack.push_str(url);
            haystack.push(' ');
        }
        if let Some(title) = &browser.title {
            haystack.push_str(title);
            haystack.push(' ');
        }
    }
    if let Some(title) = &environment.window_title {
        haystack.push_str(title);
    }
    let haystack = haystack.to_ascii_lowercase();

    if haystack.contains("chatgpt") || haystack.contains("chat.openai.com") {
        Some(Platform::OpenAI)
    } else if haystack.contains("claude.ai") || haystack.contains("claude") {
        Some(Platform::Claude)
    } else if haystack.contains("gemini.google.com")
        || haystack.contains("aistudio.google.com")
        || haystack.contains("gemini")
    {
        Some(Platform::Gemini)
    } else {
        None
    }
}

fn is_terminal_app(app: &ActiveApp) -> bool {
    let haystack = app_identity(app);
    [
        "terminal",
        "iterm",
        "ghostty",
        "warp",
        "alacritty",
        "kitty",
        "wezterm",
        "hyper",
        "tabby",
    ]
    .iter()
    .any(|needle| haystack.contains(needle))
}

fn is_browser_app(app: &ActiveApp) -> bool {
    let haystack = app_identity(app);
    [
        "safari", "chrome", "chromium", "firefox", "arc", "brave", "edge", "dia", "browser",
    ]
    .iter()
    .any(|needle| haystack.contains(needle))
}

fn app_identity(app: &ActiveApp) -> String {
    format!(
        "{} {}",
        app.name,
        app.bundle_id.as_deref().unwrap_or_default()
    )
    .to_ascii_lowercase()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PrefixRoute<'a> {
    prefix: &'a str,
    platform: Platform,
    rest: &'a str,
}

fn parse_explicit_prefix(input: &str) -> Option<PrefixRoute<'_>> {
    let without_slash = input.strip_prefix('/')?;
    let split_at = without_slash.find(char::is_whitespace);
    let (prefix_body, rest) = match split_at {
        Some(index) => (&without_slash[..index], without_slash[index..].trim_start()),
        None => (without_slash, ""),
    };
    let platform = platform_for_prefix(prefix_body)?;
    let prefix = &input[..prefix_body.len() + 1];

    Some(PrefixRoute {
        prefix,
        platform,
        rest,
    })
}

fn platform_for_prefix(prefix: &str) -> Option<Platform> {
    Some(match prefix.trim().to_ascii_lowercase().as_str() {
        "cc" | "claude-code" | "claudecode" => Platform::ClaudeCode,
        "cursor" => Platform::Cursor,
        "codex" | "openai-codex" => Platform::Codex,
        "agent" | "coding-agent" | "generic-agent" => Platform::CodingAgent,
        "gpt" | "openai" | "chatgpt" => Platform::OpenAI,
        "claude" => Platform::Claude,
        "gemini" | "google" => Platform::Gemini,
        "generic" => Platform::Generic,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_default_quick_enhance_target_without_prefix() {
        let config = Config {
            default_platform: Platform::Cursor,
            ..Default::default()
        };

        let resolved = resolve_quick_enhance_input(&config, "  fix upload bug  ").unwrap();

        assert_eq!(resolved.prompt, "fix upload bug");
        assert_eq!(resolved.resolution.platform, Platform::Cursor);
        assert_eq!(resolved.resolution.source, ResolutionSource::ConfigDefault);
        assert_eq!(
            resolved.resolution.confidence,
            ResolutionConfidence::Fallback
        );
    }

    #[test]
    fn explicit_prefix_overrides_default_and_is_stripped() {
        let config = Config {
            default_platform: Platform::Claude,
            ..Default::default()
        };
        let environment = EnvironmentSnapshot {
            active_app: Some(ActiveApp::new("Cursor")),
            ..Default::default()
        };

        let resolved = resolve_quick_enhance_input_with_environment(
            &config,
            "/cc fix upload bug",
            Some(&environment),
        )
        .unwrap();

        assert_eq!(resolved.prompt, "fix upload bug");
        assert_eq!(resolved.resolution.platform, Platform::ClaudeCode);
        assert_eq!(resolved.resolution.source, ResolutionSource::ExplicitPrefix);
        assert_eq!(
            resolved.resolution.confidence,
            ResolutionConfidence::Explicit
        );
        assert_eq!(resolved.resolution.reason, "via /cc");
    }

    #[test]
    fn explicit_prefix_supports_newline_separator() {
        let config = Config::default();

        let resolved =
            resolve_quick_enhance_input(&config, "/cursor\nadd auth middleware").unwrap();

        assert_eq!(resolved.prompt, "add auth middleware");
        assert_eq!(resolved.resolution.platform, Platform::Cursor);
    }

    #[test]
    fn explicit_prefix_supports_chat_assistant_targets() {
        let config = Config::default();

        let resolved =
            resolve_quick_enhance_input(&config, "/gpt explain borrow checking").unwrap();

        assert_eq!(resolved.prompt, "explain borrow checking");
        assert_eq!(resolved.resolution.platform, Platform::OpenAI);
    }

    #[test]
    fn active_cursor_app_routes_to_cursor() {
        let config = Config {
            default_platform: Platform::Claude,
            ..Default::default()
        };
        let environment = EnvironmentSnapshot {
            active_app: Some(ActiveApp::new("Cursor")),
            ..Default::default()
        };

        let resolved = resolve_quick_enhance_input_with_environment(
            &config,
            "add auth middleware",
            Some(&environment),
        )
        .unwrap();

        assert_eq!(resolved.resolution.platform, Platform::Cursor);
        assert_eq!(resolved.resolution.source, ResolutionSource::ActiveApp);
        assert_eq!(resolved.resolution.confidence, ResolutionConfidence::High);
    }

    #[test]
    fn active_chatgpt_app_routes_to_openai() {
        let config = Config::default();
        let environment = EnvironmentSnapshot {
            active_app: Some(ActiveApp::new("ChatGPT")),
            ..Default::default()
        };

        let resolved = resolve_quick_enhance_input_with_environment(
            &config,
            "make this sharper",
            Some(&environment),
        )
        .unwrap();

        assert_eq!(resolved.resolution.platform, Platform::OpenAI);
        assert_eq!(resolved.resolution.source, ResolutionSource::ActiveApp);
    }

    #[test]
    fn browser_title_routes_to_claude_when_browser_is_active() {
        let config = Config::default();
        let environment = EnvironmentSnapshot {
            active_app: Some(ActiveApp::new("Safari")),
            window_title: Some("Claude".to_string()),
            ..Default::default()
        };

        let resolved = resolve_quick_enhance_input_with_environment(
            &config,
            "explain this tradeoff",
            Some(&environment),
        )
        .unwrap();

        assert_eq!(resolved.resolution.platform, Platform::Claude);
        assert_eq!(resolved.resolution.source, ResolutionSource::BrowserContext);
    }

    #[test]
    fn terminal_app_uses_terminal_default_when_configured() {
        let config = Config {
            default_platform: Platform::Claude,
            quick_enhance: crate::config::QuickEnhanceConfig {
                terminal_platform: Some(Platform::ClaudeCode),
                ..Default::default()
            },
            ..Default::default()
        };
        let environment = EnvironmentSnapshot {
            active_app: Some(ActiveApp::new("Ghostty")),
            ..Default::default()
        };

        let resolved = resolve_quick_enhance_input_with_environment(
            &config,
            "fix upload bug",
            Some(&environment),
        )
        .unwrap();

        assert_eq!(resolved.resolution.platform, Platform::ClaudeCode);
        assert_eq!(
            resolved.resolution.source,
            ResolutionSource::TerminalDefault
        );
        assert_eq!(resolved.resolution.reason, "using Terminal default");
    }

    #[test]
    fn terminal_app_without_terminal_default_falls_back_to_quick_target() {
        let config = Config {
            default_platform: Platform::CodingAgent,
            ..Default::default()
        };
        let environment = EnvironmentSnapshot {
            active_app: Some(ActiveApp::new("Ghostty")),
            ..Default::default()
        };

        let resolved = resolve_quick_enhance_input_with_environment(
            &config,
            "fix upload bug",
            Some(&environment),
        )
        .unwrap();

        assert_eq!(resolved.resolution.platform, Platform::CodingAgent);
        assert_eq!(resolved.resolution.source, ResolutionSource::ConfigDefault);
    }

    #[test]
    fn disabled_auto_detection_uses_quick_target() {
        let mut config = Config {
            default_platform: Platform::Claude,
            ..Default::default()
        };
        config.quick_enhance.auto_detect_target = false;
        let environment = EnvironmentSnapshot {
            active_app: Some(ActiveApp::new("Cursor")),
            ..Default::default()
        };

        let resolved = resolve_quick_enhance_input_with_environment(
            &config,
            "add auth middleware",
            Some(&environment),
        )
        .unwrap();

        assert_eq!(resolved.resolution.platform, Platform::Claude);
        assert_eq!(resolved.resolution.source, ResolutionSource::ConfigDefault);
    }

    #[test]
    fn unknown_slash_prefix_is_preserved_as_prompt_text() {
        let config = Config {
            default_platform: Platform::Gemini,
            ..Default::default()
        };

        let resolved = resolve_quick_enhance_input(&config, "/api route is broken").unwrap();

        assert_eq!(resolved.prompt, "/api route is broken");
        assert_eq!(resolved.resolution.platform, Platform::Gemini);
        assert_eq!(resolved.resolution.source, ResolutionSource::ConfigDefault);
    }

    #[test]
    fn known_prefix_without_prompt_is_rejected() {
        let config = Config::default();

        let result = resolve_quick_enhance_input(&config, "/cc");

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("after routing prefix")
        );
    }
}
