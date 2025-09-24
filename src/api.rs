use std::path::Path;
use crate::error::ProjectError;
use crate::tool_models::{Tool, ToolConfig};
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



