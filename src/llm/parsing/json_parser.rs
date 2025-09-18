use once_cell::sync::Lazy;
use regex::Regex;
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

pub struct JsonSnippetParser;

impl JsonSnippetParser {
    /// Finds the first valid, raw JSON object or array in a string.
    fn find_raw_json<'a>(&self, text: &'a str) -> Option<SnippetMatch<'a>> {
        let first_brace = text.find('{');
        let first_bracket = text.find('[');

        let start_index = match (first_brace, first_bracket) {
            (Some(b), Some(br)) => b.min(br),
            (Some(b), None) => b,
            (None, Some(br)) => br,
            (None, None) => return None,
        };

        let mut stream = serde_json::Deserializer::from_str(&text[start_index..]).into_iter::<Value>();

        match stream.next() {
            Some(Ok(value)) => {
                let end_index = start_index + stream.byte_offset();
                let tool_call = ToolCall {
                    name: "".to_string(),
                    arguments: value.clone(), // Clone value for the match_mode check
                };

                let match_mode = if value.is_object() {
                    JsonMatchMode::RawObject
                } else if value.is_array() {
                    JsonMatchMode::RawArray
                } else {
                    return None; // Should not happen if we only look for { or [
                };

                Some(SnippetMatch {
                    start: start_index,
                    end: end_index,
                    content: AssistantContent::ToolCall(tool_call),
                    source: ParserSource::Json(match_mode),
                    original_text: &text[start_index..end_index],
                })
            }
            _ => None,
        }
    }

    /// Finds the first fenced JSON block and extracts its content.
    fn find_fenced_json<'a>(&self, text: &'a str) -> Option<SnippetMatch<'a>> {
        let start_marker_pos = text.find(FENCED_CODE_BLOCK_START)?;
        let content_start = start_marker_pos + FENCED_CODE_BLOCK_START.len();

        let end_marker_pos = text[content_start..].find(FENCED_CODE_BLOCK_END)?;
        let content_end = content_start + end_marker_pos;

        let json_content = &text[content_start..content_end];

        if let Ok(value) = serde_json::from_str::<Value>(json_content) {
            let tool_call = ToolCall {
                name: "".to_string(),
                arguments: value,
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
    }
}

impl SnippetParser for JsonSnippetParser {
    fn find_first_match<'a>(&self, text: &'a str) -> Option<SnippetMatch<'a>> {
        // Priority 1: Find a fenced JSON block.
        if let Some(fenced_match) = self.find_fenced_json(text) {
            return Some(fenced_match);
        }

        // Priority 2: Find a raw JSON object or array.
        self.find_raw_json(text)
    }
}
