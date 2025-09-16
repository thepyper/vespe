use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::json;
use tracing::info;

use crate::config::models::LlmConfig;
use crate::llm::models::{ChatMessage, LlmResponse};

pub struct OpenAiClient {
    client: Client,
    config: LlmConfig,
}

impl OpenAiClient {
    pub fn new(config: LlmConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    pub async fn generate_response(&self, messages: Vec<ChatMessage>) -> Result<LlmResponse> {
        let api_key = self.config.api_key.as_ref().context("OpenAI API key not provided")?;
        let model = &self.config.model_id;
        let url = "https://api.openai.com/v1/chat/completions";

        info!("Sending request to OpenAI model: {}", model);

        let response = self.client.post(url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&json!({ 
                "model": model,
                "messages": messages,
                "temperature": self.config.temperature,
                "max_tokens": self.config.max_tokens,
            }))
            .send()
            .await
            .context("Failed to send request to OpenAI")?;

        let response_text = response.text().await.context("Failed to get response text from OpenAI")?;
        info!("OpenAI raw response: {}", response_text);

        let json_response: serde_json::Value = serde_json::from_str(&response_text)
            .context("Failed to parse OpenAI response JSON")?;

        let content = json_response["choices"][0]["message"]["content"]
            .as_str()
            .context("Failed to extract content from OpenAI response")?;

        Ok(LlmResponse { content: content.to_string() })
    }
}
