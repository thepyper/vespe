use super::Parser;
use crate::ast2::{Ast2Error, Content, Document, Range, Result, Text};

/// Parses a string slice into a `Document` AST.
///
/// This is the main entry point for the parser.
///
/// # Arguments
///
/// * `document` - A string slice representing the source document to be parsed.
///
/// # Returns
///
/// A `Result` containing the parsed `Document` or an `Ast2Error` on failure.
pub fn parse_document(document: &str) -> Result<Document> {
    let parser = Parser::new(document);
    let begin = parser.get_position();
    let (content, parser_after_content) = parse_content(parser)?;
    let end = parser_after_content.get_position();

    Ok(Document {
        content: content,
        range: Range { begin, end },
    })
}

pub(crate) fn parse_content<'doc>(parser: Parser<'doc>) -> Result<(Vec<Content>, Parser<'doc>)> {
    let mut contents = Vec::new();
    let mut p_current = parser; // Takes ownership

    // The core parsing loop. It processes the document line by line.
    // Each line must start at column 1 and is attempted to be parsed as a Tag,
    // an Anchor, or plain Text, in that order of precedence.
    // If none of these match, and the line is not empty, it's a parsing error.
    while !p_current.is_eod() {
        // Subroutines must always stop at end-of-line, otherwise we have a problem
        if !p_current.is_begin_of_line() {
            return Err(Ast2Error::ExpectedBeginOfLine {
                position: p_current.get_position(),
            });
        }

        // TODO: Replace with actual calls to _try_parse_tag, _try_parse_anchor, _try_parse_text
        // For now, these are placeholders to allow compilation.
        if let Some((tag, p_next)) = super::tags_anchors::_try_parse_tag(&p_current)? {
            contents.push(Content::Tag(tag));
            p_current = p_next;
            continue;
        }

        if let Some((anchor, p_next)) = super::tags_anchors::_try_parse_anchor(&p_current)? {
            contents.push(Content::Anchor(anchor));
            p_current = p_next;
            continue;
        }

        if let Some((text, p_next)) = super::values::_try_parse_text(&p_current)? {
            let latest_content = contents.pop();
            match latest_content {
                Some(Content::Text(prev_text)) => {
                    let mut new_text = String::new();
                    new_text.push_str(&prev_text.content);
                    new_text.push_str(&text.content);
                    let new_range = Range {
                        begin: prev_text.range.begin,
                        end: text.range.end,
                    };
                    contents.push(Content::Text(Text {
                        content: new_text,
                        range: new_range,
                    }));
                }
                Some(x) => {
                    contents.push(x);
                    contents.push(Content::Text(text));
                }
                None => {
                    contents.push(Content::Text(text));
                }
            }
            p_current = p_next;
            continue;
        }

        // If nothing matches, we have a problem.
        return Err(Ast2Error::ParsingError {
            position: p_current.get_position(),
            message: "Unable to parse content".to_string(),
        });
    }

    Ok((contents, p_current)) // Return the final state
}
