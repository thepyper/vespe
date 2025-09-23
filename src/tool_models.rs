use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

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
