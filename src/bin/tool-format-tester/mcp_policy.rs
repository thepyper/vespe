use anyhow::{anyhow, Result};
use handlebars::Handlebars;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::policies::{ParsedToolCall, StructuredOutputBlock, ToolCallPolicy};

pub struct McpPolicy;

impl ToolCallPolicy for McpPolicy {
    fn name(&self) -> &str {
        "mcp"
    }

    fn build_prompt_section(&self, _handlebars: &Handlebars) -> Result<String> {
        Ok(include_str!("../tool-format-tester-templates/mcp_policy_prompt.hbs").to_string())
    }

    fn validate_and_parse(&self, model_output: &str) -> Result<Vec<StructuredOutputBlock>> {
        let mut blocks = Vec::new();
        let re = Regex::new(r"^(?i)(TEXT|THOUGHT|TOOL_CALL):\s*(.*)")?;

        for line in model_output.lines() {
            if line.trim().is_empty() {
                continue;
            }
            if let Some(caps) = re.captures(line) {
                let block_type = caps.get(1).map_or("", |m| m.as_str()).to_uppercase();
                let content = caps.get(2).map_or("", |m| m.as_str()).trim();

                match block_type.as_str() {
                    "TEXT" => blocks.push(StructuredOutputBlock::Text(content.to_string())),
                    "THOUGHT" => blocks.push(StructuredOutputBlock::Thought(content.to_string())),
                    "TOOL_CALL" => {
                        match serde_json::from_str(content) {
                            Ok(parsed_tool_call) => blocks.push(StructuredOutputBlock::ToolCall(parsed_tool_call)),
                            Err(e) => return Err(anyhow!("Failed to parse TOOL_CALL JSON: {} on line: {}", e, line)),
                        }
                    }
                    _ => {}
                }
            } else {
                // If a line doesn't match the pattern, it's a validation failure.
                return Err(anyhow!("Invalid line format found: {}", line));
            }
        }
        
        if blocks.is_empty() {
            return Err(anyhow!("No valid MCP blocks found in the model output."));
        }

        Ok(blocks)
    }
}
