use async_trait::async_trait;
use crate::error::ProjectError;
use crate::agent::{LLMProviderConfig, AIConfig};

/// Trait per un client LLM generico.
/// Ogni implementazione gestirà la comunicazione con uno specifico provider LLM.
#[async_trait]
pub trait LLMClient: Send + Sync {
    /// Invia una query formattata all'LLM e restituisce la risposta raw.
    async fn send_query(&self, formatted_prompt: String) -> Result<String, ProjectError>;
}

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

        // Ollama returns a JSON object with a 'response' field
        let json_response: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| ProjectError::LLMClientError(format!("Failed to parse Ollama JSON response: {}", e)))?;

        json_response["response"].as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| ProjectError::LLMClientError("Ollama response missing 'response' field.".to_string()))
    }
}

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

        // OpenAI returns a JSON object with choices[0].message.content
        response_json["choices"][0]["message"]["content"].as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| ProjectError::LLMClientError("OpenAI response missing expected content.".to_string()))
    }
}

/// Factory function per creare un LLMClient basato su LLMProviderConfig.
pub fn create_llm_client(config: &LLMProviderConfig) -> Result<Box<dyn LLMClient>, ProjectError> {
    match config {
        LLMProviderConfig::Ollama { model, endpoint } => {
            Ok(Box::new(OllamaClient::new(model.clone(), endpoint.clone())))
        },
        LLMProviderConfig::OpenAI { model, api_key_env } => {
            let api_key = std::env::var(api_key_env)
                .map_err(|_| ProjectError::LLMClientError(format!("OpenAI API key environment variable '{}' not set.", api_key_env)))?;
            Ok(Box::new(OpenAIClient::new(model.clone(), api_key)))
        },
    }
}
