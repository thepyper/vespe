use crate::llm::messages::AssistantContent;
use crate::llm::parsing::match_source::ParserSource;

// Represents a match found by the parser in the text
#[derive(Debug, Clone)]
pub struct SnippetMatch<'a> {
    pub start: usize,
    pub end: usize,
    pub content: AssistantContent,
    pub source: ParserSource,
    // The original text that generated the match
    pub original_text: &'a str,
}

pub trait SnippetParser: Send + Sync {
    // Finds all valid matches in the given text
    fn find_matches<'a>(&self, text: &'a str) -> Vec<SnippetMatch<'a>>;
}
