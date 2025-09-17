use anyhow::Result;

use crate::llm::markdown_policy::MarkdownPolicy;
use crate::llm::messages::{Message, AssistantContent, ToolCall};
use llm::chat::{ChatMessage as LlmChatMessage, ChatRole, MessageType};

pub struct JsonMarkdownPolicy;

impl JsonMarkdownPolicy {
    pub fn new() -> Self {
        Self {}
    }
}

impl MarkdownPolicy for JsonMarkdownPolicy {
    fn markdown_format_instructions(&self) -> String {
        r#"YOUR RESPONSE MUST FOLLOW STRICTLY THE FOLLOWING GUIDELINES:
1.  TOOL CALLS: When you need to call a tool, you MUST use a VALID JSON code block with the language identifier `tool_code`, as in the following example:
    ```tool_code
    {
      "name": "tool_name",
      "arguments": {
        "arg1": "value1"
      }
    }
    ```    
2.  INTERNAL THOUGHTS: If you have an internal thought or reasoning, you MUST use a VALID JSON code block with the language identifier `thought`, as in the following example:
    ```thought
    {
      "reasoning": "I need to do X because Y"
    }
    ```    
3.  PLAIN TEXT: All other content MUST be plain text. DO NOT use any other markdown formatting (e.g., bold, italics, lists) for non-tool-call or non-thought content.
YOU MUST NOT combine multiple code blocks.
"#.to_string()
    }

    fn parse_response(&self, response: &str) -> Result<Vec<AssistantContent>> {
        use regex::Regex;
        let mut parsed_content = Vec::new();
        let mut last_index = 0;

        let tool_code_re = Regex::new(r"```tool_code\n([\s\S]*?)\n```")?;
        let thought_re = Regex::new(r"```thought\n([\s\S]*?)\n```")?;

        // Find all tool_code blocks
        for mat in tool_code_re.find_iter(response) {
            let start = mat.start();
            let end = mat.end();

            // Add any preceding plain text
            if start > last_index {
                let text = response[last_index..start].trim();
                if !text.is_empty() {
                    parsed_content.push(AssistantContent::Text(text.to_string()));
                }
            }

            let json_str = mat.as_str();
            let json_str = json_str.strip_prefix("```tool_code\n").unwrap_or(json_str);
            let json_str = json_str.strip_suffix("\n```").unwrap_or(json_str);

            let tool_call: ToolCall = serde_json::from_str(json_str)
                .map_err(|e| anyhow::anyhow!("Failed to parse tool_code JSON: {}. Original: {}", e, json_str))?;
            parsed_content.push(AssistantContent::ToolCall(tool_call));
            last_index = end;
        }

        // Find all thought blocks (only in the remaining part of the response)
        // This approach assumes tool_code blocks take precedence or are distinct.
        // A more robust solution might involve a single pass with a combined regex or a state machine.
        // For simplicity, I'll re-process the remaining text for thoughts.
        let remaining_response = &response[last_index..];
        let mut current_thought_last_index = 0;

        for mat in thought_re.find_iter(remaining_response) {
            let start = mat.start();
            let end = mat.end();

            // Add any preceding plain text within the remaining_response
            if start > current_thought_last_index {
                let text = remaining_response[current_thought_last_index..start].trim();
                if !text.is_empty() {
                    parsed_content.push(AssistantContent::Text(text.to_string()));
                }
            }

            let json_str = mat.as_str();
            let json_str = json_str.strip_prefix("```thought\n").unwrap_or(json_str);
            let json_str = json_str.strip_suffix("\n```").unwrap_or(json_str);

            // Thoughts are expected to be a JSON object with a "content" field
            #[derive(Debug, serde::Deserialize)]
            struct ThoughtContent {
                content: String,
            }
            let thought_obj: ThoughtContent = serde_json::from_str(json_str)
                .map_err(|e| anyhow::anyhow!("Failed to parse thought JSON: {}. Original: {}", e, json_str))?;
            parsed_content.push(AssistantContent::Thought(thought_obj.content));
            current_thought_last_index = end;
        }

        // Add any remaining plain text after all blocks
        if last_index < response.len() {
            let text = response[last_index..].trim();
            if !text.is_empty() {
                parsed_content.push(AssistantContent::Text(text.to_string()));
            }
        }
        if current_thought_last_index < remaining_response.len() {
            let text = remaining_response[current_thought_last_index..].trim();
            if !text.is_empty() {
                parsed_content.push(AssistantContent::Text(text.to_string()));
            }
        }


        if parsed_content.is_empty() && !response.trim().is_empty() {
            // If no structured content was found, but there was a response, treat as plain text
            parsed_content.push(AssistantContent::Text(response.to_string()));
        }


        Ok(parsed_content)
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
                                assistant_content_str.push_str(&format!("```tool_code\n{{\"name\": \"{}\", \"arguments\": {}}}
```", tool_call.name, serde_json::to_string(&tool_call.arguments)?));
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
