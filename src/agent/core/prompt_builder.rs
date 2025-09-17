use anyhow::{Context, Result};
use serde_json::Value;
use std::path::PathBuf;
use tokio::fs;

use crate::agent::models::AgentDefinition;
use crate::tools::tool_registry::ToolRegistry;
use crate::prompt_templating::PromptTemplater;

pub struct PromptBuilder {
    prompt_templater: PromptTemplater,
}

impl PromptBuilder {
    pub fn new(prompt_templater: PromptTemplater) -> Self {
        Self { prompt_templater }
    }

    pub async fn build_system_prompt(&self, agent_definition: &AgentDefinition, tool_registry: &ToolRegistry) -> Result<String> {
        let tool_prompt_part = if let Some(tool_names) = &agent_definition.tools {
            let available_tools: Vec<Value> = tool_registry.get_tool_metadata().into_iter()
                .filter(|tool_meta| tool_names.iter().any(|name| name == tool_meta["name"].as_str().unwrap_or("")))
                .collect();

            if available_tools.is_empty() {
                String::new()
            } else {
                serde_json::to_string_pretty(&available_tools).unwrap_or_default()
            }
        } else {
            String::new()
        };

        let mut data = serde_json::Map::new();
        data.insert("tool_prompt".to_string(), Value::String(tool_prompt_part));

        self.prompt_templater.render_prompt("system_prompt", &Value::Object(data))
    }
}