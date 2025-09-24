use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};
use crate::error::ProjectError;
use crate::utils::{generate_uid, get_entity_path, write_json_file, write_file_content};
use std::fs;

// Corresponds to config.json for a Tool
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolConfig {
    pub uid: String,
    pub name: String,
    pub description: String,
    pub schema: serde_json::Value, // JSON Schema for inputs/outputs
    pub implementation_details: serde_json::Value, // Details on how to execute (type, path, entrypoint, args)
}

// Represents a Tool loaded in memory
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tool {
    pub uid: String,
    pub root_path: PathBuf, // Path to tool-UID/ or kit-UID/tool-UID/
    pub config: ToolConfig,
}

impl Tool {
    /// Creates a new tool.
    pub fn create(
        name: String,
        description: String,
        schema: serde_json::Value,
        implementation_details: serde_json::Value,
        tools_base_path: &Path,
    ) -> Result<Self, ProjectError> {
        let uid = generate_uid("tool")?;
        let tool_path = get_entity_path(tools_base_path, &uid)?;

        std::fs::create_dir_all(&tool_path).map_err(|e| ProjectError::Io(e))?;

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
}
