use anyhow::{anyhow, Result};
use crate::config::models::LlmConfig;
use crate::llm::messages::{Message, AssistantContent};
use crate::llm::parsing;
use crate::llm::parsing::parser_trait::SnippetParser;
use llm::builder::{LLMBackend, LLMBuilder};
use llm::chat::ChatMessage;
use tracing::info;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::statistics::models::UsageStatistics;

pub struct LlmClient {
    config: LlmConfig,
    parsers: Vec<Box<dyn SnippetParser>>,
    stats: Arc<Mutex<UsageStatistics>>,
}

impl LlmClient {
    pub fn new(config: LlmConfig, parsers: Vec<Box<dyn SnippetParser>>, stats: Arc<Mutex<UsageStatistics>>) -> Self {
        Self { config, parsers, stats }
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

        info!("LLM Query (internal messages):
{:#?}", messages);

        // Convert our internal Message enum to the llm crate's ChatMessage struct
        let chat_messages: Vec<ChatMessage> = messages.iter().map(|m| {
            match m {
                Message::System(s) => ChatMessage::user().content(s.clone()).build(),
                Message::User(s) => ChatMessage::user().content(s.clone()).build(),
                Message::Assistant(contents) => ChatMessage::assistant().content(contents.iter().map(|c| format!("{:?}", c)).collect::<Vec<_>>().join(" ")).build(),
                Message::Tool(output) => ChatMessage::user().content(format!("Tool output for '{}':{}", output.tool_name, output.output)).build(),
            }
        }).collect();

        info!("LLM Query (formatted for llm crate):
{:#?}", chat_messages);

        let response = llm.chat(&chat_messages).await?;
        let response_content = response.to_string();
        info!("LLM Raw Response:
{}", response_content);

        let (parsed_response, _used_parsers) = parsing::parse_response(
            &response_content,
            &self.parsers,
            self.stats.clone(), // Pass the Arc to parse_response
            &self.config.provider,
            &self.config.model_id,
        ).await;
        // used_parsers is now handled within parse_response
        info!("LLM Parsed Response:
{:#?}", parsed_response);

        Ok(parsed_response)
    }
}