use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use tracing::info;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::agent::agent_trait::Agent;
use crate::agent::models::AgentDefinition;
use crate::llm::llm_client::LlmClient;
use crate::llm::messages::{AssistantContent, Message, ToolOutput};
use crate::prompt_templating::PromptTemplater;
use crate::tools::tool_registry::ToolRegistry;
use crate::statistics::models::UsageStatistics;

pub struct BasicAgent {
    definition: AgentDefinition,
    tool_registry: ToolRegistry,
    llm_client: LlmClient,
    prompt_templater: PromptTemplater,
    stats: Arc<Mutex<UsageStatistics>>,
}

impl BasicAgent {
    pub fn new(
        definition: AgentDefinition,
        tool_registry: ToolRegistry,
        llm_client: LlmClient,
        prompt_templater: PromptTemplater,
        stats: Arc<Mutex<UsageStatistics>>,
    ) -> Result<Self> {
        Ok(Self {
            definition,
            tool_registry,
            llm_client,
            prompt_templater,
            stats,
        })
    }

    /// Handles the agent loop of processing tool calls from the LLM.
    async fn _handle_tool_calls(
        &self,
        messages: &mut Vec<Message>,
        initial_assistant_content: Vec<AssistantContent>,
    ) -> Result<String> {
        let mut current_assistant_content = initial_assistant_content;
        let mut final_text_response = String::new();

        for iteration_count in 0..5 {
            // Max 5 tool calls to prevent infinite loops
            info!("Agent '{}' - Iteration {}", self.name(), iteration_count);

            messages.push(Message::Assistant(current_assistant_content.clone()));

            let mut has_tool_call = false;
            let mut text_parts = Vec::new();

            for content_part in current_assistant_content {
                match content_part {
                    AssistantContent::ToolCall(tool_call) => {
                        has_tool_call = true;
                        info!("Executing tool: {}", tool_call.name);

                        let tool_output_value = self
                            .tool_registry
                            .execute_tool(&tool_call.name, &tool_call.arguments)
                            .await?;

                        messages.push(Message::Tool(ToolOutput {
                            tool_name: tool_call.name.clone(),
                            output: tool_output_value,
                        }));
                    }
                    AssistantContent::Text(text) => {
                        text_parts.push(text);
                    }
                    AssistantContent::Thought(thought) => {
                        info!("Agent '{}' thought: {}", self.name(), thought);
                    }
                }
            }

            final_text_response = text_parts.join("\n");

            if !has_tool_call {
                break; // No tool calls, so we have our final answer.
            }

            // If there were tool calls, get a new response from the LLM
            current_assistant_content = self.llm_client.generate_response(messages).await?;
        }

        Ok(final_text_response)
    }
}

#[async_trait]
impl Agent for BasicAgent {
    fn name(&self) -> &str {
        &self.definition.name
    }

    async fn execute(&self, input: &str) -> Result<String> {
        info!("Agent '{}' executing with input: '{}'", self.name(), input);

        // 1. Get tool metadata for the system prompt
        let tool_metadata = self.tool_registry.get_tool_metadata();
        let tool_prompt_part = if tool_metadata.is_empty() {
            String::new()
        } else {
            serde_json::to_string_pretty(&tool_metadata).unwrap_or_default()
        };

        // 2. Build the system prompt using the templater
        let mut data = serde_json::Map::new();
        data.insert("tool_prompt".to_string(), Value::String(tool_prompt_part));
        let system_prompt = self
            .prompt_templater
            .render_prompt("system_prompt", &Value::Object(data))?;

        // 3. Initialize the message history
        let mut messages = vec![
            Message::System(system_prompt),
            Message::User(input.to_string()),
        ];

        // 4. Get the initial response from the LLM
        let initial_assistant_content = self.llm_client.generate_response(&messages).await?;

        // 5. Enter the tool call loop
        let final_response = self
            ._handle_tool_calls(&mut messages, initial_assistant_content)
            .await?;

        info!(
            "Agent '{}' received final response: '{}'",
            self.name(),
            final_response
        );
        Ok(final_response)
    }
}
