use async_trait::async_trait;
use tracing::debug;

use crate::error::ProjectError;
use super::{LLMClient};

/// Implementazione di LLMClient per Ollama.
pub struct OllamaClient {
    model: String,
    endpoint: String,
    client: reqwest::Client,
}

impl OllamaClient {
    pub fn new(model: String, endpoint: String) -> Self {
        OllamaClient {
            model,
            endpoint,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl LLMClient for OllamaClient {
    async fn send_query(&self, formatted_prompt: String) -> Result<String, ProjectError> {
        debug!("Ollama Request: Model={}, Endpoint={}, Prompt={}", self.model, self.endpoint, formatted_prompt);
        let url = format!("{}/api/generate", self.endpoint);
        let payload = serde_json::json!({
            "model": self.model,
            "prompt": formatted_prompt,
            "stream": false,
        });

        let response = self.client.post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| ProjectError::LLMClientError(format!("Ollama request failed: {}", e)))?;

        let response_text = response.text().await
            .map_err(|e| ProjectError::LLMClientError(format!("Failed to get Ollama response text: {}", e)))?;
        debug!("Ollama Response: {}", response_text);

        // Ollama returns a JSON object with a 'response' field
        let json_response: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| ProjectError::LLMClientError(format!("Failed to parse Ollama JSON response: {}", e)))?;

        json_response["response"].as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| ProjectError::LLMClientError("Ollama response missing 'response' field.".to_string()))
    }
}
