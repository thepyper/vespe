use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};

use crate::tools::tool_trait::Tool;

pub struct EchoTool;

#[async_trait]
impl Tool for EchoTool {
    fn name(&self) -> &str {
        "echo"
    }

    fn description(&self) -> &str {
        "Echoes back the input string. Useful for testing or simple confirmations."
    }

    fn input_schema(&self) -> &str {
        r#"{
            "type": "object",
            "properties": {
                "text": {
                    "type": "string",
                    "description": "The string to echo back."
                }
            },
            "required": ["text"]
        }"#
    }

    async fn execute(&self, input: &Value) -> Result<Value> {
        let text = input["text"].as_str().unwrap_or("Invalid input");
        Ok(json!({ "echoed_text": text.to_string() }))
    }
}
