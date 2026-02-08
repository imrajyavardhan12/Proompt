use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

const SUPERMEMORY_API_URL: &str = "https://api.supermemory.ai/v1";

pub struct SuperMemoryClient {
    client: Client,
    api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub content: String,
    pub score: f32,
}

impl SuperMemoryClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    pub async fn search(&self, query: &str, limit: u32) -> Result<Vec<Memory>> {
        let body = serde_json::json!({
            "query": query,
            "limit": limit,
        });

        let response = self
            .client
            .post(format!("{}/search", SUPERMEMORY_API_URL))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await
            .context("Failed to query SuperMemory")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".to_string());
            anyhow::bail!("SuperMemory API error ({}): {}", status, body);
        }

        let result: SearchResponse = response
            .json()
            .await
            .context("Failed to parse SuperMemory response")?;

        Ok(result.memories)
    }

    pub async fn test_connection(&self) -> Result<bool> {
        let result = self.search("test", 1).await;
        Ok(result.is_ok())
    }
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    memories: Vec<Memory>,
}
