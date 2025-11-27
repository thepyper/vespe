use super::super::{Ast2Error, Comment, Content, Range, Result, Text};
use super::anchor::_try_parse_anchor;
use super::comment::_try_parse_comment;
use super::parser::Parser;
use super::tag::_try_parse_tag;
use super::text::_try_parse_text;

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

        if let Some((comment, p_next)) = _try_parse_comment(&p_current)? {
            let latest_content = contents.pop();
            match latest_content {
                Some(Content::Comment(prev_comment)) => {
                    let mut new_comment = String::new();
                    new_comment.push_str(&prev_comment.content);
                    new_comment.push_str(&comment.content);
                    let new_range = Range {
                        begin: prev_comment.range.begin,
                        end: comment.range.end,
                    };
                    contents.push(Content::Comment(Comment {
                        content: new_comment,
                        range: new_range,
                    }));
                }
                Some(x) => {
                    contents.push(x);
                    contents.push(Content::Comment(comment));
                }
                None => {
                    contents.push(Content::Comment(comment));
                }
            }
            p_current = p_next;
            continue;
        }

        if let Some((tag, p_next)) = _try_parse_tag(&p_current)? {
            contents.push(Content::Tag(tag));
            p_current = p_next;
            continue;
        }

        if let Some((anchor, p_next)) = _try_parse_anchor(&p_current)? {
            contents.push(Content::Anchor(anchor));
            p_current = p_next;
            continue;
        }

        if let Some((text, p_next)) = _try_parse_text(&p_current)? {
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
