use serde_json::Value;

use crate::llm::messages::{AssistantContent, ToolCall};
use crate::llm::parsing::match_source::{JsonMatchMode, ParserSource};
use crate::llm::parsing::parser_trait::{SnippetMatch, SnippetParser};

// Regex to find fenced JSON blocks, e.g., ```json
// {...}
// ```
// This regex is now only for finding the markers, not validating content.
static FENCED_CODE_BLOCK_START: &str = "```json";
static FENCED_CODE_BLOCK_END: &str = "```";

pub struct FencedJsonParser;

impl FencedJsonParser {
    /// Finds fenced JSON blocks and extracts their content, handling arrays of tool calls.
    fn find_fenced_json<'a>(text: &'a str) -> Vec<SnippetMatch<'a>> {
        let mut matches = Vec::new();
        let start_marker_pos = if let Some(pos) = text.find(FENCED_CODE_BLOCK_START) { pos } else { return matches; };

        let content_start = start_marker_pos + FENCED_CODE_BLOCK_START.len();

        let end_marker_pos = if let Some(pos) = text[content_start..].find(FENCED_CODE_BLOCK_END) { pos } else { return matches; };

        let json_content_str = &text[content_start..content_start + end_marker_pos].trim();

        if let Ok(mut value) = serde_json::from_str::<Value>(json_content_str) {
            if let Some(arr) = value.as_array_mut() {
                for item in arr {
                    if let Some(tool_code_obj) = item.as_object_mut().and_then(|obj| obj.remove("tool_code")) {
                        if let Some(tool_code_map) = tool_code_obj.as_object() {
                            if let (Some(name_val), Some(args_val)) = (tool_code_map.get("name"), tool_code_map.get("arguments")) {
                                if let Some(name_str) = name_val.as_str() {
                                    let name = name_str.to_string();
                                    let arguments = args_val.clone();

                                    let tool_call = ToolCall { name, arguments };
                                    matches.push(SnippetMatch {
                                        start: start_marker_pos,
                                        end: content_start + end_marker_pos + FENCED_CODE_BLOCK_END.len(),
                                        content: AssistantContent::ToolCall(tool_call),
                                        source: ParserSource::Json(JsonMatchMode::FencedCodeBlock),
                                        original_text: &text[start_marker_pos..content_start + end_marker_pos + FENCED_CODE_BLOCK_END.len()],
                                    });
                                }
                            }
                        }
                    }
                }
            } else if let Some(tool_code_obj) = value.as_object_mut().and_then(|obj| obj.remove("tool_code")) {
                if let Some(tool_code_map) = tool_code_obj.as_object() {
                    if let (Some(name_val), Some(args_val)) = (tool_code_map.get("name"), tool_code_map.get("arguments")) {
                        if let Some(name_str) = name_val.as_str() {
                            let name = name_str.to_string();
                            let arguments = args_val.clone();

                            let tool_call = ToolCall { name, arguments };
                            matches.push(SnippetMatch {
                                start: start_marker_pos,
                                end: content_start + end_marker_pos + FENCED_CODE_BLOCK_END.len(),
                                content: AssistantContent::ToolCall(tool_call),
                                source: ParserSource::Json(JsonMatchMode::FencedCodeBlock),
                                original_text: &text[start_marker_pos..content_start + end_marker_pos + FENCED_CODE_BLOCK_END.len()],
                            });
                        }
                    }
                }
			}
		}
		matches
	

pub struct RawJsonObjectParser;

impl RawJsonObjectParser {
    /// Finds the first valid, raw JSON object in a string.
    fn find_raw_json_object<'a>(text: &'a str) -> Option<SnippetMatch<'a>> {
        let first_brace = if let Some(pos) = text.find('{') { pos } else { return None; };
        let trimmed_text = &text[first_brace..].trim();
        let mut stream = serde_json::Deserializer::from_str(trimmed_text).into_iter::<Value>();

        if let Some(Ok(mut value)) = stream.next() {
            if value.is_object() {
                if let Some(tool_code_obj) = value.as_object_mut().and_then(|obj| obj.remove("tool_code")) {
                    if let Some(tool_code_map) = tool_code_obj.as_object() {
                        if let (Some(name_val), Some(args_val)) = (tool_code_map.get("name"), tool_code_map.get("arguments")) {
                            if let Some(name_str) = name_val.as_str() {
                                let name = name_str.to_string();
                                let arguments = args_val.clone();

                                let end_index = first_brace + stream.byte_offset();
                                let tool_call = ToolCall {
                                    name,
                                    arguments,
                                };
                                return Some(SnippetMatch {
                                    start: first_brace,
                                    end: end_index,
                                    content: AssistantContent::ToolCall(tool_call),
                                    source: ParserSource::Json(JsonMatchMode::RawObject),
                                    original_text: &text[first_brace..end_index],
                                });
                            }
                        }
                    }
                }
            }
        }
        None
    }
}

pub struct RawJsonArrayParser;

impl RawJsonArrayParser {
    /// Finds the first valid, raw JSON array in a string.
    fn find_raw_json_array<'a>(text: &'a str) -> Vec<SnippetMatch<'a>> {
        let mut matches = Vec::new();
        let first_bracket = if let Some(pos) = text.find('[') { pos } else { return matches; };
        let trimmed_text = &text[first_bracket..].trim();
        let mut stream = serde_json::Deserializer::from_str(trimmed_text).into_iter::<Value>();

        if let Some(Ok(mut value)) = stream.next() {
            if let Some(arr) = value.as_array_mut() {
                for item in arr {
                    if let Some(tool_code_obj) = item.as_object_mut().and_then(|obj| obj.remove("tool_code")) {
                        if let Some(tool_code_map) = tool_code_obj.as_object() {
                            if let (Some(name_val), Some(args_val)) = (tool_code_map.get("name"), tool_code_map.get("arguments")) {
                                if let Some(name_str) = name_val.as_str() {
                                    let name = name_str.to_string();
                                    let arguments = args_val.clone();

                                    let end_index = first_bracket + stream.byte_offset();
                                    let tool_call = ToolCall {
                                        name,
                                        arguments,
                                    };
                                    matches.push(SnippetMatch {
                                        start: first_bracket,
                                        end: end_index,
                                        content: AssistantContent::ToolCall(tool_call),
                                        source: ParserSource::Json(JsonMatchMode::RawArray),
                                        original_text: &text[first_bracket..end_index],
                                    });
                                }
                            }
                        }
                    }
                }
            } else if let Some(tool_code_obj) = value.as_object_mut().and_then(|obj| obj.remove("tool_code")) {
                if let Some(tool_code_map) = tool_code_obj.as_object() {
                    if let (Some(name_val), Some(args_val)) = (tool_code_map.get("name"), tool_code_map.get("arguments")) {
                        if let Some(name_str) = name_val.as_str() {
                            let name = name_str.to_string();
                            let arguments = args_val.clone();

                            let end_index = first_bracket + stream.byte_offset();
                            let tool_call = ToolCall {
                                name,
                                arguments,
                            };
                            matches.push(SnippetMatch {
                                start: first_bracket,
                                end: end_index,
                                content: AssistantContent::ToolCall(tool_call),
                                source: ParserSource::Json(JsonMatchMode::RawArray),
                                original_text: &text[first_bracket..end_index],
                            });
                        }
                    }
                }
            }
        }
        matches
    }
}