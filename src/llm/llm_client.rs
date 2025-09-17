use anyhow::{anyhow, Result};
use crate::config::models::LlmConfig;
use crate::llm::models::LlmResponse; // LlmResponse is still used for raw content
use crate::llm::markdown_policy::MarkdownPolicy;
use crate::llm::messages::{Message, AssistantContent};
use llm::builder::{LLMBackend, LLMBuilder};
use llm::chat::{ChatMessage as LlmChatMessage, ChatRole, MessageType}; // Still needed for the underlying LLM crate
use tracing::info;

pub struct LlmClient {
    config: LlmConfig,
    markdown_policy: Box<dyn MarkdownPolicy>,
}

impl LlmClient {
    pub fn new(config: LlmConfig, markdown_policy: Box<dyn MarkdownPolicy>) -> Self {
        Self { config, markdown_policy }
    } 

    pub async fn generate_response(&self, messages: &[Message]) -> Result<Vec<AssistantContent>> {
        let backend = match self.config.provider.as_str() {
            "openai" => LLMBackend::OpenAI,
            "ollama" => LLMBackend::Ollama,
            _ => return Err(anyhow!("Unsupported LLM provider: {}", self.config.provider)),
        };

        let llm = LLMBuilder::new()
            .backend(backend)
            .api_key(self.config.api_key.clone().unwrap_or_default())
            .model(self.config.model_id.clone())
            .build()?;

        info!("LLM Query (internal messages):\n{:#?}", messages);

        let chat_messages = self.markdown_policy.format_query(messages)?;

        info!("LLM Query (formatted for LLM):\n{:#?}", chat_messages);

        let response = llm.chat(&chat_messages).await?;
        let response_content = response.to_string();
        info!("LLM Raw Response:\n{}", response_content);

        let parsed_response = self.markdown_policy.parse_response(&response_content)?;
        info!("LLM Parsed Response:\n{:#?}", parsed_response);

        Ok(parsed_response)
    }
}
