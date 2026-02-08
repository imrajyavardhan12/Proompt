use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{LlmRequest, LlmResponse, LlmUsage};

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";

pub struct AnthropicClient {
    client: Client,
    api_key: String,
    model: String,
}

impl AnthropicClient {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model: model.unwrap_or_else(|| "claude-sonnet-4-20250514".to_string()),
        }
    }

    pub async fn complete(&self, request: LlmRequest) -> Result<LlmResponse> {
        let body = AnthropicRequest {
            model: self.model.clone(),
            max_tokens: request.max_tokens,
            system: Some(request.system_prompt),
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: request.user_prompt,
            }],
        };

        let response = self
            .client
            .post(ANTHROPIC_API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to send request to Anthropic")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".to_string());
            anyhow::bail!("Anthropic API error ({}): {}", status, body);
        }

        let api_response: AnthropicResponse = response
            .json()
            .await
            .context("Failed to parse Anthropic response")?;

        let content = api_response
            .content
            .into_iter()
            .filter(|block| block.block_type == "text")
            .map(|block| block.text.unwrap_or_default())
            .collect::<Vec<_>>()
            .join("");

        Ok(LlmResponse {
            content,
            usage: api_response.usage.map(|u| LlmUsage {
                input_tokens: u.input_tokens,
                output_tokens: u.output_tokens,
            }),
        })
    }
}

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    messages: Vec<AnthropicMessage>,
}

#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<ContentBlock>,
    usage: Option<AnthropicUsage>,
}

#[derive(Debug, Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}
