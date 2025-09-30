use anyhow::Result;
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};

// Represents a single, parsed tool call.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ParsedToolCall {
    pub name: String,
    pub parameters: serde_json::Value,
}

// Represents a block of structured output from the model.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum StructuredOutputBlock {
    Text(String),
    Thought(String),
    ToolCall(ParsedToolCall),
}

// Defines the contract for a model output parsing policy.
pub trait ToolCallPolicy {
    // Returns the unique name of the policy (e.g., "mcp").
    fn name(&self) -> &str;

    // Builds the section of the system prompt that instructs the model on the required format.
    fn build_prompt_section(&self, handlebars: &Handlebars) -> Result<String>;

    // Validates the model's output. If valid, returns an ordered list of structured blocks.
    // If invalid, returns a descriptive error.
    fn validate_and_parse(&self, model_output: &str) -> Result<Vec<StructuredOutputBlock>>;
}

// --- Concrete Implementations ---

/*
  NOTE: Concrete policy implementations are now in separate files (e.g., mcp_policy.rs, tagged_policy.rs).
*/
