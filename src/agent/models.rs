use serde::{Deserialize, Serialize};

use crate::config::models::LlmConfig;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentDefinition {
    pub name: String,
    pub llm_config: LlmConfig,
    // Add other agent-specific fields as needed later
}
