use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::path::{Path, PathBuf};

use crate::memory::{Memory, Message};

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
    // Placeholder for methods that will be implemented later
    // pub fn create_ai(name: String, config: AIConfig, agents_base_path: &Path) -> Result<Self, ProjectError> { unimplemented!() }
    // pub fn create_human(name: String, config: HumanConfig, agents_base_path: &Path) -> Result<Self, ProjectError> { unimplemented!() }
    // pub fn load(project_root: &Path, uid: &str) -> Result<Self, ProjectError> { unimplemented!() }
    // pub fn save_state(&self) -> Result<(), ProjectError> { unimplemented!() }
}