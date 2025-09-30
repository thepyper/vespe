use async_trait::async_trait;
use crate::error::ProjectError;
use crate::agent::LLMProviderConfig;
use chrono::{DateTime, Utc};
use tokio::sync::Mutex;
use std::sync::Arc;
use tracing::{debug};

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

// --- Gemini Client Implementation ---

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct GeminiTokenState {
    access_token: String,
    expires_at: DateTime<Utc>,
    refresh_token: String, // Intended for use in a real token refresh flow
    client_id: String,     // Intended for use in a real token refresh flow
    client_secret: String, // Intended for use in a real token refresh flow
}

impl GeminiTokenState {
    async fn new(client_id: String, client_secret: String, refresh_token: String) -> Result<Self, ProjectError> {
        // Placeholder for actual token exchange logic
        // In a real scenario, this would make an HTTP request to Google's token endpoint
        // to exchange the refresh_token for an initial access_token.
        eprintln!("DEBUG: GeminiTokenState::new - Performing initial token exchange...");
        Ok(GeminiTokenState {
            access_token: "initial_gemini_access_token_placeholder".to_string(),
            expires_at: Utc::now() + chrono::Duration::hours(1), // Token valid for 1 hour
            refresh_token,
            client_id,
            client_secret,
        })
    }

    async fn refresh_access_token(&mut self) -> Result<(), ProjectError> {
        // Placeholder for actual token refresh logic
        // This would make an HTTP request to Google's token endpoint using the refresh_token.
        eprintln!("DEBUG: GeminiTokenState::refresh_access_token - Refreshing token...");
        self.access_token = "refreshed_gemini_access_token_placeholder".to_string();
        self.expires_at = Utc::now() + chrono::Duration::hours(1); // New token valid for 1 hour
        Ok(())
    }

    fn is_expired(&self) -> bool {
        // Check if token is expired or will expire within the next 5 minutes
        self.expires_at < Utc::now() + chrono::Duration::minutes(5)
    }
}

pub struct GeminiClient {
    model: String,
    client: reqwest::Client,
    token_state: Arc<Mutex<GeminiTokenState>>,
}

impl GeminiClient {
    pub async fn new(model: String, client_id: String, client_secret: String, refresh_token: String) -> Result<Self, ProjectError> {
        let token_state = GeminiTokenState::new(client_id, client_secret, refresh_token).await?;
        Ok(GeminiClient {
            model,
            client: reqwest::Client::new(),
            token_state: Arc::new(Mutex::new(token_state)),
        })
    }
}

#[async_trait]
impl LLMClient for GeminiClient {
    async fn send_query(&self, formatted_prompt: String) -> Result<String, ProjectError> {
        debug!("Gemini Request: Model={}, Prompt={}", self.model, formatted_prompt);
        let mut token_state_guard = self.token_state.lock().await;

        if token_state_guard.is_expired() {
            token_state_guard.refresh_access_token().await?;
        }

        let access_token = token_state_guard.access_token.clone();
        drop(token_state_guard); // Release the lock as soon as access_token is cloned

        let url = format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent", self.model);
        let payload = serde_json::json!({
            "contents": [
                {"parts": [{"text": formatted_prompt}]}
            ]
        });

        let response = self.client.post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .json(&payload)
            .send()
            .await
            .map_err(|e| ProjectError::LLMClientError(format!("Gemini request failed: {}", e)))?;

        let response_json: serde_json::Value = response.json().await
            .map_err(|e| ProjectError::LLMClientError(format!("Failed to get Gemini JSON response: {}", e)))?;
        debug!("Gemini Response: {}", serde_json::to_string_pretty(&response_json).unwrap_or_else(|_| "<unparseable JSON>".to_string()));

        // Extract content from Gemini response
        response_json["candidates"][0]["content"]["parts"][0]["text"].as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| ProjectError::LLMClientError("Gemini response missing expected content.".to_string()))
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
        LLMProviderConfig::Gemini { model, client_id_env, client_secret_env, refresh_token_env } => {
            let client_id = std::env::var(client_id_env)
                .map_err(|_| ProjectError::LLMClientError(format!("Gemini CLIENT_ID environment variable '{}' not set.", client_id_env)))?;
            let client_secret = std::env::var(client_secret_env)
                .map_err(|_| ProjectError::LLMClientError(format!("Gemini CLIENT_SECRET environment variable '{}' not set.", client_secret_env)))?;
            let refresh_token = std::env::var(refresh_token_env)
                .map_err(|_| ProjectError::LLMClientError(format!("Gemini REFRESH_TOKEN environment variable '{}' not set.", refresh_token_env)))?;

            // block_on is used here for simplicity in this prototype. In a real application,
            // create_llm_client should ideally be async or called from an async context.
            let client = tokio::runtime::Handle::current().block_on(async {
                GeminiClient::new(model.clone(), client_id, client_secret, refresh_token).await
            })?;
            Ok(Box::new(client))
        },
    }
}
