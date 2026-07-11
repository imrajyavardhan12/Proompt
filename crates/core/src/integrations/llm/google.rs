use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{
    LlmRequest, LlmResponse, LlmUsage, ensure_provider_success, provider_response_error,
    send_provider_request,
};

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
                temperature: request.temperature,
            }),
        };

        let response = send_provider_request(
            self.client
                .post(&url)
                .header("Content-Type", "application/json")
                .json(&body),
            "Gemini",
        )
        .await?;

        let response = ensure_provider_success(response, "Gemini").await?;

        let api_response: GeminiResponse = response
            .json()
            .await
            .map_err(|error| provider_response_error("Gemini", "parse response from", error))?;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
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
