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
    /// Finds the first fenced JSON block and extracts its content.
    fn find_fenced_json<'a>(text: &'a str) -> Option<SnippetMatch<'a>> {
        let start_marker_pos = text.find(FENCED_CODE_BLOCK_START)?;
        let content_start = start_marker_pos + FENCED_CODE_BLOCK_START.len();

        let end_marker_pos = text[content_start..].find(FENCED_CODE_BLOCK_END)?;
        let content_end = content_start + end_marker_pos;

        let json_content = &text[content_start..content_end].trim();

        if let Ok(mut value) = serde_json::from_str::<Value>(json_content) {
            if let Some(tool_code_obj) = value.as_object_mut()?.remove("tool_code") {
                if let Some(tool_code_map) = tool_code_obj.as_object() {
                    let name = tool_code_map.get("name")?.as_str()?.to_string();
                    let arguments = tool_code_map.get("arguments")?.clone();

                    let tool_call = ToolCall {
                        name,
                        arguments,
                    };
                    Some(SnippetMatch {
                        start: start_marker_pos,
                        end: content_end + FENCED_CODE_BLOCK_END.len(),
                        content: AssistantContent::ToolCall(tool_call),
                        source: ParserSource::Json(JsonMatchMode::FencedCodeBlock),
                        original_text: &text[start_marker_pos..content_end + FENCED_CODE_BLOCK_END.len()],
                    })
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl SnippetParser for FencedJsonParser {
    fn find_first_match<'a>(&self, text: &'a str) -> Option<SnippetMatch<'a>> {
        FencedJsonParser::find_fenced_json(text)
    }
}

pub struct RawJsonObjectParser;

impl RawJsonObjectParser {
    /// Finds the first valid, raw JSON object in a string.
    fn find_raw_json_object<'a>(text: &'a str) -> Option<SnippetMatch<'a>> {
        let first_brace = text.find('{')?;
        let trimmed_text = &text[first_brace..].trim();
        let mut stream = serde_json::Deserializer::from_str(trimmed_text).into_iter::<Value>();

        if let Some(Ok(mut value)) = stream.next() {
            if value.is_object() {
                if let Some(tool_code_obj) = value.as_object_mut()?.remove("tool_code") {
                    if let Some(tool_code_map) = tool_code_obj.as_object() {
                        let name = tool_code_map.get("name")?.as_str()?.to_string();
                        let arguments = tool_code_map.get("arguments")?.clone();

                        let end_index = first_brace + stream.byte_offset();
                        let tool_call = ToolCall {
                            name,
                            arguments,
                        };
                        Some(SnippetMatch {
                            start: first_brace,
                            end: end_index,
                            content: AssistantContent::ToolCall(tool_call),
                            source: ParserSource::Json(JsonMatchMode::RawObject),
                            original_text: &text[first_brace..end_index],
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl SnippetParser for RawJsonObjectParser {
    fn find_first_match<'a>(&self, text: &'a str) -> Option<SnippetMatch<'a>> {
        RawJsonObjectParser::find_raw_json_object(text)
    }
}

pub struct RawJsonArrayParser;

impl RawJsonArrayParser {
    /// Finds the first valid, raw JSON array in a string.
    fn find_raw_json_array<'a>(text: &'a str) -> Option<SnippetMatch<'a>> {
        let first_bracket = text.find('[')?;
        let trimmed_text = &text[first_bracket..].trim();
        let mut stream = serde_json::Deserializer::from_str(trimmed_text).into_iter::<Value>();

        match stream.next() {
            Some(Ok(mut value)) => {
                if value.is_array() {
                    if let Some(tool_code_obj) = value.as_object_mut()?.remove("tool_code") {
                        if let Some(tool_code_map) = tool_code_obj.as_object() {
                            let name = tool_code_map.get("name")?.as_str()?.to_string();
                            let arguments = tool_code_map.get("arguments")?.clone();

                            let end_index = first_bracket + stream.byte_offset();
                            let tool_call = ToolCall {
                                name,
                                arguments,
                            };
                            Some(SnippetMatch {
                                start: first_bracket,
                                end: end_index,
                                content: AssistantContent::ToolCall(tool_call),
                                source: ParserSource::Json(JsonMatchMode::RawArray),
                                original_text: &text[first_bracket..end_index],
                            })
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl SnippetParser for RawJsonArrayParser {
    fn find_first_match<'a>(&self, text: &'a str) -> Option<SnippetMatch<'a>> {
        RawJsonArrayParser::find_raw_json_array(text)
    }
}