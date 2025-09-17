use anyhow::Result;
use serde_json::Value;

use crate::llm::markdown_policy::MarkdownPolicy;
use crate::llm::messages::{Message, AssistantContent, ToolCall};
use crate::agent::core::text_utils::trim_markdown_code_blocks;
use llm::chat::{ChatMessage as LlmChatMessage, ChatRole, MessageType};

pub struct JsonMarkdownPolicy;

impl JsonMarkdownPolicy {
    pub fn new() -> Self {
        Self {}
    }
}

impl MarkdownPolicy for DefaultMarkdownPolicy {
    fn markdown_format_instructions(&self) -> String {
        r#"Your responses should be formatted using markdown. 
When you need to call a tool, use a JSON code block with the language identifier `tool_code`.
Example:
```tool_code
{
  "name": "tool_name",
  "arguments": {
    "arg1": "value1"
  }
}
```
If you have an internal thought, use a JSON code block with the language identifier `thought`.
Example:
```thought
{
  "reasoning": "I need to do X because Y"
}
```
All other content should be plain text.
"#.to_string()
    }

    fn parse_response(&self, response: &str) -> Result<Vec<AssistantContent>> {
        let trimmed_content = trim_markdown_code_blocks(response);

        // Define a temporary enum for deserialization that mirrors AgentAction
        #[derive(Debug, serde::Deserialize)]
        #[serde(untagged)] // Allows deserialization of different types from a single enum
        enum AgentActionDeserializer {
            ToolCall { name: String, args: Value },
            TextResponse { content: String },
            Thought { content: String },
        }

        // Try to parse as a Vec<AgentActionDeserializer>
        if let Ok(actions_deserialized) = serde_json::from_str::<Vec<AgentActionDeserializer>>(trimmed_content) {
            let actions: Vec<AssistantContent> = actions_deserialized.into_iter().map(|action| {
                match action {
                    AgentActionDeserializer::ToolCall { name, args } => AssistantContent::ToolCall(ToolCall { name, arguments: args }),
                    AgentActionDeserializer::TextResponse { content } => AssistantContent::Text(content),
                    AgentActionDeserializer::Thought { content } => AssistantContent::Thought(content),
                }
            }).collect();
            return Ok(actions);
        }

        // If parsing as Vec<AgentActionDeserializer> fails, try parsing as a single AgentActionDeserializer
        if let Ok(action_deserialized) = serde_json::from_str::<AgentActionDeserializer>(trimmed_content) {
            let action = match action_deserialized {
                AgentActionDeserializer::ToolCall { name, args } => AssistantContent::ToolCall(ToolCall { name, arguments: args }),
                AgentActionDeserializer::TextResponse { content } => AssistantContent::Text(content),
                AgentActionDeserializer::Thought { content } => AssistantContent::Thought(content),
            };
            return Ok(vec![action]);
        }

        // If all structured parsing fails, treat as plain text
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
                            AssistantContent::Thought(thought) => {
                                assistant_content_str.push_str(&format!("```thought\n{{\"content\": \"{}\"}}\n```", thought));
                            },
                            AssistantContent::ToolCall(tool_call) => {
                                assistant_content_str.push_str(&format!("```tool_code\n{{\"name\": \"{}\", \"arguments\": {}}}\n```", tool_call.name, serde_json::to_string(&tool_call.arguments)?));
                            },
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
