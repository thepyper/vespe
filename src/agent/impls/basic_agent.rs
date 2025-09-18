use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use tracing::info;

use crate::agent::agent_trait::Agent;
use crate::agent::models::AgentDefinition;
use crate::llm::llm_client::LlmClient;
use crate::llm::messages::{Message, AssistantContent};
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
}

#[async_trait]
impl Agent for BasicAgent {
    fn name(&self) -> &str {
        &self.definition.name
    }

    async fn execute(&self, input: &str) -> Result<String> {
        info!("Agent '{}' executing with input: '{}'", self.name(), input);
        
        // Phase 0: Drastically simplified execute method.
        // The old logic for prompts, tool calls, and agent loops has been removed.
        // This will be rebuilt in the next phases with the new parsing logic.

        let system_prompt = "You are a helpful assistant.".to_string();

        let mut messages = vec![
            Message::System(system_prompt),
            Message::User(input.to_string()),
        ];

        let assistant_response = self.llm_client.generate_response(&messages).await?;

        // For now, just concatenate the text parts of the response.
        let final_response = assistant_response.into_iter().filter_map(|content| {
            if let AssistantContent::Text(text) = content {
                Some(text)
            } else {
                None
            }
        }).collect::<Vec<_>>().join("
");

        info!("Agent '{}' received final response: '{}'", self.name(), final_response);
        Ok(final_response)
    }
}
