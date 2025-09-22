use anyhow::Result;
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
