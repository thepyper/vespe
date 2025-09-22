use std::path::{Path, PathBuf};
use uuid::Uuid;
use chrono::Utc;

use crate::error::ProjectError;
use crate::models::{TaskStatus, TaskState};

// Base path for all tasks. For now, hardcoded.
// In a real application, this would be configurable (e.g., from project config).
pub fn get_tasks_base_path() -> Result<PathBuf, ProjectError> {
    // This path should be relative to the project root, which is where vespe is run.
    // For now, we assume the current working directory is the project root.
    let current_dir = std::env::current_dir().map_err(|e| ProjectError::Io(e))?;
    let base_path = current_dir.join(".vespe").join("tasks");
    Ok(base_path)
}

/// Generates a unique UID for a task.
pub fn generate_task_uid() -> Result<String, ProjectError> {
    let uuid = Uuid::new_v4();
    Ok(format!("tsk-{}", uuid.to_string().replace("-", "")))
}

/// Constructs the full path for a given task UID.
pub fn get_task_path(uid: &str) -> Result<PathBuf, ProjectError> {
    let base_path = get_tasks_base_path()?;
    Ok(base_path.join(uid))
}

/// Reads the content of a file as a String.
pub fn read_file_content(path: &Path) -> Result<String, ProjectError> {
    std::fs::read_to_string(path).map_err(|e| ProjectError::Io(e))
}

/// Writes content to a file.
pub fn write_file_content(path: &Path, content: &str) -> Result<(), ProjectError> {
    std::fs::write(path, content).map_err(|e| ProjectError::Io(e))
}

/// Reads and deserializes a JSON file.
pub fn read_json_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, ProjectError> {
    let content = read_file_content(path)?;
    serde_json::from_str(&content).map_err(|e| ProjectError::Json(e))
}

/// Serializes and writes to a JSON file.
pub fn write_json_file<T: serde::Serialize>(path: &Path, data: &T) -> Result<(), ProjectError> {
    let content = serde_json::to_string_pretty(data).map_err(|e| ProjectError::Json(e))?;
    write_file_content(path, &content)
}

/// Updates the status.json file for a given task.
pub fn update_task_status(task_path: &Path, new_state: TaskState, current_status: &mut TaskStatus) -> Result<(), ProjectError> {
    if !current_status.current_state.can_transition_to(new_state) {
        return Err(ProjectError::InvalidStateTransition(current_status.current_state, new_state));
    }
    current_status.current_state = new_state;
    current_status.last_updated_at = Utc::now();
    write_json_file(&task_path.join("status.json"), current_status)?;
    Ok(())
}
