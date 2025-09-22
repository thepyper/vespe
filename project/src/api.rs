use std::fs;
use chrono::Utc;
use uuid::Uuid;
use sha2::{Sha256, Digest};
use std::io::Read;

use crate::models::{Task, TaskConfig, TaskStatus, TaskState, TaskDependencies, PersistentEvent};
use crate::error::ProjectError;
use crate::utils::{self, get_task_path, generate_task_uid, write_json_file, write_file_content, read_json_file, read_file_content, update_task_status, hash_file};

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

/// Transitions from `CREATED` to `OBJECTIVE_DEFINED`.
/// Writes the objective content to `objective.md`.
pub fn define_objective(task_uid: &str, objective_content: String) -> Result<Task, ProjectError> {
    let mut task = load_task(task_uid)?;
    let task_path = get_task_path(task_uid)?;

    // Update objective.md
    write_file_content(&task_path.join("objective.md"), &objective_content)?;
    task.objective = objective_content;

    // Update status
    update_task_status(&task_path, TaskState::ObjectiveDefined, &mut task.status)?;

    Ok(task)
}

/// Transitions from `OBJECTIVE_DEFINED` to `PLAN_DEFINED`.
/// Writes the plan content to `plan.md`.
pub fn define_plan(task_uid: &str, plan_content: String) -> Result<Task, ProjectError> {
    let mut task = load_task(task_uid)?;
    let task_path = get_task_path(task_uid)?;

    // Update plan.md
    write_file_content(&task_path.join("plan.md"), &plan_content)?;
    task.plan = Some(plan_content);

    // Update status
    update_task_status(&task_path, TaskState::PlanDefined, &mut task.status)?;

    Ok(task)
}

/// Adds a new event to the `persistent/` folder of the task.
pub fn add_persistent_event(task_uid: &str, event: PersistentEvent) -> Result<(), ProjectError> {
    let task_path = get_task_path(task_uid)?;
    let persistent_path = task_path.join("persistent");

    let filename = format!("{}_{}.json", event.timestamp.format("%Y%m%d%H%M%S%3f"), event.event_type);
    let file_path = persistent_path.join(filename);

    write_json_file(&file_path, &event)?;

    Ok(())
}

/// Retrieves all persistent events for a task, sorted by timestamp.
pub fn get_all_persistent_events(task_uid: &str) -> Result<Vec<PersistentEvent>, ProjectError> {
    let task_path = get_task_path(task_uid)?;
    let persistent_path = task_path.join("persistent");

    if !persistent_path.exists() {
        return Ok(Vec::new());
    }

    let mut events = Vec::new();
    for entry in fs::read_dir(&persistent_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
            let event: PersistentEvent = read_json_file(&path)?;
            events.push(event);
        }
    }

    events.sort_by_key(|event| event.timestamp);

    Ok(events)
}

/// Calculates the SHA256 hash of the `result/` folder content for a task.
/// The hash is based on the content of all files and their relative paths within the folder.
pub fn calculate_result_hash(task_uid: &str) -> Result<String, ProjectError> {
    let task_path = get_task_path(task_uid)?;
    let result_path = task_path.join("result");

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
                .map_err(|e| ProjectError::InvalidPath(path.to_path_buf()))?
                .to_string_lossy()
                .into_owned();
            let file_hash = hash_file(path)?;
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
pub fn add_result_file(task_uid: &str, filename: &str, content: Vec<u8>) -> Result<(), ProjectError> {
    let task_path = get_task_path(task_uid)?;
    let result_path = task_path.join("result");

    // Ensure the result directory exists
    fs::create_dir_all(&result_path).map_err(|e| ProjectError::Io(e))?;

    let file_path = result_path.join(filename);
    fs::write(&file_path, content).map_err(|e| ProjectError::Io(e))?;

    Ok(())
}
