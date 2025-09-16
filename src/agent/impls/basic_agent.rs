use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use serde_json::Value;
use tracing::info;

use crate::agent::agent_trait::Agent;
use crate::agent::models::AgentDefinition;
use crate::llm::llm_client::{GenericLlmClient, LlmClient};
use crate::llm::models::ChatMessage;
use crate::tools::tool_registry::ToolRegistry;
use crate::agent::actions::{AgentAction, ToolCall};
use crate::config::models::{LlmConfig, MalformedJsonHandling};

pub struct BasicAgent {
    definition: AgentDefinition,
    llm_client: GenericLlmClient,
    tool_registry: ToolRegistry,
}

impl BasicAgent {
    pub fn new(definition: AgentDefinition, tool_registry: ToolRegistry) -> Result<Self> {
        let llm_client = GenericLlmClient::new(definition.llm_config.clone())?;
        Ok(Self { definition, llm_client, tool_registry })
    }

    fn get_tool_prompt(&self) -> String {
        if let Some(tool_names) = &self.definition.tools {
            let available_tools: Vec<Value> = self.tool_registry.get_tool_metadata().into_iter()
                .filter(|tool_meta| tool_names.iter().any(|name| name == tool_meta["name"].as_str().unwrap_or("")))
                .collect();

            if available_tools.is_empty() {
                return String::new();
            }

            format!(
                "\n\nAvailable tools:\n{}\n\nYour response should be a JSON object representing an action or a response. It must have a \"tool_call\" key for tool calls, or \"text_response\" for direct text, or \"thought\" for internal thoughts. If you use a tool, the \"tool_call\" object must contain \"name\" (string) and \"args\" (object). If you have multiple actions, respond with a JSON array of these objects.\n",
                serde_json::to_string_pretty(&available_tools).unwrap_or_default()
            )
        } else {
            String::new()
        }
    }
}

#[async_trait]
impl Agent for BasicAgent {
    fn name(&self) -> &str {
        &self.definition.name
    }

    async fn execute(&self, input: &str) -> Result<String> {
        info!("Agent '{}' executing with input: '{}'", self.name(), input);

        let tool_prompt = self.get_tool_prompt();

        let mut messages = vec![
            ChatMessage { role: "system".to_string(), content: format!("You are a helpful AI assistant. {}\n", tool_prompt) },
            ChatMessage { role: "user".to_string(), content: input.to_string() },
        ];

        let mut response = self.llm_client.generate_response(messages.clone()).await?;
        let mut final_response_content = String::new();

        // Loop for tool calls
        for _ in 0..5 { // Max 5 tool calls to prevent infinite loops
            let parsed_actions = parse_llm_response(&response.content, &self.definition.llm_config.on_malformed_json)?;

            let mut has_tool_call = false;
            for action in parsed_actions {
                match action {
                    AgentAction::ToolCall(tool_call) => {
                        has_tool_call = true;
                        info!("Agent '{}' calling tool: {} with args: {:?}", self.name(), tool_call.name, tool_call.args);

                        let tool_output = self.tool_registry.execute_tool(&tool_call.name, &tool_call.args).await?;
                        let tool_output_str = serde_json::to_string_pretty(&tool_output)?;

                        messages.push(ChatMessage { role: "assistant".to_string(), content: response.content.clone() });
                        messages.push(ChatMessage { role: "tool".to_string(), content: tool_output_str.clone() });

                        response = self.llm_client.generate_response(messages.clone()).await?;
                        final_response_content = format!("Tool output: {}\nLLM response: {}", tool_output_str, response.content);
                    },
                    AgentAction::TextResponse { content } => {
                        final_response_content = content;
                    },
                    AgentAction::Thought { content } => {
                        info!("Agent '{}' thought: {}", self.name(), content);
                    },
                }
            }

            if !has_tool_call {
                break; // No tool call detected, or all actions processed
            }
        }

        info!("Agent '{}' received final response: '{}'", self.name(), final_response_content);
        Ok(final_response_content)
    }
}

fn parse_llm_response(response_content: &str, handling: &MalformedJsonHandling) -> Result<Vec<AgentAction>> {
    // Try to parse as a Vec<AgentAction>
    if let Ok(actions) = serde_json::from_str::<Vec<AgentAction>>(response_content) {
        return Ok(actions);
    }

    // Try to parse as a single AgentAction
    if let Ok(action) = serde_json::from_str::<AgentAction>(response_content) {
        return Ok(vec![action]);
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
