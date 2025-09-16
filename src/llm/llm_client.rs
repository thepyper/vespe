use anyhow::{anyhow, Result};
use async_trait::async_trait;
use tracing::error;

use crate::config::models::LlmConfig;
use crate::llm::models::{ChatMessage, LlmResponse};
use crate::llm::providers::openai::OpenAiClient;

#[async_trait]
pub trait LlmClient {
    async fn generate_response(&self, messages: Vec<ChatMessage>) -> Result<LlmResponse>;
}

pub struct GenericLlmClient {
    config: LlmConfig,
    openai_client: Option<OpenAiClient>,
    // Add other provider clients here
}

impl GenericLlmClient {
    pub fn new(config: LlmConfig) -> Result<Self> {
        let openai_client = if config.provider == "openai" {
            Some(OpenAiClient::new(config.clone()))
        } else {
            None
        };

        // Add other provider initializations here

        if openai_client.is_none() {
            error!("Unsupported LLM provider: {}", config.provider);
            return Err(anyhow!("Unsupported LLM provider: {}", config.provider));
        }

        Ok(Self {
            config,
            openai_client,
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
            // Add other provider matches here
            _ => Err(anyhow!("Unsupported LLM provider: {}", self.config.provider)),
        }
    }
}
