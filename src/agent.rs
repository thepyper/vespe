use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::path::{Path, PathBuf};

use crate::memory::{Memory, Message};
use crate::error::ProjectError;
use crate::utils::{generate_uid, get_entity_path, read_json_file, write_json_file};

// 1. METADATA COMUNI: Dati condivisi da tutti gli agenti.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentMetadata {
    pub uid: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub parent_uid: Option<String>,
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
}

impl Agent {
    pub fn create_ai(
        project_root: &Path,
        name: String,
        config: AIConfig,
    ) -> Result<Agent, ProjectError> {
        let uid = generate_uid("agt")?;
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

        Ok(Agent {
            metadata,
            details,
            state,
            memory,
        })
    }
    // pub fn save_state(&self) -> Result<(), ProjectError> { unimplemented!() }
}