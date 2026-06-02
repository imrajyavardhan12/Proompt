pub mod image;
pub mod text;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::platform::{EnhanceType, Platform};

fn build_changes_summary(
    original: &str,
    enhanced: &str,
    platform: Platform,
    used_context: bool,
) -> String {
    let mut changes = Vec::new();

    let ratio = enhanced.len() as f64 / original.len().max(1) as f64;
    if ratio > 3.0 {
        changes.push("Added significant structure and detail");
    } else if ratio > 1.5 {
        changes.push("Added structure and clarity");
    } else {
        changes.push("Refined and clarified");
    }

    changes.push(match platform {
        Platform::Claude => "Optimized for Claude (XML tags, thinking prompts)",
        Platform::OpenAI => "Optimized for GPT (role framing, markdown structure)",
        Platform::Gemini => "Optimized for Gemini (explicit formatting, grounding)",
        Platform::Midjourney => "Formatted for Midjourney (parameters, style keywords)",
        Platform::DallE => "Formatted for DALL-E (natural language, safety-aware)",
        Platform::StableDiffusion => "Formatted for SD (weighted tokens, negative prompt)",
        Platform::Generic => "Applied universal best practices",
    });

    if used_context {
        changes.push("Enriched with SuperMemory context");
    }

    changes.join(". ")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhanceRequest {
    pub prompt: String,
    pub platform: Platform,
    pub enhancement_type: EnhanceType,
    pub options: EnhanceOptions,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnhanceOptions {
    #[serde(default)]
    pub include_supermemory: bool,
    pub style_hints: Option<Vec<String>>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhanceResponse {
    pub enhanced_prompt: String,
    pub changes_summary: String,
    pub context_used: Option<Vec<String>>,
    pub platform: Platform,
}

#[derive(Debug, thiserror::Error)]
pub enum EnhanceError {
    #[error("API key not configured. Run `proompt config set` to add your API key.")]
    ApiKeyMissing,
    #[error("Invalid API key")]
    ApiKeyInvalid,
    #[error("Rate limited. Retry after {retry_after} seconds.")]
    RateLimited { retry_after: u64 },
    #[error("Provider error: {message}")]
    ProviderError { message: String },
    #[error("Network error: {message}")]
    NetworkError { message: String },
    #[error("Invalid input: {message}")]
    InvalidInput { message: String },
}

pub async fn enhance(
    request: EnhanceRequest,
    provider: &str,
    api_key: &str,
    model: Option<String>,
    supermemory_context: Option<Vec<String>>,
) -> Result<EnhanceResponse> {
    if request.prompt.trim().is_empty() {
        anyhow::bail!("Prompt cannot be empty");
    }

    let (system_prompt, user_prompt) = match request.enhancement_type {
        EnhanceType::Text => text::build_prompts(
            &request.prompt,
            request.platform,
            supermemory_context.as_deref(),
        ),
        EnhanceType::Image => image::build_prompts(
            &request.prompt,
            request.platform,
            request.options.style_hints.as_deref(),
        ),
    };

    let llm_request = crate::integrations::llm::LlmRequest {
        system_prompt,
        user_prompt,
        max_tokens: request.options.max_tokens.unwrap_or(2048),
    };

    let provider = crate::config::normalize_provider(provider)
        .ok_or_else(|| anyhow::anyhow!("Unsupported provider: {}", provider))?;

    let response = match provider {
        "openai" => {
            let client =
                crate::integrations::llm::openai::OpenAIClient::new(api_key.to_string(), model);
            client.complete(llm_request).await?
        }
        "openrouter" => {
            let client = crate::integrations::llm::openai::OpenAIClient::openrouter(
                api_key.to_string(),
                model,
            );
            client.complete(llm_request).await?
        }
        "google" => {
            let client =
                crate::integrations::llm::google::GoogleClient::new(api_key.to_string(), model);
            client.complete(llm_request).await?
        }
        "anthropic" => {
            let client = crate::integrations::llm::anthropic::AnthropicClient::new(
                api_key.to_string(),
                model,
            );
            client.complete(llm_request).await?
        }
        _ => unreachable!("provider was normalized before matching"),
    };

    let changes_summary = build_changes_summary(
        &request.prompt,
        &response.content,
        request.platform,
        supermemory_context.is_some(),
    );

    Ok(EnhanceResponse {
        enhanced_prompt: response.content,
        changes_summary,
        context_used: supermemory_context,
        platform: request.platform,
    })
}

/// Streaming enhancement - calls `on_token` for each token as it arrives.
/// OpenAI-compatible providers stream tokens; other providers fall back to batch completion.
pub async fn enhance_stream(
    request: EnhanceRequest,
    provider: &str,
    api_key: &str,
    model: Option<String>,
    supermemory_context: Option<Vec<String>>,
    on_token: impl FnMut(&str),
) -> Result<EnhanceResponse> {
    if request.prompt.trim().is_empty() {
        anyhow::bail!("Prompt cannot be empty");
    }

    let (system_prompt, user_prompt) = match request.enhancement_type {
        EnhanceType::Text => text::build_prompts(
            &request.prompt,
            request.platform,
            supermemory_context.as_deref(),
        ),
        EnhanceType::Image => image::build_prompts(
            &request.prompt,
            request.platform,
            request.options.style_hints.as_deref(),
        ),
    };

    let llm_request = crate::integrations::llm::LlmRequest {
        system_prompt,
        user_prompt,
        max_tokens: request.options.max_tokens.unwrap_or(2048),
    };

    let provider = crate::config::normalize_provider(provider)
        .ok_or_else(|| anyhow::anyhow!("Unsupported provider: {}", provider))?;

    let response = match provider {
        "openai" => {
            let client =
                crate::integrations::llm::openai::OpenAIClient::new(api_key.to_string(), model);
            client.stream(llm_request, on_token).await?
        }
        "openrouter" => {
            let client = crate::integrations::llm::openai::OpenAIClient::openrouter(
                api_key.to_string(),
                model,
            );
            client.stream(llm_request, on_token).await?
        }
        "google" => {
            let client =
                crate::integrations::llm::google::GoogleClient::new(api_key.to_string(), model);
            client.complete(llm_request).await?
        }
        "anthropic" => {
            let client = crate::integrations::llm::anthropic::AnthropicClient::new(
                api_key.to_string(),
                model,
            );
            client.complete(llm_request).await?
        }
        _ => unreachable!("provider was normalized before matching"),
    };

    let changes_summary = build_changes_summary(
        &request.prompt,
        &response.content,
        request.platform,
        supermemory_context.is_some(),
    );

    Ok(EnhanceResponse {
        enhanced_prompt: response.content,
        changes_summary,
        context_used: supermemory_context,
        platform: request.platform,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn enhance_rejects_unknown_provider_before_network_call() {
        let request = EnhanceRequest {
            prompt: "make this clearer".to_string(),
            platform: Platform::Generic,
            enhancement_type: EnhanceType::Text,
            options: EnhanceOptions::default(),
        };

        let result = enhance(request, "unknown-provider", "test-key", None, None).await;

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Unsupported provider")
        );
    }
}
