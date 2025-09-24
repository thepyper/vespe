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
