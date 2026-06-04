use anyhow::Result;

use crate::{config::Config, platform::Platform};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionSource {
    ExplicitPrefix,
    ConfigDefault,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetResolution {
    pub platform: Platform,
    pub source: ResolutionSource,
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
                reason: format!("via {}", route.prefix),
            },
        });
    }

    let platform = if config.default_platform.is_text_platform() {
        config.default_platform
    } else {
        Platform::Generic
    };

    Ok(ResolvedQuickEnhanceInput {
        prompt: prompt.to_string(),
        resolution: TargetResolution {
            platform,
            source: ResolutionSource::ConfigDefault,
            reason: "using Quick Enhance target".to_string(),
        },
    })
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
    }

    #[test]
    fn explicit_prefix_overrides_default_and_is_stripped() {
        let config = Config {
            default_platform: Platform::Claude,
            ..Default::default()
        };

        let resolved = resolve_quick_enhance_input(&config, "/cc fix upload bug").unwrap();

        assert_eq!(resolved.prompt, "fix upload bug");
        assert_eq!(resolved.resolution.platform, Platform::ClaudeCode);
        assert_eq!(resolved.resolution.source, ResolutionSource::ExplicitPrefix);
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
