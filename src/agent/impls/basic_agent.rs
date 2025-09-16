use anyhow::Result;
use async_trait::async_trait;
use tracing::info;

use crate::agent::agent_trait::Agent;
use crate::agent::models::AgentDefinition;
use crate::llm::llm_client::{GenericLlmClient, LlmClient};
use crate::llm::models::ChatMessage;

pub struct BasicAgent {
    definition: AgentDefinition,
    llm_client: GenericLlmClient,
}

impl BasicAgent {
    pub fn new(definition: AgentDefinition) -> Result<Self> {
        let llm_client = GenericLlmClient::new(definition.llm_config.clone())?;
        Ok(Self { definition, llm_client })
    }
}

#[async_trait]
impl Agent for BasicAgent {
    fn name(&self) -> &str {
        &self.definition.name
    }

    async fn execute(&self, input: &str) -> Result<String> {
        info!("Agent '{}' executing with input: '{}'", self.name(), input);

        let messages = vec![
            ChatMessage { role: "user".to_string(), content: input.to_string() },
        ];

        let response = self.llm_client.generate_response(messages).await?;

        info!("Agent '{}' received response: '{}'", self.name(), response.content);
        Ok(response.content)
    }
}
