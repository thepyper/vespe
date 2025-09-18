use crate::llm::messages::AssistantContent;
use crate::llm::parsing::parser_trait::SnippetParser;

pub mod parser_trait;
pub mod json_parser;
pub mod xml_parser;

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
pub fn parse_response(response: &str, parsers: &[Box<dyn SnippetParser>]) -> Vec<AssistantContent> {
    let mut blocks: Vec<AssistantContent> = vec![AssistantContent::Text(response.to_string())];

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
    blocks
}
