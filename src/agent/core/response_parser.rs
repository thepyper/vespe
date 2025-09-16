use anyhow::{anyhow, Result};
use serde_json::Value;
use tracing::info;

use crate::agent::actions::AgentAction;
use crate::config::MalformedJsonHandling;

pub struct ResponseParser;

impl ResponseParser {
    pub fn new() -> Self {
        Self {}
    }

    pub fn parse_response(&self, response_content: &str, handling: &MalformedJsonHandling) -> Result<Vec<AgentAction>> {
        // Try to parse as a Vec<AgentAction>
        if let Ok(actions) = serde_json::from_str::<Vec<AgentAction>>(response_content) {
            return Ok(actions);
        }

        // If parsing fails, handle based on configuration
        match handling {
            MalformedJsonHandling::TreatAsText => {
                info!("Malformed JSON, treating as text: {}", response_content);
                Ok(vec![AgentAction::TextResponse { content: response_content.to_string() }])
            },
            MalformedJsonHandling::Error => {
                Err(anyhow!("LLM response is not valid JSON or does not match expected action format: {}", response_content))
            },
        }
    }
}
