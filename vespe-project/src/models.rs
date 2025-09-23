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
    Replanned,
}

impl TaskState {
    pub fn can_transition_to(self, next_state: TaskState) -> bool {
        match self {
            TaskState::Created => matches!(next_state, TaskState::ObjectiveDefined | TaskState::Failed | TaskState::Aborted),
            TaskState::ObjectiveDefined => matches!(next_state, TaskState::PlanDefined | TaskState::Failed | TaskState::Aborted | TaskState::NeedsReview),
            TaskState::PlanDefined => matches!(next_state, TaskState::Executing | TaskState::Failed | TaskState::Aborted | TaskState::NeedsReview),
            TaskState::Executing => matches!(next_state, TaskState::WaitingForSubtasks | TaskState::Completed | TaskState::Failed | TaskState::Aborted | TaskState::NeedsReview),
            TaskState::WaitingForSubtasks => matches!(next_state, TaskState::Executing | TaskState::Completed | TaskState::Failed | TaskState::Aborted | TaskState::NeedsReview),
            TaskState::NeedsReview => matches!(next_state, TaskState::ObjectiveDefined | TaskState::PlanDefined | TaskState::Executing | TaskState::Failed | TaskState::Aborted | TaskState::Completed),
            TaskState::Completed | TaskState::Failed | TaskState::Aborted | TaskState::Replanned => false, // Final states, no transitions out
        }
    }
}

impl std::fmt::Display for TaskState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// Corrisponde a config.json
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskConfig {
    pub uid: String,
    pub name: String,
    pub created_by_agent_uid: String, // Riferimento all'UID dell'Agente
    pub created_at: DateTime<Utc>,
    pub parent_uid: Option<String>, // UID del task genitore, se è un subtask
}

// Corrisponde a status.json
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskStatus {
    pub current_state: TaskState,
    pub last_updated_at: DateTime<Utc>,
    pub progress: Option<String>, // Es. "50% completato"
    pub parent_content_hashes: HashMap<String, String>, // Key: UID_dipendenza, Value: hash_contenuto_result
}

// Corrisponde a dependencies.json
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskDependencies {
    pub depends_on: Vec<String>, // Lista di UID dei task da cui dipende
}

// Rappresenta un task completo caricato in memoria
#[derive(Debug, Serialize, Deserialize, Clone)]
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
