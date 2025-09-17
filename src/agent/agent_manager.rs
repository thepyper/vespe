use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::fs;

use crate::agent::agent_trait::Agent;
use crate::agent::impls::basic_agent::BasicAgent;
use crate::agent::models::AgentDefinition;
use crate::tools::tool_registry::ToolRegistry;
use crate::agent::core::prompt_builder::PromptBuilder;
use crate::agent::core::response_parser::ResponseParser;
use crate::prompt_templating::PromptTemplater;

pub struct AgentManager {
    project_root: PathBuf,
    tool_registry: ToolRegistry,
    prompt_templater: PromptTemplater<'static>, // Use 'static lifetime for PromptTemplater
}

impl AgentManager {
    pub fn new(project_root: PathBuf, tool_registry: ToolRegistry) -> Result<Self> {
        let prompt_templater = PromptTemplater::new(project_root.join(".vespe").join("prompts"))?;
        Ok(Self { project_root, tool_registry, prompt_templater })
    }

    pub async fn load_agent_definition(&self, name: &str) -> Result<AgentDefinition> {
        let agent_dir = self.project_root.join(".vespe").join("agents");
        let agent_path = agent_dir.join(format!("{}.json", name)); // Assuming JSON for now

        let content = fs::read_to_string(&agent_path)
            .await
            .with_context(|| format!("Failed to read agent definition from {:?}", agent_path))?;
        let definition: AgentDefinition = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse agent definition from {:?}", agent_path))?;

        Ok(definition)
    }

    pub fn create_agent(&self, definition: AgentDefinition) -> Result<Box<dyn Agent>> {
        let prompt_builder = PromptBuilder::new(self.project_root.clone());
        let response_parser = ResponseParser::new();
        let agent = BasicAgent::new(definition, self.tool_registry.clone(), prompt_builder, response_parser)?;
        Ok(Box::new(agent))
    }
}
