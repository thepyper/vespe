use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use serde_json::Value;
use tracing::info;

use crate::agent::agent_trait::Agent;
use crate::agent::models::AgentDefinition;
use crate::llm::models::{ChatMessage, LlmResponse};
use crate::tools::tool_registry::ToolRegistry;
use crate::agent::actions::AgentAction;
use crate::config::MalformedJsonHandling;
use crate::agent::core::prompt_builder::PromptBuilder;
use crate::agent::core::response_parser::ResponseParser;
use crate::logging::Logger;

pub struct BasicAgent {
    definition: AgentDefinition,
    tool_registry: ToolRegistry,
    prompt_builder: PromptBuilder,
    response_parser: ResponseParser,
}

impl BasicAgent {
    pub fn new(definition: AgentDefinition, tool_registry: ToolRegistry, prompt_builder: PromptBuilder, response_parser: ResponseParser) -> Result<Self> {
        Ok(Self { definition, tool_registry, prompt_builder, response_parser })
    }

    async fn _handle_tool_calls(
        &self,
        messages: &mut Vec<ChatMessage>,
        response: &mut LlmResponse,
        final_response_parts: &mut Vec<String>,
        mut logger: &mut Logger,
    ) -> Result<()> {
        // Loop for tool calls
        for iteration_count in 0..5 { // Max 5 tool calls to prevent infinite loops
            info!("Agent '{}' - Iteration {}", self.name(), iteration_count);

            let parsed_actions = self.response_parser.parse_response(&response.content, &self.definition.llm_config.on_malformed_json)?;

            let mut has_tool_call = false;
            for action in parsed_actions {
                match action {
                    AgentAction::ToolCall(tool_call) => {
                        has_tool_call = true;
                        final_response_parts.push(format!("[TOOL_CALL]: {}\n```json\n{}\n```", tool_call.name, serde_json::to_string_pretty(&tool_call)?));

                                                                        let tool_output = self.tool_registry.execute_tool(&tool_call.name, &tool_call.args, &mut logger).await?;
                        let tool_output_str = serde_json::to_string_pretty(&tool_output)?;

                        messages.push(ChatMessage { role: "assistant".to_string(), content: response.content.clone() });
                        messages.push(ChatMessage { role: "tool".to_string(), content: tool_output_str.clone() });

                        *response = crate::llm::llm_client::generate_response(&self.definition.llm_config, messages.clone(), &mut logger).await?;
                        
                        // Explicitly extract and report transformed_text for echo tool
                        if tool_call.name == "echo" {
                            if let Ok(output_val) = serde_json::from_str::<Value>(&tool_output_str) {
                                if let Some(transformed_text) = output_val["transformed_text"].as_str() {
                                    final_response_parts.push(format!("Echo Tool Output: {}", transformed_text));
                                } else {
                                    final_response_parts.push(format!("Echo Tool Output (raw): {}", tool_output_str));
                                }
                            } else {
                                final_response_parts.push(format!("Echo Tool Output (raw): {}", tool_output_str));
                            }
                        } else {
                            final_response_parts.push(format!("Tool output: {}", tool_output_str));
                        }
                    },
                    AgentAction::TextResponse { content } => {
                        final_response_parts.push(format!("[RESPONSE]: {}", content));
                    },
                    AgentAction::Thought { content } => {
                        info!("Agent '{}' thought: {}", self.name(), content);
                        final_response_parts.push(format!("[THOUGHT]: {}", content));
                    },
                }
            }

            if !has_tool_call {
                break; // No tool call detected, or all actions processed
            }
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
        let mut logger = Logger::new("vespe.log");

        let tool_prompt = self.prompt_builder.build_system_prompt(&self.definition, &self.tool_registry).await?;

        let mut messages = vec![
            ChatMessage { role: "system".to_string(), content: format!("You are a helpful AI assistant. {}\n", tool_prompt) },
            ChatMessage { role: "user".to_string(), content: input.to_string() },
        ];

        let mut response = crate::llm::llm_client::generate_response(&self.definition.llm_config, messages.clone()).await?;
        let mut final_response_parts = Vec::new();

        self._handle_tool_calls(&mut messages, &mut response, &mut final_response_parts, &mut logger).await?;

        let final_response_content = final_response_parts.join("\n");
        info!("Agent '{}' received final response: '{}'", self.name(), final_response_content);
        Ok(final_response_content)
    }
}

fn parse_llm_response(response_content: &str, handling: &MalformedJsonHandling) -> Result<Vec<AgentAction>> {
    // Try to parse as a Vec<AgentAction>
    if let Ok(actions) = serde_json::from_str::<Vec<AgentAction>>(response_content) {
        return Ok(actions);
    }

    // If parsing fails, handle based on configuration
    match handling {
        MalformedJsonHandling::TreatAsText => {
            info!("Malformed JSON, treating as text: {}", response_content);
            Ok(vec![AgentAction::TextResponse { content: response_content.to_string() }])
        },
        MalformedJsonHandling::Error => {
            Err(anyhow!("LLM response is not valid JSON or does not match expected action format: {}", response_content))
        },
    }
}
