use crate::llm::messages::AssistantContent;
use crate::llm::parsing::parser_trait::SnippetParser;
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

/// Finds the first match from any of the provided parsers in the text.
/// This is a helper function for the main parsing loop.
fn find_first_match_in_text<'a>(
    text: &'a str,
    parsers: &[Box<dyn SnippetParser>],
) -> Option<parser_trait::SnippetMatch<'a>> {
    parsers
        .iter()
        .filter_map(|parser| parser.find_first_match(text))
        .min_by_key(|m| m.start)
}

/// Parses a raw LLM response string into a vector of `AssistantContent` blocks.
/// It iteratively applies a list of snippet parsers to find and extract structured content,
/// leaving the surrounding text as `Text` blocks.
pub async fn parse_response(
    response: &str,
    parsers: &[Box<dyn SnippetParser>],
    global_stats: Arc<Mutex<UsageStatistics>>,
    provider: &str,
    model_id: &str,
) -> (Vec<AssistantContent>, HashSet<ParserSource>) {
    let mut blocks: Vec<AssistantContent> = vec![AssistantContent::Text(response.to_string())];
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

    loop {
        let mut split_occurred = false;
        let mut next_blocks = Vec::new();
        let mut blocks_iter = blocks.into_iter();

        while let Some(block) = blocks_iter.next() {
            if let AssistantContent::Text(text) = block {
                if let Some(m) = find_first_match_in_text(&text, parsers) {
                    // A split occurs!
                    let pre_text = &text[..m.start];
                    if !pre_text.is_empty() {
                        next_blocks.push(AssistantContent::Text(pre_text.to_string()));
                    }

                    next_blocks.push(m.content);
                    used_parsers.insert(m.source.clone()); // Record the parser source

                    // Update parser usage statistics
                    let mut stats = global_stats.lock().await;
                    let llm_key = format!("{}::{}", provider, model_id);

                    let parser_entry = stats.model_stats.entry(llm_key.clone()).or_default()
                        .parser_stats.entry(m.source.clone()).or_default();
                    parser_entry.usage += 1;
                    parser_entry.fading_usage += 1.0;

                    drop(stats); // Release the lock early

                    let post_text = &text[m.end..];
                    if !post_text.is_empty() {
                        next_blocks.push(AssistantContent::Text(post_text.to_string()));
                    }

                    // Append the rest of the blocks and restart the main loop
                    next_blocks.extend(blocks_iter);
                    split_occurred = true;
                    break; // Exit the 'while let' to restart the 'loop'
                } else {
                    // No match, keep the block as is
                    next_blocks.push(AssistantContent::Text(text));
                }
            } else {
                // Not a Text block, keep it as is
                next_blocks.push(block);
            }
        }

        blocks = next_blocks;

        if !split_occurred {
            break; // No splits in a full pass, we are done.
        }
    }
    (blocks, used_parsers)
}