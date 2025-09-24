use std::collections::HashMap;
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use crate::error::ProjectError;
use crate::utils::{write_file_content, update_task_status, write_json_file};
use uuid::Uuid;
use sha2::{Sha256, Digest};
use walkdir;

impl Agent {
    /// Creates a new agent (AI or human).
    pub fn create(
        agent_type: AgentType,
        name: String,
        agents_base_path: &Path,
    ) -> Result<Self, ProjectError> {
        let uid_prefix = match agent_type {
            AgentType::Human => "usr", // Or "human"
            AgentType::AI => "agt",
        };
        let uid = crate::utils::generate_uid(uid_prefix)?;
        let agent_path = crate::utils::get_entity_path(agents_base_path, &uid)?;

        std::fs::create_dir_all(&agent_path).map_err(|e| ProjectError::Io(e))?;

        let now = chrono::Utc::now();

        let agent_config = Agent {
            uid: uid.clone(),
            name: name.clone(),
            agent_type,
            created_at: now,
            parent_agent_uid: None,
            model_id: None,
            temperature: None,
            top_p: None,
            default_tools: None,
            context_strategy: None,
        };
        crate::utils::write_json_file(&agent_path.join("config.json"), &agent_config)?;

        Ok(agent_config)
    }
}

// Struttura per gli eventi persistenti (da persistent/)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PersistentEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: String, // Es. "llm_response", "tool_call", "agent_decision"
    pub acting_agent_uid: String, // Riferimento all'UID dell'Agente che ha generato l'evento
    pub content: String, // Contenuto dell'evento (es. prompt, output tool)
    // Altri metadati specifici dell'evento
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum AgentType {
    Human,
    AI,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Agent {
    pub uid: String, // Unique ID for the agent (e.g., "usr-pyper", "agt-manager-v1")
    pub name: String, // Display name
    pub agent_type: AgentType,
    pub created_at: DateTime<Utc>,
    // Campi specifici per AI (opzionali)
    pub parent_agent_uid: Option<String>,
    pub model_id: Option<String>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub default_tools: Option<Vec<String>>, // UIDs of tools
    pub context_strategy: Option<String>,
    // Campi specifici per Human (opzionali)
    // pub user_preferences: Option<UserPreferences>, // Placeholder for future user-specific settings
}
