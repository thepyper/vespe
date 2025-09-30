use async_trait::async_trait;
use tracing::debug;

use crate::error::ProjectError;
use super::LLMClient;

/// Implementazione di LLMClient per OpenAI.
pub struct OpenAIClient {
    model: String,
    api_key: String,
    client: reqwest::Client,
}

impl OpenAIClient {
    pub fn new(model: String, api_key: String) -> Self {
        OpenAIClient {
            model,
            api_key,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl LLMClient for OpenAIClient {
    async fn send_query(&self, formatted_prompt: String) -> Result<String, ProjectError> {
        debug!("OpenAI Request: Model={}, Prompt={}", self.model, formatted_prompt);
        let url = "https://api.openai.com/v1/chat/completions";
        let payload = serde_json::json!({
            "model": self.model,
            "messages": [
                {"role": "user", "content": formatted_prompt}
            ],
            "stream": false,
        });

        let response = self.client.post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&payload)
            .send()
            .await
            .map_err(|e| ProjectError::LLMClientError(format!("OpenAI request failed: {}", e)))?;

        let response_json: serde_json::Value = response.json().await
            .map_err(|e| ProjectError::LLMClientError(format!("Failed to get OpenAI JSON response: {}", e)))?;
        debug!("OpenAI Response: {}", serde_json::to_string_pretty(&response_json).unwrap_or_else(|_| "<unparseable JSON>".to_string()));

        // OpenAI returns a JSON object with choices[0].message.content
        response_json["choices"][0]["message"]["content"].as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| ProjectError::LLMClientError("OpenAI response missing expected content.".to_string()))
    }
}
