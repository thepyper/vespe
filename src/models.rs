use std::collections::HashMap;
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use crate::error::ProjectError;
use crate::utils::{write_file_content, update_task_status, write_json_file};
use uuid::Uuid;
use sha2::{Sha256, Digest};
use walkdir;

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

impl Task {
    /// Transitions from `CREATED` to `OBJECTIVE_DEFINED`.
    /// Writes the objective content to `objective.md`.
    pub fn define_objective(&mut self, objective_content: String) -> Result<(), ProjectError> {
        // Update objective.md
        write_file_content(&self.root_path.join("objective.md"), &objective_content)?;
        self.objective = objective_content;

        // Update status
        update_task_status(&self.root_path, TaskState::ObjectiveDefined, &mut self.status)?;

        Ok(())
    }

    /// Transitions from `OBJECTIVE_DEFINED` to `PLAN_DEFINED`.
    /// Writes the plan content to `plan.md`.
    pub fn define_plan(&mut self, plan_content: String) -> Result<(), ProjectError> {
        // Prevent defining a plan for a replanned task
        if self.status.current_state == TaskState::Replanned {
            return Err(ProjectError::InvalidStateTransition(
                self.status.current_state,
                TaskState::PlanDefined, // Attempted target state
            ));
        }

        // Update plan.md
        write_file_content(&self.root_path.join("plan.md"), &plan_content)?;
        self.plan = Some(plan_content);

        // Update status
        update_task_status(&self.root_path, TaskState::PlanDefined, &mut self.status)?;

        Ok(())
    }

    /// Adds a new event to the `persistent/` folder of the task.
    pub fn add_persistent_event(&self, event: PersistentEvent) -> Result<(), ProjectError> {
        let persistent_path = self.root_path.join("persistent");

        // Use UUID for filename to guarantee uniqueness, append timestamp for sorting
        let filename = format!("{}_{}_{}.json", event.timestamp.format("%Y%m%d%H%M%S%3f"), Uuid::new_v4().as_simple(), event.event_type);
        let file_path = persistent_path.join(filename);

        write_json_file(&file_path, &event)?;

        Ok(())
    }

    /// Retrieves all persistent events for a task, sorted by timestamp.
    pub fn get_all_persistent_events(&self) -> Result<Vec<PersistentEvent>, ProjectError> {
        let persistent_path = self.root_path.join("persistent");

        if !persistent_path.exists() {
            return Ok(Vec::new());
        }

        let mut events = Vec::new();
        for entry in std::fs::read_dir(&persistent_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                let event: PersistentEvent = crate::utils::read_json_file(&path)?;
                events.push(event);
            }
        }

        events.sort_by_key(|event| event.timestamp);

        Ok(events)
    }

    /// Calculates the SHA256 hash of the `result/` folder content for a task.
    /// The hash is based on the content of all files and their relative paths within the folder.
    pub fn calculate_result_hash(&self) -> Result<String, ProjectError> {
        let result_path = self.root_path.join("result");

        if !result_path.exists() {
            return Ok(format!("{:x}", Sha256::new().finalize())); // Hash of empty content
        }

        let mut hasher = Sha256::new();
        let mut file_hashes: Vec<(String, String)> = Vec::new(); // (relative_path, hash)

        for entry in walkdir::WalkDir::new(&result_path) {
            let entry = entry.map_err(|e| ProjectError::ContentHashError(result_path.clone(), e.to_string()))?;
            let path = entry.path();

            if path.is_file() {
                let relative_path = path.strip_prefix(&result_path)
                    .map_err(|_e| ProjectError::InvalidPath(path.to_path_buf()))?
                    .to_string_lossy()
                    .into_owned();
                let file_hash = crate::utils::hash_file(path)?;
                file_hashes.push((relative_path, file_hash));
            }
        }

        // Sort to ensure canonical representation regardless of filesystem iteration order
        file_hashes.sort_by(|a, b| a.0.cmp(&b.0));

        for (relative_path, file_hash) in file_hashes {
            hasher.update(relative_path.as_bytes());
            hasher.update(file_hash.as_bytes());
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Adds a file to the `result/` folder of the task.
    pub fn add_result_file(
        &self,
        filename: &str,
        content: Vec<u8>
    ) -> Result<(), ProjectError> {
        let result_path = self.root_path.join("result");

        // Ensure the result directory exists
        std::fs::create_dir_all(&result_path).map_err(|e| ProjectError::Io(e))?;

        let file_path = result_path.join(filename);
        std::fs::write(&file_path, content).map_err(|e| ProjectError::Io(e))?;

        Ok(())
    }

    /// Reviews a task, transitioning it to Completed (approved) or Replanned (rejected).
    pub fn review_task(
        &mut self,
        approved: bool, // true for approve, false for reject
    ) -> Result<(), ProjectError> {
        if self.status.current_state != TaskState::NeedsReview {
            return Err(ProjectError::InvalidStateTransition(
                self.status.current_state,
                TaskState::NeedsReview, // This is not quite right, should be the target state
            ));
        }

        let next_state = if approved {
            TaskState::Completed // Or TaskState::Ready if we add it
        } else {
            TaskState::Replanned
        };

        if !self.status.current_state.can_transition_to(next_state) {
            return Err(ProjectError::InvalidStateTransition(
                self.status.current_state,
                next_state,
            ));
        }

        update_task_status(&self.root_path, next_state, &mut self.status)?;

        Ok(())
    }

    /// Loads a task from the filesystem given its UID.
    pub fn load(
        project_root: &Path,
        uid: &str
    ) -> Result<Self, ProjectError> {
        let tasks_base_path = project_root.join(".vespe").join("tasks"); // Re-construct tasks_base_path
        let task_path = crate::utils::get_entity_path(&tasks_base_path, uid)?;

        if !task_path.exists() {
            return Err(ProjectError::TaskNotFound(uid.to_string()));
        }

        let config: TaskConfig = crate::utils::read_json_file(&task_path.join("config.json"))?;
        let status: TaskStatus = crate::utils::read_json_file(&task_path.join("status.json"))?;
        let dependencies: TaskDependencies = crate::utils::read_json_file(&task_path.join("dependencies.json"))?;
        let objective = crate::utils::read_file_content(&task_path.join("objective.md"))?;
        let plan = Some(crate::utils::read_file_content(&task_path.join("plan.md"))?);

        Ok(Task {
            uid: uid.to_string(),
            root_path: task_path,
            config,
            status,
            objective,
            plan,
            dependencies,
        })
    }
}

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
