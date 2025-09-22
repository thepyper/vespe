use anyhow::{anyhow, Result};
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};

// Represents a parsed and validated tool call.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ParsedToolCall {
    pub name: String,
    pub parameters: serde_json::Value,
}

// Defines the contract for a tool call format policy.
pub trait ToolCallPolicy {
    // Returns the unique name of the policy (e.g., "json", "xml").
    fn name(&self) -> &str;

    // Builds the section of the system prompt that instructs the model on the required format.
    fn build_prompt_section(&self, handlebars: &Handlebars) -> Result<String>;

    // Validates the model's output. If valid, returns the parsed tool calls.
    // If invalid, returns a descriptive error.
    fn validate_and_parse(&self, model_output: &str) -> Result<Vec<ParsedToolCall>>;
}


// --- Concrete Implementations ---

/// A policy for handling JSON-based tool calls.
pub struct JsonToolCallPolicy;

impl ToolCallPolicy for JsonToolCallPolicy {
    fn name(&self) -> &str {
        "json"
    }

    fn build_prompt_section(&self, _handlebars: &Handlebars) -> Result<String> {
        Ok(include_str!("../tool-format-tester-templates/json_policy_prompt.hbs").to_string())
    }

    fn validate_and_parse(&self, model_output: &str) -> Result<Vec<ParsedToolCall>> {
        // Find a JSON block enclosed in ```json ... ``` or ``` ... ```
        let json_str = if let Some(captures) = regex::Regex::new(r"```(?:json)?\s*([\s\S]*?)\s*```")?.captures(model_output) {
            captures.get(1).map_or("", |m| m.as_str()).trim()
        } else {
            model_output.trim()
        };

        if json_str.is_empty() {
            return Err(anyhow!("Model output is empty or contains no JSON block."));
        }

        // Attempt to parse as a single tool call
        if let Ok(parsed) = serde_json::from_str::<ParsedToolCall>(json_str) {
            return Ok(vec![parsed]);
        }

        // Attempt to parse as a list of tool calls
        if let Ok(parsed_list) = serde_json::from_str::<Vec<ParsedToolCall>>(json_str) {
            if !parsed_list.is_empty() {
                return Ok(parsed_list);
            }
        }

        Err(anyhow!("Failed to parse model output as a valid JSON tool call or list of tool calls."))
    }
}