use std::fs;
use std::path::Path;
use chrono::Utc;
use uuid::Uuid;
use sha2::{Sha256, Digest};

use crate::models::{Task, TaskConfig, TaskStatus, TaskState, TaskDependencies, PersistentEvent};
use crate::tool_models::{Tool, ToolConfig};
use crate::project_models::ProjectConfig;
use crate::error::ProjectError;
use crate::utils::{get_entity_path, generate_uid, write_json_file, write_file_content, read_json_file, read_file_content, update_task_status, hash_file, get_tasks_base_path, get_tools_base_path};

/// Creates a new task or subtask.
/// Initializes the task directory with config.json, empty objective.md, etc.
/// The task is created in the `CREATED` state.
pub fn create_task(
    project_root_path: &Path,
    parent_uid: Option<String>,
    name: String,
    created_by: String,
    _template_name: String, // Template not yet implemented, ignored for now
) -> Result<Task, ProjectError> {
    let uid = generate_uid("tsk")?;
    let tasks_base_path = get_tasks_base_path(project_root_path);
    let task_path = get_entity_path(&tasks_base_path, &uid)?;

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
    load_task(project_root_path, &uid)
}

/// Loads a task from the filesystem given its UID.
pub fn load_task(
    project_root_path: &Path,
    uid: &str
) -> Result<Task, ProjectError> {
    let tasks_base_path = get_tasks_base_path(project_root_path);
    let task_path = get_entity_path(&tasks_base_path, uid)?;

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
pub fn define_objective(
    project_root_path: &Path,
    task_uid: &str,
    objective_content: String
) -> Result<Task, ProjectError> {
    let mut task = load_task(project_root_path, task_uid)?;
    let tasks_base_path = get_tasks_base_path(project_root_path);
    let task_path = get_entity_path(&tasks_base_path, task_uid)?;

    // Update objective.md
    write_file_content(&task_path.join("objective.md"), &objective_content)?;
    task.objective = objective_content;

    // Update status
    update_task_status(&task_path, TaskState::ObjectiveDefined, &mut task.status)?;

    Ok(task)
}

/// Transitions from `OBJECTIVE_DEFINED` to `PLAN_DEFINED`.
/// Writes the plan content to `plan.md`.
pub fn define_plan(
    project_root_path: &Path,
    task_uid: &str,
    plan_content: String
) -> Result<Task, ProjectError> {
    let mut task = load_task(project_root_path, task_uid)?;
    let tasks_base_path = get_tasks_base_path(project_root_path);
    let task_path = get_entity_path(&tasks_base_path, task_uid)?;

    // Update plan.md
    write_file_content(&task_path.join("plan.md"), &plan_content)?;
    task.plan = Some(plan_content);

    // Update status
    update_task_status(&task_path, TaskState::PlanDefined, &mut task.status)?;

    Ok(task)
}

/// Adds a new event to the `persistent/` folder of the task.
pub fn add_persistent_event(
    project_root_path: &Path,
    task_uid: &str,
    event: PersistentEvent
) -> Result<(), ProjectError> {
    let tasks_base_path = get_tasks_base_path(project_root_path);
    let task_path = get_entity_path(&tasks_base_path, task_uid)?;
    let persistent_path = task_path.join("persistent");

    // Use UUID for filename to guarantee uniqueness, append timestamp for sorting
    let filename = format!("{}_{}_{}.json", event.timestamp.format("%Y%m%d%H%M%S%3f"), Uuid::new_v4().as_simple(), event.event_type);
    let file_path = persistent_path.join(filename);

    write_json_file(&file_path, &event)?;

    Ok(())
}

/// Retrieves all persistent events for a task, sorted by timestamp.
pub fn get_all_persistent_events(
    project_root_path: &Path,
    task_uid: &str
) -> Result<Vec<PersistentEvent>, ProjectError> {
    let tasks_base_path = get_tasks_base_path(project_root_path);
    let task_path = get_entity_path(&tasks_base_path, task_uid)?;
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
pub fn calculate_result_hash(
    project_root_path: &Path,
    task_uid: &str
) -> Result<String, ProjectError> {
    let tasks_base_path = get_tasks_base_path(project_root_path);
    let task_path = get_entity_path(&tasks_base_path, task_uid)?;
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
                .map_err(|_e| ProjectError::InvalidPath(path.to_path_buf()))?
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
pub fn add_result_file(
    project_root_path: &Path,
    task_uid: &str,
    filename: &str,
    content: Vec<u8>
) -> Result<(), ProjectError> {
    let tasks_base_path = get_tasks_base_path(project_root_path);
    let task_path = get_entity_path(&tasks_base_path, task_uid)?;
    let result_path = task_path.join("result");

    // Ensure the result directory exists
    fs::create_dir_all(&result_path).map_err(|e| ProjectError::Io(e))?;

    let file_path = result_path.join(filename);
    fs::write(&file_path, content).map_err(|e| ProjectError::Io(e))?;

    Ok(())
}

/// Creates a new tool.
pub fn create_tool(
    project_root_path: &Path,
    name: String,
    description: String,
    schema: serde_json::Value,
    implementation_details: serde_json::Value,
) -> Result<Tool, ProjectError> {
    let uid = generate_uid("tool")?;
    let tools_base_path = get_tools_base_path(project_root_path);
    let tool_path = get_entity_path(&tools_base_path, &uid)?;

    fs::create_dir_all(&tool_path).map_err(|e| ProjectError::Io(e))?;

    let config = ToolConfig {
        uid: uid.clone(),
        name: name.clone(),
        description: description.clone(),
        schema,
        implementation_details,
    };
    write_json_file(&tool_path.join("config.json"), &config)?;
    write_file_content(&tool_path.join("description.md"), &description)?;

    Ok(Tool {
        uid: uid.clone(),
        root_path: tool_path,
        config,
    })
}

/// Loads a tool from the filesystem given its absolute path.
pub fn load_tool(
    tool_path: &Path,
) -> Result<Tool, ProjectError> {
    if !tool_path.exists() {
        return Err(ProjectError::ToolNotFound(tool_path.to_string_lossy().into_owned()));
    }

    let config: ToolConfig = read_json_file(&tool_path.join("config.json"))?;
    let description_content = read_file_content(&tool_path.join("description.md"))?;

    Ok(Tool {
        uid: config.uid.clone(),
        root_path: tool_path.to_path_buf(),
        config,
    })
}

/// Lists all tools in a given base path.
fn list_all_tools_in_path(base_path: &Path) -> Result<Vec<Tool>, ProjectError> {
    let mut tools = Vec::new();
    if !base_path.exists() {
        return Ok(tools);
    }

    for entry in fs::read_dir(base_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(uid_str) = path.file_name().and_then(|s| s.to_str()) {
                // Attempt to load the tool using its direct path
                match load_tool(&path) {
                    Ok(tool) => tools.push(tool),
                    Err(e) => eprintln!("Warning: Could not load tool {}: {}", uid_str, e),
                }
            }
        }
    }
    Ok(tools)
}

/// Resolves a tool given its name and project configuration.
/// For now, only resolves project-specific tools.
pub fn resolve_tool(
    project_root_path: &Path,
    _project_config: &ProjectConfig, // project_config is not used for now as kits are out of scope
    tool_name: &str
) -> Result<Tool, ProjectError> {
    let tools_base_path = get_tools_base_path(project_root_path);
    let project_tools = list_all_tools_in_path(&tools_base_path)?;
    if let Some(tool) = project_tools.into_iter().find(|t| t.config.name == tool_name) {
        return Ok(tool);
    }

    Err(ProjectError::ToolNotFound(tool_name.to_string()))
}

/// Lists all tools available for a given project.
/// For now, only lists project-specific tools.
pub fn list_available_tools(
    project_root_path: &Path,
    _project_config: &ProjectConfig // project_config is not used for now as kits are out of scope
) -> Result<Vec<Tool>, ProjectError> {
    let tools_base_path = get_tools_base_path(project_root_path);
    let available_tools = list_all_tools_in_path(&tools_base_path)?;

    // No deduplication needed as only project-specific tools are listed.
    Ok(available_tools)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use chrono::Duration;
    use tempfile::tempdir;

    // Helper to set up a clean test environment
    fn setup_test_env() -> PathBuf {
        let temp_dir = tempdir().unwrap();
        let base_path = temp_dir.path(); // This is the project root for the test
        let vespe_dir = base_path.join(".vespe");
        let tasks_dir = vespe_dir.join("tasks");
        let tools_dir = vespe_dir.join("tools");
        fs::create_dir_all(&tasks_dir).unwrap();
        fs::create_dir_all(&tools_dir).unwrap();
        base_path.to_path_buf()
    }

    #[test]
    fn test_create_task() {
        let project_root_path = setup_test_env();

        let task_name = "Test Task 1".to_string();
        let created_by = "test_user".to_string();
        let template = "default".to_string();

        let task = create_task(&project_root_path, None, task_name.clone(), created_by.clone(), template.clone()).unwrap();

        assert_eq!(task.config.name, task_name);
        assert_eq!(task.config.created_by, created_by);
        assert_eq!(task.status.current_state, TaskState::Created);
        assert!(task.root_path.exists());
        assert!(task.root_path.join("config.json").exists());
        assert!(task.root_path.join("status.json").exists());
        assert!(task.root_path.join("objective.md").exists());
        assert!(task.root_path.join("plan.md").exists());
        assert!(task.root_path.join("dependencies.json").exists());
        assert!(task.root_path.join("persistent").exists());
        assert!(task.root_path.join("result").exists());
    }

    #[test]
    fn test_load_task() {
        let project_root_path = setup_test_env();

        let task = create_task(&project_root_path, None, "Load Test".to_string(), "loader".to_string(), "default".to_string()).unwrap();
        let loaded_task = load_task(&project_root_path, &task.uid).unwrap();

        assert_eq!(task.uid, loaded_task.uid);
        assert_eq!(task.config.name, loaded_task.config.name);
        assert_eq!(task.status.current_state, loaded_task.status.current_state);
        assert_eq!(loaded_task.objective, task.objective);
        assert_eq!(loaded_task.plan, task.plan);
    }

    #[test]
    fn test_define_objective() {
        let project_root_path = setup_test_env();

        let task = create_task(&project_root_path, None, "Define Objective Test".to_string(), "tester".to_string(), "default".to_string()).unwrap();
        assert_eq!(task.status.current_state, TaskState::Created);

        let objective_content = "This is the new objective.".to_string();
        let updated_task = define_objective(&project_root_path, &task.uid, objective_content.clone()).unwrap();

        assert_eq!(updated_task.status.current_state, TaskState::ObjectiveDefined);
        assert_eq!(updated_task.objective, objective_content);
        assert!(updated_task.status.last_updated_at > task.status.last_updated_at);

        let loaded_task = load_task(&project_root_path, &task.uid).unwrap();
        assert_eq!(loaded_task.status.current_state, TaskState::ObjectiveDefined);
        assert_eq!(loaded_task.objective, objective_content);
    }

    #[test]
    fn test_define_plan() {
        let project_root_path = setup_test_env();

        let task = create_task(&project_root_path, None, "Define Plan Test".to_string(), "tester".to_string(), "default".to_string()).unwrap();
        define_objective(&project_root_path, &task.uid, "Objective for plan.".to_string()).unwrap(); // Transition to ObjectiveDefined

        let plan_content = "This is the detailed plan.".to_string();
        let updated_task = define_plan(&project_root_path, &task.uid, plan_content.clone()).unwrap();

        assert_eq!(updated_task.status.current_state, TaskState::PlanDefined);
        assert_eq!(updated_task.plan, Some(plan_content));
        assert!(updated_task.status.last_updated_at > task.status.last_updated_at);

        let loaded_task = load_task(&project_root_path, &task.uid).unwrap();
        assert_eq!(loaded_task.status.current_state, TaskState::PlanDefined);
        assert_eq!(loaded_task.plan, Some("This is the detailed plan.".to_string()));
    }

    #[test]
    fn test_add_persistent_event() {
        let project_root_path = setup_test_env();

        let task = create_task(&project_root_path, None, "Persistent Event Test".to_string(), "tester".to_string(), "default".to_string()).unwrap();

        let event1 = PersistentEvent {
            timestamp: Utc::now(),
            event_type: "llm_response".to_string(),
            agent_id: "agent_a".to_string(),
            content: "LLM thought 1".to_string(),
        };
        add_persistent_event(&project_root_path, &task.uid, event1.clone()).unwrap();

        let event2 = PersistentEvent {
            timestamp: Utc::now() + Duration::seconds(1), // Ensure different timestamp
            event_type: "tool_call".to_string(),
            agent_id: "agent_b".to_string(),
            content: "Tool called with args".to_string(),
        };
        add_persistent_event(&project_root_path, &task.uid, event2.clone()).unwrap();

        let events = get_all_persistent_events(&project_root_path, &task.uid).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event_type, event1.event_type);
        assert_eq!(events[1].event_type, event2.event_type);
        assert_eq!(events[0].content, event1.content);
        assert_eq!(events[1].content, event2.content);
    }

    #[test]
    fn test_add_result_file_and_calculate_hash() {
        let project_root_path = setup_test_env();

        let task = create_task(&project_root_path, None, "Result Hash Test".to_string(), "tester".to_string(), "default".to_string()).unwrap();

        // Initial hash of empty result folder
        let initial_hash = calculate_result_hash(&project_root_path, &task.uid).unwrap();
        assert_ne!(initial_hash, ""); // Should not be empty

        // Add a file
        add_result_file(&project_root_path, &task.uid, "file1.txt", "content1".as_bytes().to_vec()).unwrap();
        let hash1 = calculate_result_hash(&project_root_path, &task.uid).unwrap();
        assert_ne!(initial_hash, hash1);

        // Add another file
        add_result_file(&project_root_path, &task.uid, "file2.txt", "content2".as_bytes().to_vec()).unwrap();
        let hash2 = calculate_result_hash(&project_root_path, &task.uid).unwrap();
        assert_ne!(hash1, hash2);

        // Modify a file
        add_result_file(&project_root_path, &task.uid, "file1.txt", "new_content1".as_bytes().to_vec()).unwrap();
        let hash3 = calculate_result_hash(&project_root_path, &task.uid).unwrap();
        assert_ne!(hash2, hash3);

        // Add a file in a subdirectory
        let task_path = get_entity_path(&get_tasks_base_path(&project_root_path), &task.uid).unwrap();
        fs::create_dir_all(task_path.join("result").join("subdir")).unwrap();
        add_result_file(&project_root_path, &task.uid, "subdir/file3.txt", "content3".as_bytes().to_vec()).unwrap();
        let hash4 = calculate_result_hash(&project_root_path, &task.uid).unwrap();
        assert_ne!(hash3, hash4);
    }

    #[test]
    fn test_invalid_state_transition() {
        let project_root_path = setup_test_env();

        let task = create_task(&project_root_path, None, "Invalid Transition Test".to_string(), "tester".to_string(), "default".to_string()).unwrap();
        
        // Try to define plan from Created state (invalid)
        let result = define_plan(&project_root_path, &task.uid, "Invalid plan".to_string());
        assert!(result.is_err());
        if let Err(ProjectError::InvalidStateTransition(from, to)) = result {
            assert_eq!(from, TaskState::Created);
            assert_eq!(to, TaskState::PlanDefined);
        } else {
            panic!("Expected InvalidStateTransition error");
        }
    }
}