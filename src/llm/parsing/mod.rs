use crate::llm::messages::AssistantContent;
use crate::llm::parsing::parser_trait::{SnippetMatch, SnippetParser};
use std::collections::HashSet;
use crate::llm::parsing::match_source::ParserSource;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::statistics::models::UsageStatistics;

pub mod parser_trait;
pub mod json_parser;
pub mod xml_parser;
pub use crate::llm::parsing::json_parser::{FencedJsonParser, RawJsonObjectParser, RawJsonArrayParser};
pub use crate::llm::parsing::xml_parser::{FencedXmlParser, ToolCodeXmlParser};
pub mod match_source;

/// Parses a raw LLM response string into a vector of `AssistantContent` blocks.
/// It applies a list of snippet parsers to find and extract structured content,
/// leaving the surrounding text as `Text` blocks.
pub async fn parse_response(
    response: &str,
    parsers: &[Box<dyn SnippetParser>],
    global_stats: Arc<Mutex<UsageStatistics>>,
    provider: &str,
    model_id: &str,
) -> (Vec<AssistantContent>, HashSet<ParserSource>) {
    let mut final_blocks: Vec<AssistantContent> = Vec::new();
    let mut used_parsers = HashSet::new();

    // Increment LLM invocation count and handle fading
    let mut stats = global_stats.lock().await;
    let llm_key = format!("{}::{}", provider, model_id);

    let model_entry = stats.model_stats.entry(llm_key.clone()).or_default();
    model_entry.invocations += 1;
    model_entry.fading_invocations += 1.0;

    // Logica di fading: se il conteggio LLM supera la soglia, dimezza tutto
    if model_entry.fading_invocations > 100.0 {
        model_entry.fading_invocations /= 2.0;
        // Dimezza anche tutte le statistiche di parser_stats per questa llm_key
        for (_, parser_stats) in model_entry.parser_stats.iter_mut() {
            parser_stats.fading_usage /= 2.0;
        }
    }
    drop(stats); // Release the lock early

    let mut all_found_matches: Vec<SnippetMatch> = Vec::new();
    for parser in parsers.iter() {
        all_found_matches.extend(parser.find_matches(response));
    }

    // Sort matches by their start position
    all_found_matches.sort_by_key(|m| m.start);

    // Filter out overlapping matches, prioritizing earlier and longer matches
    let mut filtered_matches = Vec::new();
    let mut last_end = 0;
    for m in all_found_matches {
        if m.start >= last_end {
            filtered_matches.push(m);
            last_end = m.end;
        }
    }

    let mut current_text_offset = 0;

    // Process matches and split the response string
    for m in filtered_matches {
        // Add text before the current match
        if m.start > current_text_offset {
            final_blocks.push(AssistantContent::Text(response[current_text_offset..m.start].to_string()));
        }

        // Add the matched content
        final_blocks.push(m.content);
        used_parsers.insert(m.source.clone());

        // Update parser usage statistics
        let mut stats = global_stats.lock().await;
        let llm_key = format!("{}::{}", provider, model_id);
        let parser_entry = stats.model_stats.entry(llm_key.clone()).or_default()
            .parser_stats.entry(m.source.clone()).or_default();
        parser_entry.usage += 1;
        parser_entry.fading_usage += 1.0;
        drop(stats);

        current_text_offset = m.end;
    }

    // Add any remaining text after the last match
    if current_text_offset < response.len() {
        final_blocks.push(AssistantContent::Text(response[current_text_offset..].to_string()));
    }

    (final_blocks, used_parsers)
}