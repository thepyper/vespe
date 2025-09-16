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
        "Echoes back the input string, transformed in three ways: lowercase, capitalized, and uppercase, separated by asterisks."
    }

    fn input_schema(&self) -> &str {
        r#"{
            "type": "object",
            "properties": {
                "text": {
                    "type": "string",
                    "description": "The string to echo back and transform."
                }
            },
            "required": ["text"]
        }"#
    }

    async fn execute(&self, input: &Value) -> Result<Value> {
        let text = input["text"].as_str().unwrap_or("Invalid input");

        let lowercase = text.to_lowercase();
        let capitalized = {
            let mut c = text.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        };
        let uppercase = text.to_uppercase();

        let result = format!(
            "*{}* *{}* *{}*",
            lowercase,
            capitalized,
            uppercase
        );

        Ok(json!({ "transformed_text": result }))
    }
}