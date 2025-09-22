use std::path::{Path, PathBuf};
use uuid::Uuid;
use chrono::Utc;
use sha2::{Sha256, Digest};

use crate::error::ProjectError;
use crate::models::{TaskStatus, TaskState};

/// Generates a unique UID for a task, agent, or tool.
pub fn generate_uid(prefix: &str) -> Result<String, ProjectError> {
    let uuid = Uuid::new_v4();
    Ok(format!("{}-{}", prefix, uuid.to_string().replace("-", "")))
}

/// Finds the project root by traversing up the directory tree until a .vespe/ directory is found.
pub fn get_project_root_path() -> Result<PathBuf, ProjectError> {
    let mut current_dir = std::env::current_dir().map_err(|e| ProjectError::Io(e))?;

    loop {
        let vespe_dir = current_dir.join(".vespe");
        if vespe_dir.exists() && vespe_dir.is_dir() {
            return Ok(current_dir);
        }

        if !current_dir.pop() {
            // Reached the root of the filesystem
            return Err(ProjectError::ProjectRootNotFound(std::env::current_dir().map_err(|e| ProjectError::Io(e))?));
        }
    }
}

/// Constructs the full path for a given entity UID within a base path.
pub fn get_entity_path(base_path: &Path, uid: &str) -> Result<PathBuf, ProjectError> {
    Ok(base_path.join(uid))
}

/// Returns the base path for tasks within a project.
pub fn get_tasks_base_path(project_root_path: &Path) -> PathBuf {
    project_root_path.join(".vespe").join("tasks")
}

/// Returns the base path for agents within a project.
pub fn get_agents_base_path(project_root_path: &Path) -> PathBuf {
    project_root_path.join(".vespe").join("agents")
}

/// Returns the base path for project-specific tools within a project.
pub fn get_tools_base_path(project_root_path: &Path) -> PathBuf {
    project_root_path.join(".vespe").join("tools")
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

/// Calculates the SHA256 hash of a file.
pub fn hash_file(path: &Path) -> Result<String, ProjectError> {
    let mut file = std::fs::File::open(path).map_err(|e| ProjectError::Io(e))?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher).map_err(|e| ProjectError::Io(e))?;
    Ok(format!("{:x}", hasher.finalize()))
}
