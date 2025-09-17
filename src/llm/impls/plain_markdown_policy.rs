use anyhow::Result;

use crate::llm::markdown_policy::MarkdownPolicy;
use crate::llm::messages::{Message, AssistantContent};
use llm::chat::{ChatMessage as LlmChatMessage, ChatRole, MessageType};

pub struct PlainMarkdownPolicy;

impl PlainMarkdownPolicy {
    pub fn new() -> Self {
        Self {}
    }
}

impl MarkdownPolicy for PlainMarkdownPolicy {
    fn markdown_format_instructions(&self) -> String {
        "Your responses should be in plain text. Do not use any markdown formatting, especially for tool calls or thoughts.".to_string()
    }

    fn parse_response(&self, response: &str) -> Result<Vec<AssistantContent>> {
        Ok(vec![AssistantContent::Text(response.to_string())])
    }

    fn format_query(&self, messages: &[Message]) -> Result<Vec<LlmChatMessage>> {
        let mut llm_chat_messages = Vec::new();

        for msg in messages {
            match msg {
                Message::System(content) => {
                    llm_chat_messages.push(LlmChatMessage {
                        role: ChatRole::User,
                        content: content.clone(),
                        message_type: MessageType::Text,
                    });
                },
                Message::User(content) => {
                    llm_chat_messages.push(LlmChatMessage {
                        role: ChatRole::User,
                        content: content.clone(),
                        message_type: MessageType::Text,
                    });
                },
                Message::Assistant(content_parts) => {
                    let mut assistant_content_str = String::new();
                    for part in content_parts {
                        match part {
                            AssistantContent::Text(text) => assistant_content_str.push_str(text),
                            // For PlainMarkdownPolicy, thoughts and tool calls are treated as plain text
                            AssistantContent::Thought(thought) => assistant_content_str.push_str(&format!("Thought: {}", thought)),
                            AssistantContent::ToolCall(tool_call) => assistant_content_str.push_str(&format!("Tool Call: {}({:?})", tool_call.name, tool_call.arguments)),
                        }
                    }
                    llm_chat_messages.push(LlmChatMessage {
                        role: ChatRole::Assistant,
                        content: assistant_content_str,
                        message_type: MessageType::Text,
                    });
                },
                Message::Tool(tool_output) => {
                    llm_chat_messages.push(LlmChatMessage {
                        role: ChatRole::User, // Mapped to User role as there's no specific Tool role
                        content: serde_json::to_string(&tool_output.output)?,
                        message_type: MessageType::Text,
                    });
                },
            }
        }
        Ok(llm_chat_messages)
    }
}
