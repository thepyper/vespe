use anyhow::{Context, Result};
use async_trait::async_trait;
use serde_json::Value;
use tracing::info;

use crate::agent::agent_trait::Agent;
use crate::agent::models::AgentDefinition;
use crate::llm::llm_client::{GenericLlmClient, LlmClient};
use crate::llm::models::ChatMessage;
use crate::tools::tool_registry::ToolRegistry;

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
                "\n\nAvailable tools:\n{}\n\nTo use a tool, respond with a JSON object like this:\n{{\"tool_call\": {{ \"name\": \"tool_name\", \"args\": {{...}} }}}}}",
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
            ChatMessage { role: "system".to_string(), content: format!("You are a helpful AI assistant. If you use a tool, always report its output to the user.\n{}", tool_prompt) },
            ChatMessage { role: "user".to_string(), content: input.to_string() },
        ];

        let mut response = self.llm_client.generate_response(messages.clone()).await?;
        let mut final_response_content = response.content.clone();

        // Loop for tool calls
        for _ in 0..5 { // Max 5 tool calls to prevent infinite loops
            if let Ok(tool_call) = serde_json::from_str::<Value>(&response.content)
                .and_then(|v| Ok(v["tool_call"].clone()))
            {
                if tool_call.is_object() {
                    let tool_name = tool_call["name"].as_str().context("Tool call missing name")?;
                    let tool_args = tool_call["args"].clone();

                    info!("Agent '{}' calling tool: {} with args: {}", self.name(), tool_name, tool_args);

                    let tool_output = self.tool_registry.execute_tool(tool_name, &tool_args).await?;
                    let tool_output_str = serde_json::to_string_pretty(&tool_output)?;

                    messages.push(ChatMessage { role: "assistant".to_string(), content: response.content.clone() });
                    messages.push(ChatMessage { role: "tool".to_string(), content: tool_output_str.clone() });

                    response = self.llm_client.generate_response(messages.clone()).await?;
                    final_response_content = format!("Tool output: {{}}\nLLM response: {{}}", tool_output_str, response.content);
                } else {
                    break; // Not a valid tool call format
                }
            } else {
                break; // No tool call detected
            }
        }

        info!("Agent '{}' received final response: '{}'", self.name(), final_response_content);
        Ok(final_response_content)
    }
}