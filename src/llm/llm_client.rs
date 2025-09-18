use anyhow::{anyhow, Result};
use crate::config::models::LlmConfig;
use crate::llm::messages::{Message, AssistantContent};
use crate::llm::parsing;
use crate::llm::parsing::parser_trait::SnippetParser;
use llm::builder::{LLMBackend, LLMBuilder};
use llm::chat::ChatMessage;
use tracing::info;

pub struct LlmClient {
    config: LlmConfig,
    parsers: Vec<Box<dyn SnippetParser>>,
}

impl LlmClient {
    pub fn new(config: LlmConfig, parsers: Vec<Box<dyn SnippetParser>>) -> Self {
        Self { config, parsers }
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

        let parsed_response = parsing::parse_response(&response_content, &self.parsers);
        info!("LLM Parsed Response:
{:#?}", parsed_response);

        Ok(parsed_response)
    }
}
