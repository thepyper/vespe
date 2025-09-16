use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: String,
    pub model_id: String,
    pub api_key: Option<String>,
    // Add other LLM-specific parameters as needed
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
}

fn default_temperature() -> f32 { 0.7 }
fn default_max_tokens() -> u32 { 512 }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentDefinition {
    pub name: String,
    pub llm_config: LlmConfig,
    // Add other agent-specific fields as needed later
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub default_llm_config: LlmConfig,
    // Add other global settings as needed
}
