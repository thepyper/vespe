use std::fs;
use std::path::Path;
use chrono::Utc;
use uuid::Uuid;
use sha2::{Sha256, Digest};

use crate::models::{Task, TaskConfig, TaskStatus, TaskState, TaskDependencies, PersistentEvent, Agent, AgentType};
use crate::tool_models::{Tool, ToolConfig};
use crate::project_models::Project;
use crate::error::ProjectError;
use crate::utils::{get_entity_path, generate_uid, write_json_file, write_file_content, read_json_file, read_file_content, update_task_status, hash_file};

/// Loads a task from the filesystem given its UID.
pub fn load_task(
    project: &Project,
    uid: &str
) -> Result<Task, ProjectError> {
    let tasks_base_path = project.tasks_dir();
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
    project: &Project,
    task_uid: &str,
    objective_content: String
) -> Result<Task, ProjectError> {
    let mut task = load_task(project, task_uid)?;
    let tasks_base_path = project.tasks_dir();
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
    project: &Project,
    task_uid: &str,
    plan_content: String
) -> Result<Task, ProjectError> {
    let mut task = load_task(project, task_uid)?;
    let tasks_base_path = project.tasks_dir();
    let task_path = get_entity_path(&tasks_base_path, task_uid)?;

    // Prevent defining a plan for a replanned task
    if task.status.current_state == TaskState::Replanned {
        return Err(ProjectError::InvalidStateTransition(
            task.status.current_state,
            TaskState::PlanDefined, // Attempted target state
        ));
    }

    // Update plan.md
    write_file_content(&task_path.join("plan.md"), &plan_content)?;
    task.plan = Some(plan_content);

    // Update status
    update_task_status(&task_path, TaskState::PlanDefined, &mut task.status)?;

    Ok(task)
}

/// Adds a new event to the `persistent/` folder of the task.
pub fn add_persistent_event(
    project: &Project,
    task_uid: &str,
    event: PersistentEvent
) -> Result<(), ProjectError> {
    let tasks_base_path = project.tasks_dir();
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
    project: &Project,
    task_uid: &str
) -> Result<Vec<PersistentEvent>, ProjectError> {
    let tasks_base_path = project.tasks_dir();
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
    project: &Project,
    task_uid: &str
) -> Result<String, ProjectError> {
    let tasks_base_path = project.tasks_dir();
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
    project: &Project,
    task_uid: &str,
    filename: &str,
    content: Vec<u8>
) -> Result<(), ProjectError> {
    let tasks_base_path = project.tasks_dir();
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
    project: &Project,
    name: String,
    description: String,
    schema: serde_json::Value,
    implementation_details: serde_json::Value,
) -> Result<Tool, ProjectError> {
    let uid = generate_uid("tool")?;
    let tools_base_path = project.tasks_dir();
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

/// Reviews a task, transitioning it to Completed (approved) or Replanned (rejected).
pub fn review_task(
    project: &Project,
    task_uid: &str,
    approved: bool, // true for approve, false for reject
) -> Result<Task, ProjectError> {
    let mut task = load_task(project, task_uid)?;
    let tasks_base_path = project.tasks_dir();
    let task_path = get_entity_path(&tasks_base_path, task_uid)?;

    if task.status.current_state != TaskState::NeedsReview {
        return Err(ProjectError::InvalidStateTransition(
            task.status.current_state,
            TaskState::NeedsReview, // This is not quite right, should be the target state
        ));
    }

    let next_state = if approved {
        TaskState::Completed // Or TaskState::Ready if we add it
    } else {
        TaskState::Replanned
    };

    if !task.status.current_state.can_transition_to(next_state) {
        return Err(ProjectError::InvalidStateTransition(
            task.status.current_state,
            next_state,
        ));
    }

    update_task_status(&task_path, next_state, &mut task.status)?;

    Ok(task)
}

/// Creates a new agent (AI or human).
pub fn create_agent(
    project: &Project,
    agent_type: AgentType,
    name: String,
    // ... other initial configuration fields
) -> Result<Agent, ProjectError> {
    let uid_prefix = match agent_type {
        AgentType::Human => "usr", // Or "human"
        AgentType::AI => "agt",
    };
    let uid = generate_uid(uid_prefix)?;
    let agents_base_path = project.agents_dir();
    let agent_path = get_entity_path(&agents_base_path, &uid)?;

    fs::create_dir_all(&agent_path).map_err(|e| ProjectError::Io(e))?;

    let now = Utc::now();

    let agent_config = Agent {
        uid: uid.clone(),
        name: name.clone(),
        agent_type,
        created_at: now,
        parent_agent_uid: None, // For now, no parent on creation
        model_id: None,
        temperature: None,
        top_p: None,
        default_tools: None,
        context_strategy: None,
    };
    write_json_file(&agent_path.join("config.json"), &agent_config)?;

    // For human agents, we might not need a system_prompt.md or description.md initially
    // For AI agents, these would be created. For now, we'll skip.

    Ok(agent_config)
}

/// Loads an agent from the filesystem given its UID.
pub fn load_agent(
    project: &Project,
    agent_uid: &str,
) -> Result<Agent, ProjectError> {
    let agents_base_path = project.agents_dir();
    let agent_path = get_entity_path(&agents_base_path, agent_uid)?;

    if !agent_path.exists() {
        return Err(ProjectError::AgentNotFound(agent_uid.to_string()));
    }

    let agent_config: Agent = read_json_file(&agent_path.join("config.json"))?;

    Ok(agent_config)
}

/// Lists all agents available in the project.
pub fn list_agents(
    project: &Project,
) -> Result<Vec<Agent>, ProjectError> {
    let agents_base_path = project.agents_dir();
    let mut agents = Vec::new();

    if !agents_base_path.exists() {
        return Ok(agents);
    }

    for entry in fs::read_dir(agents_base_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(uid_str) = path.file_name().and_then(|s| s.to_str()) {
                match load_agent(project, uid_str) {
                    Ok(agent) => agents.push(agent),
                    Err(e) => eprintln!("Warning: Could not load agent {}: {}", uid_str, e),
                }
            }
        }
    }

    Ok(agents)
}
