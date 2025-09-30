use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

use crate::error::ProjectError;
use super::LLMClient;

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
