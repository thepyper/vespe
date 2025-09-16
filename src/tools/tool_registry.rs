use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;
use serde_json::Value;

use crate::tools::tool_trait::Tool;

#[derive(Clone)]
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool + Send + Sync>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register_tool(&mut self, tool: Arc<dyn Tool + Send + Sync>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    pub fn get_tool_metadata(&self) -> Vec<Value> {
        self.tools.values().map(|tool| {
            serde_json::json!({
                "name": tool.name(),
                "description": tool.description(),
                "input_schema": serde_json::from_str::<Value>(tool.input_schema()).unwrap_or_default(),
            })
        }).collect()
    }

    pub async fn execute_tool(&self, tool_name: &str, input: &Value) -> Result<Value> {
        let tool = self.tools.get(tool_name)
            .ok_or_else(|| anyhow!("Tool '{}' not found in registry", tool_name))?;
        tool.execute(input).await
    }
}