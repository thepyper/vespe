use reqwest::Client;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use tracing::debug;

#[derive(Debug, Serialize)]
pub struct OllamaGenerateRequest<'a> {
    pub model: &'a str,
    pub prompt: &'a str,
    pub system: Option<&'a str>,
    pub stream: bool,
}

#[derive(Debug, Deserialize)]
pub struct OllamaGenerateResponse {
    pub response: String,
}

pub async fn query_ollama(
    client: &Client,
    ollama_url: &str,
    model: &str,
    prompt: &str,
    system: Option<&str>,
) -> Result<String> {
    debug!("Ollama Request: model={}, system={:?}, prompt={}", model, system, prompt);
    let request_payload = OllamaGenerateRequest {
        model,
        prompt,
        system,
        stream: false,
    };
    let response = client
        .post(format!("{}/api/generate", ollama_url))
        .json(&request_payload)
        .send()
        .await?;
    let response_body = response.json::<OllamaGenerateResponse>().await?;
    debug!("Ollama Response: {}", response_body.response);
    Ok(response_body.response.trim().to_string())
}
