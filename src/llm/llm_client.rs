use anyhow::{anyhow, Result};
use crate::config::models::LlmConfig;
use crate::llm::models::{ChatMessage, LlmResponse};
use llm::builder::{LLMBackend, LLMBuilder};
use llm::chat::{ChatMessage as LlmChatMessage, ChatRole, MessageType};

pub async fn generate_response(
    config: &LlmConfig,
    messages: Vec<ChatMessage>,
) -> Result<LlmResponse> {
    let backend = match config.provider.as_str() {
        "openai" => LLMBackend::OpenAI,
        "ollama" => LLMBackend::Ollama,
        _ => return Err(anyhow!("Unsupported LLM provider: {}", config.provider)),
    };

    let llm = LLMBuilder::new()
        .backend(backend)
        .api_key(config.api_key.clone().unwrap_or_default())
        .model(config.model_id.clone())
        .build()?;

    let chat_messages: Vec<LlmChatMessage> = messages
        .into_iter()
        .map(|msg| LlmChatMessage {
            role: match msg.role.as_str() {
                "user" => ChatRole::User,
                "assistant" => ChatRole::Assistant,
                "system" => ChatRole::User, // FIXME
                _ => ChatRole::User,
            },
            content: msg.content,
            message_type: MessageType::Text,
        })
        .collect();

    let response = llm.chat(&chat_messages).await?;

    Ok(LlmResponse {
        content: response.to_string(),
    })
}