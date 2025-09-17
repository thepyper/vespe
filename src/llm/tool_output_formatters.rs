use anyhow::Result;
use serde_json::{Value, to_string_pretty};
use crate::llm::policy_types::PolicyType;
use crate::llm::tool_types::ToolType;

pub fn format_tool_output(tool_type: ToolType, result: &Value, policy_type: PolicyType) -> Result<String> {
    match tool_type {
        ToolType::WebSearch => {
            match policy_type {
                PolicyType::Json => {
                    if let Some(results) = result.get("results").and_then(|r| r.as_array()) {
                        let formatted_results: Vec<String> = results.iter()
                            .take(3) // Limita a primi 3 risultati
                            .filter_map(|r| {
                                let title = r.get("title")?.as_str()?;
                                let snippet = r.get("snippet")?.as_str()?;
                                Some(format!(