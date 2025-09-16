use anyhow::{anyhow, Result};
use async_trait::async_trait;
use tracing::error;

use crate::config::LlmConfig;
use crate::llm::models::{ChatMessage, LlmResponse};
use crate::llm::providers::openai::OpenAiClient;
use crate::llm::providers::ollama::OllamaClient;

#[async_trait]
pub trait LlmClient {
    async fn generate_response(&self, messages: Vec<ChatMessage>) -> Result<LlmResponse>;
}

pub struct GenericLlmClient {
    config: LlmConfig,
    openai_client: Option<OpenAiClient>,
    ollama_client: Option<OllamaClient>,
}

impl GenericLlmClient {
    pub fn new(config: LlmConfig) -> Result<Self> {
        let openai_client = if config.provider == "openai" {
            Some(OpenAiClient::new(config.clone()))
        } else {
            None
        };

        let ollama_client = if config.provider == "ollama" {
            Some(OllamaClient::new(config.clone()))
        } else {
            None
        };

        if openai_client.is_none() && ollama_client.is_none() {
            error!("Unsupported LLM provider: {}", config.provider);
            return Err(anyhow!("Unsupported LLM provider: {}", config.provider));
        }

        Ok(Self {
            config,
            openai_client,
            ollama_client,
        })
    }
}

#[async_trait]
impl LlmClient for GenericLlmClient {
    async fn generate_response(&self, messages: Vec<ChatMessage>) -> Result<LlmResponse> {
        match self.config.provider.as_str() {
            "openai" => {
                if let Some(client) = &self.openai_client {
                    client.generate_response(messages).await
                } else {
                    Err(anyhow!("OpenAI client not initialized"))
                }
            }
            "ollama" => {
                if let Some(client) = &self.ollama_client {
                    client.generate_response(messages).await
                } else {
                    Err(anyhow!("Ollama client not initialized"))
                }
            }
            _ => Err(anyhow!("Unsupported LLM provider: {}", self.config.provider)),
        }
    }
}