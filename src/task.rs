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

impl TaskState {
    pub fn can_transition_to(self, next_state: TaskState) -> bool {
        match self {
            TaskState::Created => matches!(next_state, TaskState::ObjectiveDefined | TaskState::Error | TaskState::Failed),
            TaskState::ObjectiveDefined => matches!(next_state, TaskState::ObjectiveDefined | TaskState::PlanDefined | TaskState::Error | TaskState::Failed),
            TaskState::PlanDefined => matches!(next_state, TaskState::PlanDefined | TaskState::Working | TaskState::Delegating | TaskState::ObjectiveDefined | TaskState::Error | TaskState::Failed),
            TaskState::Delegating => matches!(next_state, TaskState::Delegating | TaskState::Harvesting | TaskState::Error | TaskState::Failed),
            TaskState::Harvesting => matches!(next_state, TaskState::Completed | TaskState::Error | TaskState::Failed),
            TaskState::Working => matches!(next_state, TaskState::Completed | TaskState::Error | TaskState::Failed),
            TaskState::Error => matches!(next_state, TaskState::Failed | TaskState::Error), // From Error, can transition to Failed or stay in Error
            TaskState::Failed | TaskState::Completed => false, // Final states, no transitions out
    }
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
    pub subtasks: HashMap<String, TaskState>,
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

    pub fn define_plan(&mut self, plan_content: String, task_type: TaskType) -> Result<(), ProjectError> {
        if !self.status.current_state.can_transition_to(TaskState::PlanDefined) {
            return Err(ProjectError::InvalidStateTransition(
                self.status.current_state,
                TaskState::PlanDefined,
            ));
        }

        write_file_content(&self.root_path.join("plan.md"), &plan_content)?;
        self.plan = Some(plan_content);

        self.config.task_type = Some(task_type);
        write_json_file(&self.root_path.join("config.json"), &self.config)?;

        update_task_status(&self.root_path, TaskState::PlanDefined, &mut self.status)?;

        Ok(())
    }

    pub fn accept_plan(&mut self) -> Result<(), ProjectError> {
        if !self.status.current_state.can_transition_to(TaskState::Working) &&
           !self.status.current_state.can_transition_to(TaskState::Delegating) {
            return Err(ProjectError::InvalidStateTransition(
                self.status.current_state,
                TaskState::Working, // Or Delegating, depending on task_type
            ));
        }

        let next_state = match self.config.task_type {
            Some(TaskType::Monolithic) => TaskState::Working,
            Some(TaskType::Subdivided) => TaskState::Delegating,
            None => return Err(ProjectError::InvalidProjectConfig("TaskType not defined for task when accepting plan.".to_string())),
        };

        update_task_status(&self.root_path, next_state, &mut self.status)?;

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

    pub fn add_subtask(&mut self, subtask_id: String, is_final: bool) -> Result<(), ProjectError> {
        if !self.status.current_state.can_transition_to(TaskState::Delegating) &&
           !(self.status.current_state == TaskState::Delegating && is_final) { // Allow transition to Harvesting from Delegating
            return Err(ProjectError::InvalidStateTransition(
                self.status.current_state,
                TaskState::Delegating, // Or Harvesting
            ));
        }

        self.subtasks.insert(subtask_id, TaskState::Created); // Add subtask with Created state

        if is_final {
            update_task_status(&self.root_path, TaskState::Harvesting, &mut self.status)?;
        } else {
            // If not final, stay in Delegating state, but ensure status is saved
            write_json_file(&self.root_path.join("status.json"), &self.status)?;
        }

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
