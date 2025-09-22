use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// Corresponds to project_config.json
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectConfig {
    pub name: Option<String>,
    pub description: Option<String>,
    pub default_agent_uid: Option<String>,
    // pub manager_agent_config: Option<serde_json::Value>, // Future: specific config for Manager Agent
}

impl Default for ProjectConfig {
    fn default() -> Self {
        ProjectConfig {
            name: None,
            description: None,
            default_agent_uid: None,
        }
    }
}
