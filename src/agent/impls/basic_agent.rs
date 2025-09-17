use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use tracing::info;

use crate::agent::agent_trait::Agent;
use crate::agent::models::AgentDefinition;
use crate::llm::llm_client::LlmClient;
use crate::llm::messages::{Message, AssistantContent, ToolOutput};
use crate::tools::tool_registry::ToolRegistry;
use crate::prompt_templating::PromptTemplater;

pub struct BasicAgent {
    definition: AgentDefinition,
    tool_registry: ToolRegistry,
    llm_client: LlmClient,
    prompt_templater: PromptTemplater,
}

impl BasicAgent {
    pub fn new(definition: AgentDefinition, tool_registry: ToolRegistry, llm_client: LlmClient, prompt_templater: PromptTemplater) -> Result<Self> {
        Ok(Self { definition, tool_registry, llm_client, prompt_templater })
    }

    async fn _handle_tool_calls(
        &self,
        messages: &mut Vec<Message>,
        initial_assistant_content: Vec<AssistantContent>,
        final_response_parts: &mut Vec<String>,
    ) -> Result<()> {
        let mut current_assistant_content = initial_assistant_content;

        // Loop for tool calls
        for iteration_count in 0..5 { // Max 5 tool calls to prevent infinite loops
            info!("Agent '{}' - Iteration {}", self.name(), iteration_count);

            messages.push(Message::Assistant(current_assistant_content.clone()));

            let mut has_tool_call = false;
            for content_part in &current_assistant_content {
                match content_part {
                    AssistantContent::ToolCall(tool_call) => {
                        has_tool_call = true;
                        final_response_parts.push(format!("[TOOL_CALL]: {}\n```json\n{}\n```", tool_call.name, serde_json::to_string_pretty(&tool_call.arguments)?));

                        let tool_output_value = self.tool_registry.execute_tool(&tool_call.name, &tool_call.arguments).await?;
                        let tool_output_str = serde_json::to_string_pretty(&tool_output_value)?;

                        messages.push(Message::Tool(ToolOutput { tool_name: tool_call.name.clone(), output: tool_output_value.clone() }));

                        // Explicitly extract and report transformed_text for echo tool
                        if tool_call.name == "echo" {
                            if let Some(transformed_text) = tool_output_value["transformed_text"].as_str() {
                                final_response_parts.push(format!("Echo Tool Output: {}", transformed_text));
                            } else {
                                final_response_parts.push(format!("Echo Tool Output (raw): {}", tool_output_str));
                            }
                        } else {
                            final_response_parts.push(format!("Tool output: {}", tool_output_str));
                        }
                    },
                    AssistantContent::Text(content) => {
                        final_response_parts.push(content.clone());
                    },
                    AssistantContent::Thought(content) => {
                        info!("Agent '{}' thought: {}", self.name(), content);
                        final_response_parts.push(format!("[THOUGHT]: {}", content));
                    },
                }
            }

            if !has_tool_call {
                break; // No tool call detected, or all actions processed
            }

            // If there were tool calls, get a new response from the LLM
            current_assistant_content = self.llm_client.generate_response(messages).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl Agent for BasicAgent {
    fn name(&self) -> &str {
        &self.definition.name
    }

    async fn execute(&self, input: &str) -> Result<String> {
        info!("Agent '{}' executing with input: '{}'", self.name(), input);
        
        // 1. Get markdown format instructions from the policy
        let markdown_instructions = self.llm_client.markdown_policy().markdown_format_instructions();

        // 2. Get tool metadata
        let tool_metadata = self.tool_registry.get_tool_metadata();
        let tool_prompt_part = if tool_metadata.is_empty() {
            String::new()
        } else {
            serde_json::to_string_pretty(&tool_metadata).unwrap_or_default()
        };

        // 3. Combine with agent-specific instructions using PromptTemplater
        let mut data = serde_json::Map::new();
        data.insert("tool_prompt".to_string(), Value::String(tool_prompt_part));
        data.insert("markdown_instructions".to_string(), Value::String(markdown_instructions));
        // Add other agent-specific data from self.definition as needed

        let system_prompt = self.prompt_templater.render_prompt("system_prompt", &Value::Object(data))?;

        let mut messages = vec![
            Message::System(system_prompt),
            Message::User(input.to_string()),
        ];

        let initial_assistant_content = self.llm_client.generate_response(&messages).await?;
        let mut final_response_parts = Vec::new();

        self._handle_tool_calls(&mut messages, initial_assistant_content, &mut final_response_parts).await?;

        let final_response_content = final_response_parts.join("\n");
        info!("Agent '{}' received final response: '{}'", self.name(), final_response_content);
        Ok(final_response_content)
    }
}