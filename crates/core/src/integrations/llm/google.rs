use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{LlmRequest, LlmResponse, LlmUsage};

const GEMINI_API_URL: &str = "https://generativelanguage.googleapis.com/v1beta/models";

pub struct GoogleClient {
    client: Client,
    api_key: String,
    model: String,
}

impl GoogleClient {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model: model.unwrap_or_else(|| "gemini-2.0-flash".to_string()),
        }
    }

    pub async fn complete(&self, request: LlmRequest) -> Result<LlmResponse> {
        let url = format!(
            "{}/{}:generateContent?key={}",
            GEMINI_API_URL, self.model, self.api_key
        );

        let body = GeminiRequest {
            system_instruction: Some(GeminiContent {
                parts: vec![GeminiPart {
                    text: request.system_prompt,
                }],
            }),
            contents: vec![GeminiContent {
                parts: vec![GeminiPart {
                    text: request.user_prompt,
                }],
            }],
            generation_config: Some(GenerationConfig {
                max_output_tokens: Some(request.max_tokens),
            }),
        };

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to send request to Gemini")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".to_string());
            anyhow::bail!("Gemini API error ({}): {}", status, body);
        }

        let api_response: GeminiResponse = response
            .json()
            .await
            .context("Failed to parse Gemini response")?;

        let content = api_response
            .candidates
            .into_iter()
            .next()
            .and_then(|c| c.content.parts.into_iter().next())
            .map(|p| p.text)
            .unwrap_or_default();

        let usage = api_response.usage_metadata.map(|u| LlmUsage {
            input_tokens: u.prompt_token_count,
            output_tokens: u.candidates_token_count,
        });

        Ok(LlmResponse { content, usage })
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GeminiRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GeminiContent>,
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GenerationConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiPart {
    text: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiResponse {
    #[serde(default)]
    candidates: Vec<GeminiCandidate>,
    usage_metadata: Option<GeminiUsageMetadata>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiUsageMetadata {
    prompt_token_count: u32,
    candidates_token_count: u32,
}
