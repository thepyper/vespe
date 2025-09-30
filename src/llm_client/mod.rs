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
pub async fn create_llm_client(project_root: &Path, config: &LLMProviderConfig) -> Result<Box<dyn LLMClient>, ProjectError> {
    match config {
        LLMProviderConfig::Ollama { model, endpoint } => {
            Ok(Box::new(OllamaClient::new(model.clone(), endpoint.clone())))
        },
        LLMProviderConfig::OpenAI { model, api_key_env } => {
            let api_key = std::env::var(api_key_env)
                .map_err(|_| ProjectError::LLMClientError(format!("OpenAI API key environment variable '{}' not set.", api_key_env)))?;
            Ok(Box::new(OpenAIClient::new(model.clone(), api_key)))
        },
        LLMProviderConfig::Gemini { model } => {
            let client = GeminiClient::new(project_root.to_path_buf(), model.clone()).await?;
            Ok(Box::new(client))
        },
    }
}
