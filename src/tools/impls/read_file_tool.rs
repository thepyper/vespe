use anyhow::{Context, Result};
use async_trait::async_trait;
use serde_json::{json, Value};
use tokio::fs;

use crate::tools::tool_trait::Tool;

pub struct ReadFileTool;

#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "Reads the content of a specified file from the project sandbox. Input is a JSON object with a 'path' string field."
    }

    fn input_schema(&self) -> &str {
        r#"{
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The path to the file to read, relative to the project sandbox."
                }
            },
            "required": ["path"]
        }"#
    }

    fn output_schema(&self) -> &str {
        r#"{
            "type": "object",
            "properties": {
                "content": {
                    "type": "string",
                    "description": "The content of the file."
                }
            },
            "required": ["content"]
        }"#
    }

    async fn execute(&self, input: &Value) -> Result<Value> {
        let file_path_str = input["path"].as_str().context("Missing 'path' field in input for read_file tool.")?;
        
        // Construct the absolute path relative to the project sandbox
        let base_path = std::path::PathBuf::from("sandbox");
        let full_path = base_path.join(file_path_str);

        let content = fs::read_to_string(&full_path)
            .await
            .with_context(|| format!("Failed to read file from path: {:?}", full_path))?;

        Ok(json!({ "content": content }))
    }
}
