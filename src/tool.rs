use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};
use serde_json::{Value, json};
use crate::error::ProjectError;
use crate::utils::{generate_uid, get_entity_path, write_json_file, write_file_content};
use async_trait::async_trait;

// Corresponds to config.json for a Tool
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolConfig {
    pub uid: String,
    pub name: String,
    pub description: String,
    pub schema: serde_json::Value, // JSON Schema for inputs/outputs
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
        };
        write_json_file(&tool_path.join("config.json"), &config)?;
        write_file_content(&tool_path.join("description.md"), &description)?;

        Ok(Tool {
            uid: uid.clone(),
            root_path: tool_path,
            config,
        })
    }

    /// Loads a tool from the filesystem given its UID.
    pub fn from_path(
        tool_path: &Path        
    ) -> Result<Self, ProjectError> {

        if !tool_path.exists() {
            return Err(ProjectError::ToolNotFound("sticazzi TODO".to_string()));
        }

        let config: ToolConfig = crate::utils::read_json_file(&tool_path.join("config.json"))?;
        // description.md is not loaded into the Tool struct directly, but can be read on demand.

        Ok(Tool {
            uid: config.uid.clone(),
            root_path: tool_path.into(),
            config,
        })
    }

    /// Executes the tool with the given inputs.
    /// This is a placeholder implementation.
    pub async fn execute(&self, inputs: Value) -> Result<Value, ProjectError> {
        match self.config.name.as_str() {
            "dummy_tool_1" => {
                println!("Executing dummy_tool_1 with inputs: {:?}", inputs);
                Ok(json!({"status": "success", "output": "Dummy tool 1 executed successfully."}))
            },
            "dummy_tool_2" => {
                println!("Executing dummy_tool_2 with inputs: {:?}", inputs);
                Ok(json!({"status": "success", "output": "Dummy tool 2 executed successfully."}))
            },
            _ => Err(ProjectError::ToolExecutionError(format!("Tool '{}' not implemented yet.", self.config.name))),
        }
    }
}