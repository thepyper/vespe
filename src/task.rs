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
use tracing::{debug, error, warn};

// Rappresenta lo stato attuale del task
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum TaskState {
    Created,
    ObjectiveDefined,
    PlanDefined,
    Working,
    Error,
    Failed,
    Completed,
}

impl TaskState {
    pub fn can_transition_to(self, next_state: TaskState) -> bool {
        match self {
            TaskState::Created => matches!(next_state, TaskState::ObjectiveDefined | TaskState::Error | TaskState::Failed),
            TaskState::ObjectiveDefined => matches!(next_state, TaskState::ObjectiveDefined | TaskState::PlanDefined | TaskState::Error | TaskState::Failed),
            TaskState::PlanDefined => matches!(next_state, TaskState::PlanDefined | TaskState::Working | TaskState::ObjectiveDefined | TaskState::Error | TaskState::Failed),
            TaskState::Working => matches!(next_state, TaskState::Completed | TaskState::Error | TaskState::Failed),
            TaskState::Error => matches!(next_state, TaskState::Failed | TaskState::Error), // From Error, can transition to Failed or stay in Error
            TaskState::Failed | TaskState::Completed => false, // Final states, no transitions out
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
    pub progress: Option<String>,
    pub parent_content_hashes: HashMap<String, String>,
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
    pub subtask_uids: Vec<String>,
}

impl Task {
    pub fn define_objective(&mut self, objective_content: String) -> Result<(), ProjectError> {
        if !self.status.current_state.can_transition_to(TaskState::ObjectiveDefined) {
            return Err(ProjectError::InvalidStateTransition(
                self.status.current_state,
                TaskState::ObjectiveDefined,
            ));
        }

        write_file_content(&self.root_path.join("objective.md"), &objective_content)?;
        self.objective = objective_content;

        Ok(())
    }

    pub fn define_plan(&mut self, plan_content: String) -> Result<(), ProjectError> {
        if !self.status.current_state.can_transition_to(TaskState::PlanDefined) {
            return Err(ProjectError::InvalidStateTransition(
                self.status.current_state,
                TaskState::PlanDefined,
            ));
        }

        write_file_content(&self.root_path.join("plan.md"), &plan_content)?;
        self.plan = Some(plan_content);

        update_task_status(&self.root_path, TaskState::PlanDefined, &mut self.status)?;

        Ok(())
    }

    pub fn accept_plan(&mut self) -> Result<(), ProjectError> {
        if !self.status.current_state.can_transition_to(TaskState::Working) {
            return Err(ProjectError::InvalidStateTransition(
                self.status.current_state,
                TaskState::Working,
            ));
        }

        update_task_status(&self.root_path, TaskState::Working, &mut self.status)?;

        Ok(())
    }

    pub fn reject_plan(&mut self) -> Result<(), ProjectError> {
        if !self.status.current_state.can_transition_to(TaskState::ObjectiveDefined) {
            return Err(ProjectError::InvalidStateTransition(
                self.status.current_state,
                TaskState::ObjectiveDefined,
            ));
        }

        update_task_status(&self.root_path, TaskState::ObjectiveDefined, &mut self.status)?;

        Ok(())
    }



    pub fn error(&mut self, details: String, is_failure: bool) -> Result<(), ProjectError> {
        let next_state = if is_failure { TaskState::Failed } else { TaskState::Error };

        if !self.status.current_state.can_transition_to(next_state) {
            return Err(ProjectError::InvalidStateTransition(
                self.status.current_state,
                next_state,
            ));
        }

        self.status.previous_state = Some(self.status.current_state);
        self.status.error_details = Some(details);
        update_task_status(&self.root_path, next_state, &mut self.status)?;

        Ok(())
    }

    pub fn work_completed(&mut self) -> Result<(), ProjectError> {
        if !self.status.current_state.can_transition_to(TaskState::Completed) {
            return Err(ProjectError::InvalidStateTransition(
                self.status.current_state,
                TaskState::Completed,
            ));
        }

        update_task_status(&self.root_path, TaskState::Completed, &mut self.status)?;

        Ok(())
    }

    pub fn abort(&mut self, reason: String) -> Result<(), ProjectError> {
        if !self.status.current_state.can_transition_to(TaskState::Failed) {
            return Err(ProjectError::InvalidStateTransition(
                self.status.current_state,
                TaskState::Failed,
            ));
        }

        self.status.error_details = Some(format!("Aborted: {}", reason));
        update_task_status(&self.root_path, TaskState::Failed, &mut self.status)?;

        // TODO: Implement cascading abort for subtasks
        // This will require loading subtasks and calling abort on them.
        // For now, we just mark the parent as failed.

        Ok(())
    }

    pub fn pause_task(&mut self, reason: String) -> Result<(), ProjectError> {
        self.status.is_paused = true;
        // Optionally, store the reason in error_details or a new field
        // self.status.error_details = Some(format!("Paused: {}", reason));
        write_json_file(&self.root_path.join("status.json"), &self.status)?;
        Ok(())
    }

    pub fn resume_task(&mut self) -> Result<(), ProjectError> {
        self.status.is_paused = false;
        // Optionally, clear the reason if it was stored
        // self.status.error_details = None;
        write_json_file(&self.root_path.join("status.json"), &self.status)?;
        Ok(())
    }


    pub fn get_task_state(&self) -> TaskState {
        self.status.current_state
    }

    pub fn is_task_paused(&self) -> bool {
        self.status.is_paused
    }

    pub fn set_name(&mut self, new_name: String) -> Result<(), ProjectError> {
        self.config.name = new_name;
        write_json_file(&self.root_path.join("config.json"), &self.config)?;
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



    /// Loads a task from the filesystem given its UID.
    pub fn load(
        project_root: &Path,
        uid: &str
    ) -> Result<Self, ProjectError> {
        let tasks_base_path = project_root.join(".vespe").join("tasks"); // Re-construct tasks_base_path
        let task_path = crate::utils::get_entity_path(&tasks_base_path, uid)?;

        debug!("Attempting to load task from path: {:?}", task_path);

        if !task_path.exists() {
            error!("Task path does not exist: {:?}", task_path);
            return Err(ProjectError::TaskNotFound(uid.to_string()));
        }

        debug!("Loading config.json for task: {}", uid);
        let config: TaskConfig = crate::utils::read_json_file(&task_path.join("config.json"))
            .map_err(|e| {
                error!("Failed to read config.json for task {}: {:?}", uid, e);
                e
            })?;

        debug!("Loading status.json for task: {}", uid);
        let status: TaskStatus = crate::utils::read_json_file(&task_path.join("status.json"))
            .map_err(|e| {
                error!("Failed to read status.json for task {}: {:?}", uid, e);
                e
            })?;

        debug!("Loading dependencies.json for task: {}", uid);
        let dependencies: TaskDependencies = crate::utils::read_json_file(&task_path.join("dependencies.json"))
            .map_err(|e| {
                error!("Failed to read dependencies.json for task {}: {:?}", uid, e);
                e
            })?;

        debug!("Loading objective.md for task: {}", uid);
        let objective = match crate::utils::read_file_content(&task_path.join("objective.md")) {
            Ok(content) => content,
            Err(ProjectError::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => {
                warn!("objective.md not found for task {}. Initializing with empty string.", uid);
                "".to_string()
            },
            Err(e) => {
                error!("Failed to read objective.md for task {}: {:?}", uid, e);
                return Err(e);
            }
        };

        debug!("Loading plan.md for task: {}", uid);
        let plan = match crate::utils::read_file_content(&task_path.join("plan.md")) {
            Ok(content) => Some(content),
            Err(ProjectError::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => {
                warn!("plan.md not found for task {}. Initializing with None.", uid);
                None
            },
            Err(e) => {
                error!("Failed to read plan.md for task {}: {:?}", uid, e);
                return Err(e);
            }
        };

        Ok(Task {
            uid: uid.to_string(),
            root_path: task_path,
            config,
            status,
            objective,
            plan,
            dependencies,
            subtask_uids: Vec::new(),
        })
    }
}
