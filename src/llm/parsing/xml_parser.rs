use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::Value;

use crate::llm::messages::{AssistantContent, ToolCall};
use crate::llm::parsing::match_source::{ParserSource, XmlMatchMode};
use crate::llm::parsing::parser_trait::{SnippetMatch, SnippetParser};

// This regex is designed to find XML-like <tool_call> blocks.
// It assumes the content within the block is JSON.
static XML_TOOL_CALL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?s)<tool_call>\s*(?P<json>\{{.*?\}})\s*</tool_call>"#).unwrap());

pub struct XmlSnippetParser;

impl SnippetParser for XmlSnippetParser {
    fn find_first_match<'a>(&self, text: &'a str) -> Option<SnippetMatch<'a>> {
        XML_TOOL_CALL_REGEX.captures(text).and_then(|captures| {
            captures.name("json").and_then(|json_match| {
                serde_json::from_str::<Value>(json_match.as_str()).ok().map(|value| {
                    let tool_call = ToolCall {
                        // The name is often inside the JSON for this format
                        name: value["name"].as_str().unwrap_or("").to_string(),
                        arguments: value["arguments"].clone(),
                    };
                    let full_match = captures.get(0).unwrap();
                    SnippetMatch {
                        start: full_match.start(),
                        end: full_match.end(),
                        content: AssistantContent::ToolCall(tool_call),
                        source: ParserSource::Xml(XmlMatchMode::ToolCallTag),
                        original_text: full_match.as_str(),
                    }
                })
            })
        })
    }
}