use anyhow::{anyhow, Result};
use handlebars::Handlebars;
use regex::Regex;

use crate::policies::{ParsedToolCall, StructuredOutputBlock, ToolCallPolicy};

pub struct TaggedPolicy;

impl ToolCallPolicy for TaggedPolicy {
    fn name(&self) -> &str {
        "tagged"
    }

    fn build_prompt_section(&self, _handlebars: &Handlebars) -> Result<String> {
        Ok(include_str!("../tool-format-tester-templates/tagged_policy_prompt.hbs").to_string())
    }

    fn validate_and_parse(&self, model_output: &str) -> Result<Vec<StructuredOutputBlock>> {
        let mut blocks = Vec::new();
        let re_section = Regex::new(r###"(?s)<[$]@(TEXT|THOUGHT|TOOL_CALL)@$>(.*?)</[$]@\1@$>"###)?;
        //let re_param = Regex::new(r"(\w+):\s*""([^"]*)""")?; // For parsing key: "value" pairs
        let re_param = Regex::new(r###"(\w+):\s*"([^"]*)""###)?; // For parsing key: "value" pairs

        for cap in re_section.captures_iter(model_output) {
            let tag_type = cap.get(1).unwrap().as_str();
            let content = cap.get(2).unwrap().as_str().trim();

            match tag_type {
                "TEXT" => blocks.push(StructuredOutputBlock::Text(content.to_string())),
                "THOUGHT" => blocks.push(StructuredOutputBlock::Thought(content.to_string())),
                "TOOL_CALL" => {
                    // Parse function-like tool call: tool_name(param: "value", ...)
                    //let re_func_call = Regex::new("(\\\\w+)\\\\((.*)\\\\)")?;
                    let re_func_call = Regex::new(r###"(\w+)\((.*)\)"###)?;
                    if let Some(func_caps) = re_func_call.captures(content) {
                        let tool_name = func_caps.get(1).unwrap().as_str().to_string();
                        let params_str = func_caps.get(2).unwrap().as_str();

                        let mut parameters = serde_json::Map::new();
                        for param_cap in re_param.captures_iter(params_str) {
                            let key = param_cap.get(1).unwrap().as_str().to_string();
                            let value = param_cap.get(2).unwrap().as_str().to_string();
                            parameters.insert(key, serde_json::Value::String(value));
                        }
                        blocks.push(StructuredOutputBlock::ToolCall(ParsedToolCall {
                            name: tool_name,
                            parameters: serde_json::Value::Object(parameters),
                        }));
                    } else {
                        return Err(anyhow!("Invalid TOOL_CALL format: {}", content));
                    }
                }
                _ => {} // Should not happen due to regex
            }
        }

        if blocks.is_empty() {
            return Err(anyhow!("No valid tagged blocks found in the model output."));
        }

        Ok(blocks)
    }
}