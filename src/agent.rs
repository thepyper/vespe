use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::path::{Path, PathBuf};

use crate::memory::{Memory, Message};
use crate::error::ProjectError;
use crate::utils::{generate_uid, get_entity_path, read_json_file, write_json_file, write_file_content, read_file_content};

// Default protocol name for agents
const DEFAULT_AGENT_PROTOCOL_NAME: &str = "default_protocol";

fn default_protocol_name() -> String {
    DEFAULT_AGENT_PROTOCOL_NAME.to_string()
}

// 1. METADATA COMUNI: Dati condivisi da tutti gli agenti.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentMetadata {
    pub uid: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub parent_uid: Option<String>,
    #[serde(default = "default_protocol_name")]
    pub protocol_name: String,
}

// 2. DETTAGLI SPECIFICI: L'enum che modella la differenza chiave.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AgentDetails {
    AI(AIConfig),
    Human(HumanConfig),
}

// 3. CONFIGURAZIONE AI: Campi specifici per l'AI.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AIConfig {
    pub role: String,
    pub llm_provider: LLMProviderConfig,
    pub allowed_tools: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum LLMProviderConfig {
    Ollama { model: String, endpoint: String },
    OpenAI { model: String, api_key_env: String },
}

// 4. CONFIGURAZIONE HUMAN: Campi specifici per l'umano.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct HumanConfig { /* ...email, permissions, etc. */ }

// 5. STATO DINAMICO DELL'AGENTE (semplificato)
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AgentState {
    pub last_seen_at: DateTime<Utc>,
}

// L'AGENTE COMPLETO: Oggetto costruito in memoria.
#[derive(Debug, Clone)]
pub struct Agent {
    pub metadata: AgentMetadata,
    pub details: AgentDetails,
    pub state: AgentState,
    pub memory: Memory,
    pub system_prompt: Option<String>,
}

impl Agent {
    pub fn create_ai(
        project_root: &Path,
        name: String,
        config: AIConfig,
        system_prompt: Option<String>,
        protocol_name: Option<String>,
    ) -> Result<Agent, ProjectError> {
        let uid = generate_uid("agt")?;
        let agents_base_path = project_root.join(".vespe").join("agents");
        let agent_path = get_entity_path(&agents_base_path, &uid)?;

        std::fs::create_dir_all(&agent_path).map_err(|e| ProjectError::Io(e))?;
        std::fs::create_dir_all(agent_path.join("memory")).map_err(|e| ProjectError::Io(e))?;

        if let Some(prompt) = system_prompt {
            write_file_content(&agent_path.join("system_prompt.hbs"), &prompt)?;
        }

        let now = Utc::now();

        let metadata = AgentMetadata {
            uid: uid.clone(),
            name: name.clone(),
            created_at: now,
            parent_uid: None,
            protocol_name: protocol_name.unwrap_or_else(default_protocol_name),
        };
        write_json_file(&agent_path.join("metadata.json"), &metadata)?;

        let details = AgentDetails::AI(config);
        write_json_file(&agent_path.join("details.json"), &details)?;

        let state = AgentState::default();
        write_json_file(&agent_path.join("state.json"), &state)?;

        Agent::load(project_root, &uid)
    }

    pub fn create_human(
        project_root: &Path,
        name: String,
        config: HumanConfig,
        protocol_name: Option<String>,
    ) -> Result<Agent, ProjectError> {
        let uid = generate_uid("usr")?;
        let agents_base_path = project_root.join(".vespe").join("agents");
        let agent_path = get_entity_path(&agents_base_path, &uid)?;

        std::fs::create_dir_all(&agent_path).map_err(|e| ProjectError::Io(e))?;
        std::fs::create_dir_all(agent_path.join("memory")).map_err(|e| ProjectError::Io(e))?;

        let now = Utc::now();

        let metadata = AgentMetadata {
            uid: uid.clone(),
            name: name.clone(),
            created_at: now,
            parent_uid: None,
            protocol_name: protocol_name.unwrap_or_else(default_protocol_name),
        };
        write_json_file(&agent_path.join("metadata.json"), &metadata)?;

        let details = AgentDetails::Human(config);
        write_json_file(&agent_path.join("details.json"), &details)?;

        let state = AgentState::default();
        write_json_file(&agent_path.join("state.json"), &state)?;

        Agent::load(project_root, &uid)
    }

    pub fn load(project_root: &Path, agent_uid: &str) -> Result<Self, ProjectError> {
        let agents_base_path = project_root.join(".vespe").join("agents");
        let agent_path = get_entity_path(&agents_base_path, agent_uid)?;

        if !agent_path.exists() {
            return Err(ProjectError::AgentNotFound(agent_uid.to_string()));
        }

        let metadata: AgentMetadata = read_json_file(&agent_path.join("metadata.json"))?;
        let details: AgentDetails = read_json_file(&agent_path.join("details.json"))?;
        let state: AgentState = read_json_file(&agent_path.join("state.json"))?;
        let memory = Memory::load(&agent_path.join("memory")).map_err(|e| ProjectError::Memory(e))?;

        let system_prompt_path = agent_path.join("system_prompt.hbs");
        let system_prompt = if system_prompt_path.exists() {
            Some(read_file_content(&system_prompt_path)?)
        } else {
            None
        };

        Ok(Agent {
            metadata,
            details,
            state,
            memory,
            system_prompt,
        })
    }
    pub fn save_state(&self, project_root: &Path) -> Result<(), ProjectError> {
        let agents_base_path = project_root.join(".vespe").join("agents");
        let agent_path = get_entity_path(&agents_base_path, &self.metadata.uid)?;
        write_json_file(&agent_path.join("state.json"), &self.state)?;
        Ok(())
    }
}