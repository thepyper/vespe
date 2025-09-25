use std::collections::HashMap;
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use crate::error::ProjectError;
use crate::utils::{write_file_content, update_task_status, write_json_file};
use uuid::Uuid;
use sha2::{Sha256, Digest};
use walkdir;
use crate::PersistentEvent;

// Rappresenta lo stato attuale del task
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum TaskState {
    Created,
    ObjectiveDefined,
    PlanDefined,
    Delegating,
    Harvesting,
    Working,
    Error,
    Failed,
    Completed,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum TaskType {
    Monolithic,
    Subdivided,
}

// Corrisponde a config.json
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskConfig {
    pub uid: String,
    pub name: String,
    pub created_by_agent_uid: String, // Riferimento all'UID dell'Agente
    pub created_at: DateTime<Utc>,
    pub parent_uid: Option<String>, // UID del task genitore, se è un subtask
    pub task_type: Option<TaskType>,
}

// Corrisponde a status.json
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskStatus {
    pub current_state: TaskState,
    pub last_updated_at: DateTime<Utc>,
    pub progress: Option<String>, // Es. "50% completato"
    pub parent_content_hashes: HashMap<String, String>, // Key: UID_dipendenza, Value: hash_contenuto_result
    pub is_paused: bool,
    pub error_details: Option<String>,
    pub previous_state: Option<TaskState>,
    pub retry_count: u8,
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
    pub subtasks: HashMap<String, TaskState>,
}

impl Task {
    /// Loads a task from the filesystem given its UID.
    pub fn load(
        project_root: &Path,
        uid: &str
    ) -> Result<Self, ProjectError> {
