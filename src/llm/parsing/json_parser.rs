use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::Value;

use crate::llm::messages::{AssistantContent, ToolCall};
use crate::llm::parsing::parser_trait::{SnippetMatch, SnippetParser};

// Regex to find fenced JSON blocks, e.g., ```json
// {...}
// ```
static FENCED_JSON_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?s)```json\s*(?P<json>\{.*?\}|\\\[.*?\])\s*```").unwrap());

pub struct JsonSnippetParser;

impl JsonSnippetParser {
    /// Finds the first valid, raw JSON object or array in a string.
    /// It does this by finding the first '{' or '[' and then using serde's
    /// StreamDeserializer to find where the valid JSON object ends.
    fn find_raw_json<'a>(&self, text: &'a str) -> Option<SnippetMatch<'a>> {
        let first_brace = text.find('{');
        let first_bracket = text.find('[');

        let start_index = match (first_brace, first_bracket) {
            (Some(b), Some(br)) => b.min(br),
            (Some(b), None) => b,
            (None, Some(br)) => br,
            (None, None) => return None,
        };

        // Use `stream.byte_offset()` to get the correct end index for the JSON value.
        // `value.to_string().len()` can be incorrect for multi-byte characters.
        let mut stream = serde_json::Deserializer::from_str(&text[start_index..]).into_iter::<Value>();

        match stream.next() {
            Some(Ok(value)) => {
                let end_index = start_index + stream.byte_offset();
                let tool_call = ToolCall {
                    name: "".to_string(),
                    arguments: value,
                };
                Some(SnippetMatch {
                    start: start_index,
                    end: end_index,
                    content: AssistantContent::ToolCall(tool_call),
                    original_text: &text[start_index..end_index],
                })
            }
            _ => None,
        }
    }
}

impl SnippetParser for JsonSnippetParser {
    fn find_first_match<'a>(&self, text: &'a str) -> Option<SnippetMatch<'a>> {
        // Priority 1: Find a fenced JSON block.
        if let Some(captures) = FENCED_JSON_REGEX.captures(text) {
            if let Some(json_match) = captures.name("json") {
                if let Ok(value) = serde_json::from_str::<Value>(json_match.as_str()) {
                    let tool_call = ToolCall {
                        name: "".to_string(),
                        arguments: value,
                    };
                    // The match is the entire ```json...``` block
                    let full_match = captures.get(0).unwrap();
                    return Some(SnippetMatch {
                        start: full_match.start(),
                        end: full_match.end(),
                        content: AssistantContent::ToolCall(tool_call),
                        original_text: full_match.as_str(),
                    });
                }
            }
        }

        // Priority 2: Find a raw JSON object or array.
        self.find_raw_json(text)
    }
}
