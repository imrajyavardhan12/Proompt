pub mod anthropic;
pub mod google;
pub mod openai;

use std::time::Duration;

use anyhow::Result;
use reqwest::{RequestBuilder, Response};
use serde::{Deserialize, Serialize};

pub const PROVIDER_REQUEST_TIMEOUT: Duration = Duration::from_secs(60);

pub(crate) async fn send_provider_request(
    request: RequestBuilder,
    provider: &str,
) -> Result<Response> {
    send_provider_request_with_timeout(request, provider, PROVIDER_REQUEST_TIMEOUT).await
}

async fn send_provider_request_with_timeout(
    request: RequestBuilder,
    provider: &str,
    timeout: Duration,
) -> Result<Response> {
    request
        .timeout(timeout)
        .send()
        .await
        .map_err(|error| provider_request_error(provider, "send request to", error, timeout))
}

pub(crate) async fn ensure_provider_success(
    response: Response,
    provider: &str,
) -> Result<Response> {
    if response.status().is_success() {
        return Ok(response);
    }

    let status = response.status();
    let body = response.text().await.map_err(|error| {
        provider_request_error(
            provider,
            "read response from",
            error,
            PROVIDER_REQUEST_TIMEOUT,
        )
    })?;
    anyhow::bail!("{} API error ({}): {}", provider, status, body)
}

pub(crate) fn provider_response_error(
    provider: &str,
    action: &'static str,
    error: reqwest::Error,
) -> anyhow::Error {
    provider_request_error(provider, action, error, PROVIDER_REQUEST_TIMEOUT)
}

fn provider_request_error(
    provider: &str,
    action: &'static str,
    error: reqwest::Error,
    timeout: Duration,
) -> anyhow::Error {
    if error.is_timeout() {
        anyhow::anyhow!(
            "{} request timed out after {}. Check your connection and try again.",
            provider,
            format_timeout(timeout)
        )
    } else {
        anyhow::Error::new(error).context(format!("Failed to {} {}", action, provider))
    }
}

fn format_timeout(timeout: Duration) -> String {
    if timeout.as_secs() > 0 && timeout.subsec_nanos() == 0 {
        format!("{} seconds", timeout.as_secs())
    } else {
        format!("{} milliseconds", timeout.as_millis())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    pub system_prompt: String,
    pub user_prompt: String,
    pub max_tokens: u32,
    /// Optional sampling temperature. Providers may omit it for incompatible models.
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub content: String,
    pub usage: Option<LlmUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::{net::TcpListener, time::sleep};

    #[tokio::test]
    async fn provider_requests_fail_with_an_actionable_timeout() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (_socket, _) = listener.accept().await.unwrap();
            sleep(Duration::from_secs(1)).await;
        });

        let request = reqwest::Client::new().get(format!("http://{address}"));
        let error =
            send_provider_request_with_timeout(request, "Test provider", Duration::from_millis(20))
                .await
                .unwrap_err()
                .to_string();

        server.abort();
        assert_eq!(
            error,
            "Test provider request timed out after 20 milliseconds. Check your connection and try again."
        );
    }
}
