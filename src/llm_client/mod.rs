use async_trait::async_trait;
use crate::error::ProjectError;
use crate::agent::LLMProviderConfig;
use std::path::Path;

pub mod ollama;
pub mod openai;
pub mod gemini;

use ollama::OllamaClient;
use openai::OpenAIClient;
use gemini::GeminiClient;

/// Trait per un client LLM generico.
/// Ogni implementazione gestirà la comunicazione con uno specifico provider LLM.
#[async_trait]
pub trait LLMClient: Send + Sync {
    /// Invia una query formattata all'LLM e restituisce la risposta raw.
    async fn send_query(&self, formatted_prompt: String) -> Result<String, ProjectError>;
}

/// Factory function per creare un LLMClient basato su LLMProviderConfig.
pub fn create_llm_client(project_root: &Path, config: &LLMProviderConfig) -> Result<Box<dyn LLMClient>, ProjectError> {
    match config {
        LLMProviderConfig::Ollama { model, endpoint } => {
            Ok(Box::new(OllamaClient::new(model.clone(), endpoint.clone())))
        },
        LLMProviderConfig::OpenAI { model, api_key_env } => {
            let api_key = std::env::var(api_key_env)
                .map_err(|_| ProjectError::LLMClientError(format!("OpenAI API key environment variable '{}' not set.", api_key_env)))?;
            Ok(Box::new(OpenAIClient::new(model.clone(), api_key)))
        },
        LLMProviderConfig::Gemini { model, client_id_env, client_secret_env } => {
            let client_id = std::env::var(client_id_env)
                .map_err(|_| ProjectError::LLMClientError(format!("Gemini CLIENT_ID environment variable '{}' not set.", client_id_env)))?;
            let client_secret = std::env::var(client_secret_env)
                .map_err(|_| ProjectError::LLMClientError(format!("Gemini CLIENT_SECRET environment variable '{}' not set.", client_secret_env)))?;

            // block_on is used here for simplicity in this prototype. In a real application,
            // create_llm_client should ideally be async or called from an async context.
            let client = tokio::runtime::Handle::current().block_on(async {
                GeminiClient::new(project_root.to_path_buf(), model.clone(), client_id, client_secret).await
            })?;
            Ok(Box::new(client))
        },
    }
}
