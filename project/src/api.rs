use std::fs;
use chrono::Utc;
use uuid::Uuid;

use crate::models::{Task, TaskConfig, TaskStatus, TaskState, TaskDependencies, PersistentEvent};
use crate::error::ProjectError;
use crate::utils::{self, get_task_path, generate_task_uid, write_json_file, write_file_content, read_json_file, read_file_content};

/// Creates a new task or subtask.
/// Initializes the task directory with config.json, empty objective.md, etc.
/// The task is created in the `CREATED` state.
pub fn create_task(
    parent_uid: Option<String>,
    name: String,
    created_by: String,
    _template_name: String, // Template not yet implemented, ignored for now
) -> Result<Task, ProjectError> {
    let uid = generate_task_uid()?;
    let task_path = get_task_path(&uid)?;

    // Create task directory and subdirectories
    fs::create_dir_all(&task_path).map_err(|e| ProjectError::Io(e))?;
    fs::create_dir_all(task_path.join("persistent")).map_err(|e| ProjectError::Io(e))?;
    fs::create_dir_all(task_path.join("result")).map_err(|e| ProjectError::Io(e))?;

    let now = Utc::now();

    // Initialize config.json
    let config = TaskConfig {
        uid: uid.clone(),
        name: name.clone(),
        created_by: created_by.clone(),
        created_at: now,
        parent_uid,
    };
    write_json_file(&task_path.join("config.json"), &config)?;

    // Initialize status.json
    let status = TaskStatus {
        current_state: TaskState::Created,
        last_updated_at: now,
        progress: None,
        parent_content_hashes: std::collections::HashMap::new(),
    };
    write_json_file(&task_path.join("status.json"), &status)?;

    // Create empty objective.md and plan.md
    write_file_content(&task_path.join("objective.md"), "")?;
    write_file_content(&task_path.join("plan.md"), "")?;

    // Initialize dependencies.json
    let dependencies = TaskDependencies { depends_on: Vec::new() };
    write_json_file(&task_path.join("dependencies.json"), &dependencies)?;

    // Load the newly created task to return it
    load_task(&uid)
}

/// Loads a task from the filesystem given its UID.
pub fn load_task(uid: &str) -> Result<Task, ProjectError> {
    let task_path = get_task_path(uid)?;

    if !task_path.exists() {
        return Err(ProjectError::TaskNotFound(uid.to_string()));
    }

    let config: TaskConfig = read_json_file(&task_path.join("config.json"))?;
    let status: TaskStatus = read_json_file(&task_path.join("status.json"))?;
    let dependencies: TaskDependencies = read_json_file(&task_path.join("dependencies.json"))?;
    let objective = read_file_content(&task_path.join("objective.md"))?;
    let plan = Some(read_file_content(&task_path.join("plan.md"))?);

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