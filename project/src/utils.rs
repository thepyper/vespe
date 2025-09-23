use std::path::{Path, PathBuf};
use uuid::Uuid;
use chrono::Utc;
use sha2::{Sha256, Digest};
use std::fs; // Added for fs operations

use crate::error::ProjectError;
use crate::models::{TaskStatus, TaskState};

// Constants for project root detection
const VESPE_DIR: &str = ".vespe";
const VESPE_ROOT_MARKER: &str = ".vespe_root";

/// Generates a unique UID for a task, agent, or tool.
pub fn generate_uid(prefix: &str) -> Result<String, ProjectError> {
    let uuid = Uuid::new_v4();
    Ok(format!("{}-{}", prefix, uuid.to_string().replace("-", "")))
}

/// Checks if a given directory is a Vespe project root by looking for the .vespe/.vespe_root marker file.
pub fn is_project_root(dir: &Path) -> bool {
    dir.join(VESPE_DIR).join(VESPE_ROOT_MARKER).exists()
}

/// Finds the project root by traversing up the directory tree until a .vespe/ directory is found.
pub fn find_project_root(start_dir: &Path) -> Option<PathBuf> {
    let mut current_dir = Some(start_dir);

    while let Some(dir) = current_dir {
        if is_project_root(dir) {
            return Some(dir.to_path_buf());
        }
        current_dir = dir.parent();
    }
    None
}

/// Initializes a new Vespe project at the target directory.
pub fn initialize_project_root(target_dir: &Path) -> Result<(), ProjectError> {
    println!("DEBUG: initialize_project_root called with target_dir: {}", target_dir.display());
    // Create the target directory if it doesn't exist
    fs::create_dir_all(target_dir).map_err(|e| ProjectError::Io(e))?;

    let absolute_target_dir = target_dir.canonicalize()
        .map_err(|e| ProjectError::InvalidPath(target_dir.to_path_buf()))?; // Convert anyhow to ProjectError
    println!("DEBUG: absolute_target_dir: {}", absolute_target_dir.display());

    // Check if target_dir is already part of an existing Vespe project
    if let Some(found_root) = find_project_root(&absolute_target_dir) {
        println!("DEBUG: Found existing root: {}", found_root.display());
        return Err(ProjectError::InvalidProjectConfig(format!(
            "Cannot initialize a Vespe project inside an existing project. Existing root: {}",
            found_root.display()
        ))); // Convert anyhow to ProjectError
    }

    let vespe_dir = absolute_target_dir.join(VESPE_DIR);
    let vespe_root_marker = vespe_dir.join(VESPE_ROOT_MARKER);
    println!("DEBUG: Creating vespe_dir: {}", vespe_dir.display());

    fs::create_dir_all(&vespe_dir).map_err(|e| ProjectError::Io(e))?;
    println!("DEBUG: vespe_dir created.");

    fs::write(&vespe_root_marker, "Feel The BuZZ!!!!").map_err(|e| ProjectError::Io(e))?;
    println!("DEBUG: vespe_root_marker created.");

    let vespe_gitignore = vespe_dir.join(".gitignore");
    fs::write(&vespe_gitignore, "log/").map_err(|e| ProjectError::Io(e))?;
    println!("DEBUG: .gitignore created.");

    Ok(())
}

/// Constructs the full path for a given entity UID within a base path.
pub fn get_entity_path(base_path: &Path, uid: &str) -> Result<PathBuf, ProjectError> {
    Ok(base_path.join(uid))
}

/// Returns the base path for tasks within a project.
pub fn get_tasks_base_path(project_root_path: &Path) -> PathBuf {
    project_root_path.join(VESPE_DIR).join("tasks")
}

/// Returns the base path for agents within a project.
pub fn get_agents_base_path(project_root_path: &Path) -> PathBuf {
    project_root_path.join(VESPE_DIR).join("agents")
}

/// Returns the base path for project-specific tools within a project.
pub fn get_tools_base_path(project_root_path: &Path) -> PathBuf {
    project_root_path.join(VESPE_DIR).join("tools")
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
    write_json_file(&task_path.join("status.json"), current_status)?; // Await here
    Ok(())
}

/// Calculates the SHA256 hash of a file.
pub fn hash_file(path: &Path) -> Result<String, ProjectError> {
    let mut file = std::fs::File::open(path).map_err(|e| ProjectError::Io(e))?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher).map_err(|e| ProjectError::Io(e))?;
    Ok(format!("{:x}", hasher.finalize()))
}
