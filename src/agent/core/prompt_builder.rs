use anyhow::{Context, Result};
use serde_json::Value;
use std::path::PathBuf;
use tokio::fs;

use crate::agent::models::AgentDefinition;
use crate::tools::tool_registry::ToolRegistry;

pub struct PromptBuilder {
    prompt_templater: PromptTemplater,
}

impl PromptBuilder {
    pub fn new(project_root: PathBuf) -> Self {
        Self { project_root }
    }

    pub async fn build_system_prompt(&self, agent_definition: &AgentDefinition, tool_registry: &ToolRegistry) -> Result<String> {
        let prompt_path = self.project_root.join(".vespe").join("prompts").join("system_prompt_template.txt");
        let template_content = fs::read_to_string(&prompt_path)
            .await
            .with_context(|| format!("Failed to read system prompt template from {:?}", prompt_path))?;

        let tool_prompt_part = if let Some(tool_names) = &agent_definition.tools {
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

        Ok(template_content.replace("{}", &tool_prompt_part))
    }
}
}
}
