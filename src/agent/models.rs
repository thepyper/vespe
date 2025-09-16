use serde::{Deserialize, Serialize};

use crate::config::LlmConfig;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentDefinition {
    pub name: String,
    pub llm_config: LlmConfig,
    pub tools: Option<Vec<String>>, // List of tool names this agent can use
}
