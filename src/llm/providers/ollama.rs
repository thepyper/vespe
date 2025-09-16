use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::json;
use tracing::info;

use crate::config::LlmConfig;
use crate::llm::models::{ChatMessage, LlmResponse};

pub struct OllamaClient {
    client: Client,
    config: LlmConfig,
}

impl OllamaClient {
    pub fn new(config: LlmConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    pub async fn generate_response(&self, messages: Vec<ChatMessage>) -> Result<LlmResponse> {
        let model = &self.config.model_id;
        let url = "http://localhost:11434/api/chat"; // Default Ollama API endpoint

        let payload = json!({ 
            "model": model,
            "messages": messages,
            "stream": false,
            "temperature": self.config.temperature,
        });

        info!("Ollama Request Payload: {}", serde_json::to_string_pretty(&payload).unwrap_or_default());

        let response = self.client.post(url)
            .json(&payload)
            .send()
            .await
            .context("Failed to send request to Ollama")?;

        let response_text = response.text().await.context("Failed to get response text from Ollama")?;
        info!("Ollama Raw Response: {}", response_text);

        let json_response: serde_json::Value = serde_json::from_str(&response_text)
            .context("Failed to parse Ollama response JSON")?;

        let content = json_response["message"]["content"]
            .as_str()
            .context("Failed to extract content from Ollama response")?;

        Ok(LlmResponse { content: content.to_string() })
    }
}