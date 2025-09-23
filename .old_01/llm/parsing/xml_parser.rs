use once_cell::sync::Lazy;
use regex::Regex;
use serde_xml_rs::from_str;

use crate::llm::messages::{AssistantContent, ToolCall};
use crate::llm::parsing::match_source::{ParserSource, XmlMatchMode};
use crate::llm::parsing::parser_trait::{SnippetMatch, SnippetParser};

// Regex to find fenced XML blocks, e.g., ```xml
// <tool_code>...</tool_code>
// ```
static FENCED_CODE_BLOCK_START: &str = "```xml";
static FENCED_CODE_BLOCK_END: &str = "```";

// Regex to find <tool_code>...</tool_code> blocks
static TOOL_CODE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?s)<tool_code>(.*?)</tool_code>").unwrap()
});

pub struct FencedXmlParser;

impl FencedXmlParser {
    /// Finds the first fenced XML block and extracts its content.
    fn find_fenced_xml<'a>(text: &'a str) -> Option<SnippetMatch<'a>> {
        let start_marker_pos = text.find(FENCED_CODE_BLOCK_START)?;
        let content_start = start_marker_pos + FENCED_CODE_BLOCK_START.len();

        let end_marker_pos = text[content_start..].find(FENCED_CODE_BLOCK_END)?;
        let content_end = content_start + end_marker_pos;

        let xml_content = &text[content_start..content_end];

        if let Ok(tool_call) = from_str::<ToolCall>(xml_content) {
            Some(SnippetMatch {
                start: start_marker_pos,
                end: content_end + FENCED_CODE_BLOCK_END.len(),
                content: AssistantContent::ToolCall(tool_call),
                source: ParserSource::Xml(XmlMatchMode::FencedCodeBlock),
                original_text: &text[start_marker_pos..content_end + FENCED_CODE_BLOCK_END.len()],
            })
        } else {
            None
        }
    }
}

impl SnippetParser for FencedXmlParser {
    fn find_first_match<'a>(&self, text: &'a str) -> Option<SnippetMatch<'a>> {
        FencedXmlParser::find_fenced_xml(text)
    }
}

pub struct ToolCodeXmlParser;

impl ToolCodeXmlParser {
    /// Finds the first <tool_code>...</tool_code> block and extracts its content.
    fn find_tool_code_xml<'a>(text: &'a str) -> Option<SnippetMatch<'a>> {
        if let Some(captures) = TOOL_CODE_REGEX.captures(text) {
            if let Some(full_match) = captures.get(0) {
                let xml_content = captures.get(1)?.as_str(); // Content inside <tool_code>

                if let Ok(tool_call) = from_str::<ToolCall>(xml_content) {
                    Some(SnippetMatch {
                        start: full_match.start(),
                        end: full_match.end(),
                        content: AssistantContent::ToolCall(tool_call),
                        source: ParserSource::Xml(XmlMatchMode::ToolCodeBlock),
                        original_text: full_match.as_str(),
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

impl SnippetParser for ToolCodeXmlParser {
    fn find_first_match<'a>(&self, text: &'a str) -> Option<SnippetMatch<'a>> {
        ToolCodeXmlParser::find_tool_code_xml(text)
    }
}