use anyhow::{Context, Result};
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{LlmRequest, LlmResponse, LlmUsage};

const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";

pub struct OpenAIClient {
    client: Client,
    api_key: String,
    model: String,
}

impl OpenAIClient {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model: model.unwrap_or_else(|| "gpt-4o".to_string()),
        }
    }

    pub async fn complete(&self, request: LlmRequest) -> Result<LlmResponse> {
        let body = OpenAIRequest {
            model: self.model.clone(),
            max_tokens: Some(request.max_tokens),
            stream: false,
            messages: vec![
                OpenAIMessage {
                    role: "system".to_string(),
                    content: request.system_prompt,
                },
                OpenAIMessage {
                    role: "user".to_string(),
                    content: request.user_prompt,
                },
            ],
        };

        let response = self
            .client
            .post(OPENAI_API_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to send request to OpenAI")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".to_string());
            anyhow::bail!("OpenAI API error ({}): {}", status, body);
        }

        let api_response: OpenAIResponse = response
            .json()
            .await
            .context("Failed to parse OpenAI response")?;

        let content = api_response
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_default();

        Ok(LlmResponse {
            content,
            usage: api_response.usage.map(|u| LlmUsage {
                input_tokens: u.prompt_tokens,
                output_tokens: u.completion_tokens,
            }),
        })
    }

    /// Stream the response token-by-token, calling `on_token` for each chunk.
    pub async fn stream(
        &self,
        request: LlmRequest,
        mut on_token: impl FnMut(&str),
    ) -> Result<LlmResponse> {
        let body = OpenAIRequest {
            model: self.model.clone(),
            max_tokens: Some(request.max_tokens),
            stream: true,
            messages: vec![
                OpenAIMessage {
                    role: "system".to_string(),
                    content: request.system_prompt,
                },
                OpenAIMessage {
                    role: "user".to_string(),
                    content: request.user_prompt,
                },
            ],
        };

        let response = self
            .client
            .post(OPENAI_API_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to send request to OpenAI")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".to_string());
            anyhow::bail!("OpenAI API error ({}): {}", status, body);
        }

        let mut full_content = String::new();
        let mut stream = response.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Stream error")?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            // Process complete SSE lines
            while let Some(line_end) = buffer.find('\n') {
                let line = buffer[..line_end].trim().to_string();
                buffer = buffer[line_end + 1..].to_string();

                if line.is_empty() || line == "data: [DONE]" {
                    continue;
                }

                if let Some(json_str) = line.strip_prefix("data: ") {
                    if let Ok(chunk) = serde_json::from_str::<StreamChunk>(json_str) {
                        if let Some(choice) = chunk.choices.first() {
                            if let Some(content) = &choice.delta.content {
                                on_token(content);
                                full_content.push_str(content);
                            }
                        }
                    }
                }
            }
        }

        Ok(LlmResponse {
            content: full_content,
            usage: None,
        })
    }
}

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    stream: bool,
    messages: Vec<OpenAIMessage>,
}

#[derive(Debug, Serialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIResponseMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponseMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
}

// Streaming types
#[derive(Debug, Deserialize)]
struct StreamChunk {
    choices: Vec<StreamChoice>,
}

#[derive(Debug, Deserialize)]
struct StreamChoice {
    delta: StreamDelta,
}

#[derive(Debug, Deserialize)]
struct StreamDelta {
    content: Option<String>,
}
