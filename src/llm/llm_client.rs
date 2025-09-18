use anyhow::{anyhow, Result};
use crate::config::models::LlmConfig;
use crate::llm::messages::{Message, AssistantContent};
use llm::builder::{LLMBackend, LLMBuilder};
use tracing::info;

// Temporary simple formatter
fn format_messages_simple(messages: &[Message]) -> String {
    messages.iter().map(|m| match m {
        Message::System(s) => format!("System: {}", s),
        Message::User(s) => format!("User: {}", s),
        Message::Assistant(contents) => format!("Assistant: {:?}", contents),
        Message::Tool(output) => format!("Tool Output: {:?}", output),
    }).collect::<Vec<_>>().join("
")
}

pub struct LlmClient {
    config: LlmConfig,
}

impl LlmClient {
    pub fn new(config: LlmConfig) -> Self {
        Self { config }
    }

    pub async fn generate_response(&self, messages: &[
Message]) -> Result<Vec<AssistantContent>> {
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

        info!("LLM Query (internal messages):
{:#?}", messages);

        // The old `format_query` is gone. We'll use a temporary simple formatter.
        // In the next phases, this will be replaced by `query_formatter.rs`.
        let chat_history = format_messages_simple(messages);

        info!("LLM Query (formatted for LLM):
{:#?}", chat_history);

        // The `llm` crate expects a `&[ChatMessage]`. For now, we can't easily build this,
        // so this part will fail to compile. The goal of phase 0 is to remove old code.
        // We will fix the compilation in the next phases.
        // For now, let's assume we can send the raw string and get a raw string back.
        // This is a placeholder to make the code structure clean.
        
        // let response = llm.chat(&chat_messages).await?;
        // let response_content = response.to_string();
        
        // HACK: To make progress, let's pretend the LLM call happens and returns a dummy response.
        // This will be replaced in the next phases.
        let response_content = "This is a raw response from the LLM.".to_string();
        info!("LLM Raw Response:
{}", response_content);

        // The old `parse_response` is gone. We just wrap the raw response in a Text content block.
        let parsed_response = vec![AssistantContent::Text(response_content)];
        info!("LLM Parsed Response (raw wrapper):
{:#?}", parsed_response);

        Ok(parsed_response)
    }
}