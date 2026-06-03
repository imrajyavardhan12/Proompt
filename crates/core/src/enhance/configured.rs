use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::{EnhanceOptions, EnhanceRequest, EnhanceResponse, enhance, enhance_stream};
use crate::{
    config::{self as cfg, Config, Mode},
    integrations::supermemory::SuperMemoryClient,
    platform::{EnhanceType, Platform, parse_platform},
};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfiguredEnhanceRequest {
    pub prompt: String,
    /// Explicit platform string from CLI/UI. When omitted, config defaults are used.
    pub platform: Option<String>,
    /// Explicit enhancement type. When omitted, inferred from platform/config defaults.
    pub enhancement_type: Option<EnhanceType>,
    #[serde(default)]
    pub include_memory: bool,
    pub style_hints: Option<Vec<String>>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreparedEnhancement {
    pub request: EnhanceRequest,
    pub provider: String,
    pub model: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "status", content = "detail")]
pub enum SuperMemoryStatus {
    NotRequested,
    NotApplicable,
    Disabled,
    Empty,
    Used { count: usize },
    Unavailable { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfiguredEnhanceResponse {
    pub response: EnhanceResponse,
    pub provider: String,
    pub model: String,
    pub enhancement_type: EnhanceType,
    pub memory_status: SuperMemoryStatus,
}

pub fn provider_supports_streaming(provider: &str) -> bool {
    matches!(
        cfg::normalize_provider(provider),
        Some(cfg::OPENAI_PROVIDER | cfg::OPENROUTER_PROVIDER)
    )
}

pub fn prepare_enhancement(
    config: &Config,
    input: &ConfiguredEnhanceRequest,
) -> Result<PreparedEnhancement> {
    if input.prompt.trim().is_empty() {
        anyhow::bail!("Prompt cannot be empty");
    }

    let provider = cfg::normalize_provider(&config.byok.provider)
        .ok_or_else(|| anyhow::anyhow!("Unsupported provider: {}", config.byok.provider))?
        .to_string();
    let model = if config.byok.model.trim().is_empty()
        || !cfg::model_matches_provider(&config.byok.model, &provider)
    {
        cfg::default_model_for_provider(&provider)
            .ok_or_else(|| {
                anyhow::anyhow!("Provider '{}' does not have a default model", provider)
            })?
            .to_string()
    } else {
        config.byok.model.clone()
    };

    let platform = match input.platform.as_deref() {
        Some(platform) => parse_platform(platform).ok_or_else(|| {
            anyhow::anyhow!(
                "Invalid platform '{}'. Use claude, openai, gemini, generic, midjourney, dalle, or sd.",
                platform
            )
        })?,
        None => match input.enhancement_type {
            Some(EnhanceType::Image) => config.default_image_platform,
            Some(EnhanceType::Text) | None => config.default_platform,
        },
    };

    let enhancement_type = match input.enhancement_type {
        Some(enhancement_type) => enhancement_type,
        None if platform.is_image_platform() => EnhanceType::Image,
        None => EnhanceType::Text,
    };

    validate_platform_for_type(platform, enhancement_type)?;

    let style_hints = if enhancement_type == EnhanceType::Image {
        normalize_style_hints(input.style_hints.clone())
    } else {
        None
    };

    Ok(PreparedEnhancement {
        request: EnhanceRequest {
            prompt: input.prompt.clone(),
            platform,
            enhancement_type,
            options: EnhanceOptions {
                include_supermemory: input.include_memory,
                style_hints,
                max_tokens: input.max_tokens,
            },
        },
        provider,
        model,
    })
}

pub async fn enhance_with_config(
    input: ConfiguredEnhanceRequest,
) -> Result<ConfiguredEnhanceResponse> {
    let config = cfg::load_config()?;
    enhance_with_loaded_config(input, config).await
}

pub async fn enhance_with_loaded_config(
    input: ConfiguredEnhanceRequest,
    config: Config,
) -> Result<ConfiguredEnhanceResponse> {
    let prepared = prepare_enhancement(&config, &input)?;
    let api_key = provider_api_key(&config, &prepared.provider)?;
    let (memory_context, memory_status) = fetch_memory_context(&config, &prepared).await;

    let response = enhance(
        prepared.request.clone(),
        &prepared.provider,
        &api_key,
        Some(prepared.model.clone()),
        memory_context,
    )
    .await?;

    Ok(ConfiguredEnhanceResponse {
        response,
        provider: prepared.provider,
        model: prepared.model,
        enhancement_type: prepared.request.enhancement_type,
        memory_status,
    })
}

pub async fn enhance_with_loaded_config_stream(
    input: ConfiguredEnhanceRequest,
    config: Config,
    on_token: impl FnMut(&str),
) -> Result<ConfiguredEnhanceResponse> {
    let prepared = prepare_enhancement(&config, &input)?;
    let api_key = provider_api_key(&config, &prepared.provider)?;
    let (memory_context, memory_status) = fetch_memory_context(&config, &prepared).await;

    let response = enhance_stream(
        prepared.request.clone(),
        &prepared.provider,
        &api_key,
        Some(prepared.model.clone()),
        memory_context,
        on_token,
    )
    .await?;

    Ok(ConfiguredEnhanceResponse {
        response,
        provider: prepared.provider,
        model: prepared.model,
        enhancement_type: prepared.request.enhancement_type,
        memory_status,
    })
}

fn validate_platform_for_type(platform: Platform, enhancement_type: EnhanceType) -> Result<()> {
    match enhancement_type {
        EnhanceType::Text if !platform.is_text_platform() => {
            anyhow::bail!("Text enhancement requires claude, openai, gemini, or generic platform")
        }
        EnhanceType::Image if !platform.is_image_platform() && platform != Platform::Generic => {
            anyhow::bail!("Image enhancement requires midjourney, dalle, sd, or generic platform")
        }
        _ => Ok(()),
    }
}

fn normalize_style_hints(style_hints: Option<Vec<String>>) -> Option<Vec<String>> {
    style_hints
        .map(|hints| {
            hints
                .into_iter()
                .map(|hint| hint.trim().to_string())
                .filter(|hint| !hint.is_empty())
                .collect::<Vec<_>>()
        })
        .filter(|hints| !hints.is_empty())
}

fn provider_api_key(config: &Config, provider: &str) -> Result<String> {
    match config.mode {
        Mode::Byok => cfg::get_api_key(provider).with_context(|| {
            let env_var = cfg::preferred_api_key_env_var(provider).unwrap_or("<PROVIDER>_API_KEY");
            format!(
                "API key not configured for '{}'. Run: proompt config set {}.api_key YOUR_KEY or export {}",
                provider, provider, env_var
            )
        }),
        Mode::Hosted => {
            anyhow::bail!("Hosted mode not yet implemented. Use BYOK mode with your own API key.")
        }
    }
}

async fn fetch_memory_context(
    config: &Config,
    prepared: &PreparedEnhancement,
) -> (Option<Vec<String>>, SuperMemoryStatus) {
    if !prepared.request.options.include_supermemory {
        return (None, SuperMemoryStatus::NotRequested);
    }
    if prepared.request.enhancement_type != EnhanceType::Text {
        return (None, SuperMemoryStatus::NotApplicable);
    }
    if !config.supermemory.enabled {
        return (None, SuperMemoryStatus::Disabled);
    }

    let api_key = match cfg::get_api_key("supermemory") {
        Ok(api_key) => api_key,
        Err(e) => {
            return (
                None,
                SuperMemoryStatus::Unavailable {
                    message: e.to_string(),
                },
            );
        }
    };

    let client = SuperMemoryClient::new(api_key);
    match client
        .search(&prepared.request.prompt, config.supermemory.context_limit)
        .await
    {
        Ok(memories) if memories.is_empty() => (None, SuperMemoryStatus::Empty),
        Ok(memories) => {
            let context = memories.into_iter().map(|m| m.content).collect::<Vec<_>>();
            let count = context.len();
            (Some(context), SuperMemoryStatus::Used { count })
        }
        Err(e) => (
            None,
            SuperMemoryStatus::Unavailable {
                message: e.to_string(),
            },
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepare_uses_text_default_platform() {
        let config = Config::default();
        let input = ConfiguredEnhanceRequest {
            prompt: "explain docker".to_string(),
            ..Default::default()
        };

        let prepared = prepare_enhancement(&config, &input).unwrap();

        assert_eq!(prepared.request.platform, Platform::Claude);
        assert_eq!(prepared.request.enhancement_type, EnhanceType::Text);
    }

    #[test]
    fn prepare_uses_image_default_platform() {
        let config = Config::default();
        let input = ConfiguredEnhanceRequest {
            prompt: "cat in space".to_string(),
            enhancement_type: Some(EnhanceType::Image),
            ..Default::default()
        };

        let prepared = prepare_enhancement(&config, &input).unwrap();

        assert_eq!(prepared.request.platform, Platform::Midjourney);
        assert_eq!(prepared.request.enhancement_type, EnhanceType::Image);
    }

    #[test]
    fn prepare_infers_image_type_from_image_platform() {
        let config = Config::default();
        let input = ConfiguredEnhanceRequest {
            prompt: "cat in space".to_string(),
            platform: Some("midjourney".to_string()),
            ..Default::default()
        };

        let prepared = prepare_enhancement(&config, &input).unwrap();

        assert_eq!(prepared.request.platform, Platform::Midjourney);
        assert_eq!(prepared.request.enhancement_type, EnhanceType::Image);
    }

    #[test]
    fn prepare_rejects_image_platform_for_text_mode() {
        let config = Config::default();
        let input = ConfiguredEnhanceRequest {
            prompt: "explain docker".to_string(),
            platform: Some("midjourney".to_string()),
            enhancement_type: Some(EnhanceType::Text),
            ..Default::default()
        };

        let result = prepare_enhancement(&config, &input);

        assert!(result.is_err());
    }

    #[test]
    fn prepare_accepts_generic_for_image_mode() {
        let config = Config::default();
        let input = ConfiguredEnhanceRequest {
            prompt: "cat in space".to_string(),
            platform: Some("generic".to_string()),
            enhancement_type: Some(EnhanceType::Image),
            style_hints: Some(vec![" cinematic ".to_string(), "".to_string()]),
            ..Default::default()
        };

        let prepared = prepare_enhancement(&config, &input).unwrap();

        assert_eq!(prepared.request.platform, Platform::Generic);
        assert_eq!(prepared.request.enhancement_type, EnhanceType::Image);
        assert_eq!(
            prepared.request.options.style_hints,
            Some(vec!["cinematic".to_string()])
        );
    }

    #[test]
    fn prepare_rejects_unknown_platform() {
        let config = Config::default();
        let input = ConfiguredEnhanceRequest {
            prompt: "hello".to_string(),
            platform: Some("claudee".to_string()),
            ..Default::default()
        };

        let result = prepare_enhancement(&config, &input);

        assert!(result.is_err());
    }

    #[test]
    fn provider_supports_streaming_for_openai_compatible_providers() {
        assert!(provider_supports_streaming("openai"));
        assert!(provider_supports_streaming("openrouter"));
        assert!(!provider_supports_streaming("anthropic"));
    }
}
