use async_trait::async_trait;
use crate::memory::{Message, MessageContent};
use crate::tool::ToolConfig;
use crate::agent_protocol::{AgentProtocol, AgentProtocolError};
use serde_json::{json, to_string_pretty, Value};
use serde::{Serialize, Deserialize};

/// Implementazione del Model Context Protocol (MCP) per AgentProtocol.
pub struct McpAgentProtocol;



#[derive(Debug, Serialize, Deserialize)]
struct McpMessage {
    role: String,
    content: Value,
}

#[async_trait]
impl AgentProtocol for McpAgentProtocol {
    async fn format_query(
        &self,
        context: crate::agent_protocol::QueryContext<'_>,
    ) -> Result<String, AgentProtocolError> {
        let mut mcp_messages: Vec<McpMessage> = Vec::new();

        // Add system instructions if present
        if let Some(instructions) = context.system_instructions {
            mcp_messages.push(McpMessage {
                role: "system".to_string(),
                content: json!({ "type": "text", "text": instructions }),
            });
        }

        // Add available tools as a system message
        if !context.available_tools.is_empty() {
            let tools_json = to_string_pretty(context.available_tools)
                .map_err(|e| AgentProtocolError::SerializationError(e))?;
            mcp_messages.push(McpMessage {
                role: "system".to_string(),
                content: json!({ "type": "tool_definitions", "tools": serde_json::from_str::<serde_json::Value>(&tools_json).unwrap_or_default() }),
            });
        }

        // Combine agent_context and task_context and format them
        let all_messages = context.agent_context.iter().chain(context.task_context.iter());

        for message in all_messages {
            let role = match message.author_agent_uid.as_str() {
                "user" => "user".to_string(),
                "system" => "system".to_string(),
                _ => "assistant".to_string(), // Default to assistant for other agents
            };

            let content_value = match &message.content {
                MessageContent::Text(text) => {
                    json!({ "type": "text", "text": text })
                },
                MessageContent::Thought(thought) => {
                    json!({ "type": "text", "text": format!("Thought: {}", thought) })
                },
                MessageContent::ToolCall { tool_name, call_uid, inputs } => {
                    json!({ "type": "tool_use", "id": call_uid, "name": tool_name, "input": inputs })
                },
                MessageContent::ToolResult { tool_name: _, call_uid, inputs: _, outputs } => {
                    json!({ "type": "tool_result", "tool_call_id": call_uid, "output": outputs })
                },
            };

            mcp_messages.push(McpMessage {
                role,
                content: content_value,
            });
        }

        to_string_pretty(&mcp_messages)
            .map_err(|e| AgentProtocolError::SerializationError(e))
    }

    async fn parse_llm_output(
        &self,
        llm_output: String,
    ) -> Result<Vec<Message>, AgentProtocolError> {
        let parsed_output: Value = serde_json::from_str(&llm_output)
            .map_err(|e| AgentProtocolError::ParseError(format!("Failed to parse LLM output as JSON: {}", e)))?;

        let mut messages = Vec::new();

        // Expecting the LLM output to be a single message object with role "assistant"
        // and content that can be a single object or an array of objects.
        if let Some(mcp_message) = parsed_output.as_object() {
            let role = mcp_message["role"].as_str().unwrap_or("assistant");
            if role != "assistant" {
                return Err(AgentProtocolError::ParseError(format!("Expected assistant role in LLM output, got: {}", role)));
            }

            if let Some(content_array) = mcp_message["content"].as_array() {
                for content_item in content_array {
                    if let Some(content_type) = content_item["type"].as_str() {
                        match content_type {
                            "text" => {
                                if let Some(text) = content_item["text"].as_str() {
                                    messages.push(Message {
                                        uid: crate::utils::generate_uid("msg").map_err(|e| AgentProtocolError::ParseError(e.to_string()))?,
                                        timestamp: chrono::Utc::now(),
                                        author_agent_uid: "assistant".to_string(),
                                        content: MessageContent::Text(text.to_string()),
                                        status: crate::memory::MessageStatus::Enabled,
                                    });
                                }
                            },
                            "tool_use" => {
                                let id = content_item["id"].as_str().ok_or_else(|| AgentProtocolError::ParseError("Tool use 'id' missing".to_string()))?;
                                let name = content_item["name"].as_str().ok_or_else(|| AgentProtocolError::ParseError("Tool use 'name' missing".to_string()))?;
                                let inputs = content_item["input"].clone();

                                messages.push(Message {
                                    uid: crate::utils::generate_uid("msg").map_err(|e| AgentProtocolError::ParseError(e.to_string()))?,
                                    timestamp: chrono::Utc::now(),
                                    author_agent_uid: "assistant".to_string(),
                                    content: MessageContent::ToolCall {
                                        tool_name: name.to_string(),
                                        call_uid: id.to_string(),
                                        inputs,
                                    },
                                    status: crate::memory::MessageStatus::Enabled,
                                });
                            },
                            _ => {
                                // Ignore unknown content types for now
                            }
                        }
                    }
                }
            } else if let Some(single_content) = mcp_message["content"].as_object() {
                if let Some(content_type) = single_content["type"].as_str() {
                    if content_type == "text" {
                        if let Some(text) = single_content["text"].as_str() {
                            messages.push(Message {
                                uid: crate::utils::generate_uid("msg").map_err(|e| AgentProtocolError::ParseError(e.to_string()))?,
                                timestamp: chrono::Utc::now(),
                                author_agent_uid: "assistant".to_string(),
                                content: MessageContent::Text(text.to_string()),
                                status: crate::memory::MessageStatus::Enabled,
                            });
                        }
                    }
                }
            }
        }

        Ok(messages)
    }
}
