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
                match project.load_agent(uid_str) {
                    Ok(agent) => agents.push(agent),
                    Err(e) => eprintln!("Warning: Could not load agent {}: {}", uid_str, e),
                }
            }
        }
    }

    Ok(agents)
}
