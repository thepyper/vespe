use anyhow::Result;
use serde_json::Value;

use crate::agent::models::AgentDefinition;
use crate::tools::tool_registry::ToolRegistry;

pub struct PromptBuilder {
    // Potentially store common prompt parts or templates here
}

impl PromptBuilder {
    pub fn new() -> Self {
        Self {}
    }

    pub fn build_system_prompt(&self, agent_definition: &AgentDefinition, tool_registry: &ToolRegistry) -> String {
        let tool_prompt = if let Some(tool_names) = &agent_definition.tools {
            let available_tools: Vec<Value> = tool_registry.get_tool_metadata().into_iter()
                .filter(|tool_meta| tool_names.iter().any(|name| name == tool_meta["name"].as_str().unwrap_or("")))
                .collect();

            if available_tools.is_empty() {
                String::new()
            } else {
                format!(
                    "\n\nAvailable tools:\n{}\n\nTo use a tool, respond with a JSON object where the key is \"tool_call\" and its value is an object with \"name\" (string) and \"args\" (object).",
                    serde_json::to_string_pretty(&available_tools).unwrap_or_default()
                )
            }
        } else {
            String::new()
        };

        format!("You are a helpful AI assistant. If you use a tool, always report its output to the user.\n{}", tool_prompt)
    }
}
