use std::path::PathBuf;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

// Rappresenta lo stato attuale del task
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum TaskState {
    Created,
    ObjectiveDefined,
    PlanDefined,
    Executing,
    WaitingForSubtasks,
    NeedsReview,
    Completed,
    Failed,
    Aborted,
}

// Corrisponde a config.json
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskConfig {
    pub uid: String,
    pub name: String,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub parent_uid: Option<String>, // UID del task genitore, se è un subtask
}

// Corrisponde a status.json
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskStatus {
    pub current_state: TaskState,
    pub last_updated_at: DateTime<Utc>,
    pub progress: Option<String>, // Es. "50% completato"
    pub parent_content_hashes: HashMap<String, String>, // Key: UID_dipendenza, Value: hash_contenuto_result
}

// Corrisponde a dependencies.json
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskDependencies {
    pub depends_on: Vec<String>, // Lista di UID dei task da cui dipende
}

// Rappresenta un task completo caricato in memoria
#[derive(Debug)]
pub struct Task {
    pub uid: String,
    pub root_path: PathBuf, // Percorso alla directory tsk-UID/
    pub config: TaskConfig,
    pub status: TaskStatus,
    pub objective: String, // Contenuto di objective.md
    pub plan: Option<String>, // Contenuto di plan.md
    pub dependencies: TaskDependencies,
    // Potrebbero esserci altri campi per subtask caricati, ecc.
}

// Struttura per gli eventi persistenti (da persistent/)
#[derive(Debug, Serialize, Deserialize)]
pub struct PersistentEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: String, // Es. "llm_response", "tool_call", "agent_decision"
    pub agent_id: String,
    pub content: String, // Contenuto dell'evento (es. prompt, output tool)
    // Altri metadati specifici dell'evento
}
