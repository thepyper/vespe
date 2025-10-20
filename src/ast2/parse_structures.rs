use super::parser::Parser;
use super::error::{Ast2Error, Result};
use super::types::{Tag, Anchor, Text, Content, Document};
use super::parse_elements::{_try_parse_command_kind, _try_parse_anchor_kind, _try_parse_parameters, _try_parse_arguments};
use super::parse_primitives::_try_parse_uuid;

pub fn parse_document(document: &str) -> Result<Document> {
    let parser = Parser::new(document);
    let begin = parser.get_position();
    
    let (content, parser_after_content) = parse_content(parser)?;
    
    let end = parser_after_content.get_position();

    Ok(Document {
        content: content,
        range: super::types::Range { begin, end },
    })
}

pub fn parse_content<'doc>(parser: Parser<'doc>) -> Result<(Vec<Content>, Parser<'doc>)> {
    let mut contents = Vec::new();
    let mut p_current = parser; // Takes ownership

    loop {
        if p_current.is_eod() {
            break;
        }

        // TODO controlla di essere ad inizio linea. se non e' cosi, PROBLEMA perche'
        // le subroutine devono SEMPRE fermarsi ad un inizio linea.
        if !p_current.is_begin_of_line() {
            return Err(Ast2Error::ExpectedBeginOfLine {
                position: p_current.get_position(),
            });
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
            contents.push(Content::Text(text));
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

pub fn _try_parse_tag<'doc>(parser: &Parser<'doc>) -> Result<Option<(Tag, Parser<'doc>)>> {
    let begin = parser.get_position();

    // Must start with '@'
    let p1 = match parser.consume_matching_char_immutable('@') {
        Some(p) => p,
        None => return Ok(None),
    };

    // Then a command
    let (command, p2) = match _try_parse_command_kind(&p1)? {
        Some((c, p)) => (c, p),
        None => return Ok(None), // Not a valid tag if no command
    };

    let p3 = p2.skip_many_whitespaces_immutable();

    // Then optional parameters
    let (parameters, p4) = match _try_parse_parameters(&p3)? {
        Some((p, p_next)) => (p, p_next),
        None => (super::types::Parameters::new(), p2.clone()), // No parameters found, use default and continue from p2
    };

    let p5 = p4.skip_many_whitespaces_immutable();

    // Then optional arguments
    let (arguments, p6) = match _try_parse_arguments(&p5)? {
        Some((a, p_next)) => (a, p_next),
        None => (
            super::types::Arguments {
                arguments: Vec::new(),
                range: super::types::Range::null(),
            },
            p4.clone(),
        ), // No arguments found, use default and continue from p4
    };

    let end = p6.get_position();

    let p7 = p6.skip_many_whitespaces_immutable();

    // Consume EOL if it's there, but don't fail if it's not (e.g. end of file)
    let p8 = p7.consume_matching_char_immutable('\n').unwrap_or(p7);

    let tag = Tag {
        command,
        parameters,
        arguments,
        range: super::types::Range { begin, end },
    };

    Ok(Some((tag, p8)))
}

pub fn _try_parse_anchor<'doc>(parser: &Parser<'doc>) -> Result<Option<(Anchor, Parser<'doc>)>> {
    let begin = parser.get_position();

    let p1 = match parser.consume_matching_string_immutable("<!--") {
        Some(p) => p,
        None => return Ok(None),
    };

    let p2 = p1.skip_many_whitespaces_immutable();

    let (command, p3) = match _try_parse_command_kind(&p2)? {
        Some((c, p)) => (c, p),
        None => return Ok(None), // Not a valid anchor if no command
    };

    let p4 = match p3.consume_matching_char_immutable('-') {
        Some(p) => p,
        None => {
            return Err(Ast2Error::ParsingError {
                position: p3.get_position(),
                message: "Expected '-' before UUID in anchor".to_string(),
            });
        }
    };

    let (uuid, p5) = match _try_parse_uuid(&p4)? {
        Some((u, p)) => (u, p),
        None => {
            return Err(Ast2Error::InvalidUuid {
                position: p4.get_position(),
            })
        }
    };

    let p6 = match p5.consume_matching_char_immutable(':') {
        Some(p) => p,
        None => {
            return Err(Ast2Error::ParsingError {
                position: p5.get_position(),
                message: "Expected ':' after UUID in anchor".to_string(),
            });
        }
    };

    let (kind, p7) = match _try_parse_anchor_kind(&p6)? {
        Some((k, p)) => (k, p),
        None => {
            return Err(Ast2Error::InvalidAnchorKind {
                position: p6.get_position(),
            })
        }
    };

    let p8 = p7.skip_many_whitespaces_immutable();

    let (parameters, p9) = match _try_parse_parameters(&p8)? {
        Some((p, p_next)) => (p, p_next),
        None => (super::types::Parameters::new(), p8.clone()),
    };

    let p10 = p9.skip_many_whitespaces_immutable();

    let (arguments, p11) = match _try_parse_arguments(&p10)? {
        Some((a, p_next)) => (a, p_next),
        None => (
            super::types::Arguments {
                arguments: Vec::new(),
                range: super::types::Range::null(),
            },
            p10.clone(),
        ),
    };

    let p12 = p11.skip_many_whitespaces_or_eol_immutable();

    let p13 = match p12.consume_matching_string_immutable("-->") {
        Some(p) => p,
        None => {
            return Err(Ast2Error::UnclosedString { // Using UnclosedString for a missing -->
                position: p12.get_position(),
            });
        }
    };

    let end = p13.get_position();

    let p14 = p13.skip_many_whitespaces_or_eol_immutable();

    // Consume EOL if it's there
    let p15 = p14.consume_matching_char_immutable('\n').unwrap_or(p14);

    let anchor = Anchor {
        command,
        uuid,
        kind,
        parameters,
        arguments,
        range: super::types::Range { begin, end },
    };

    Ok(Some((anchor, p15)))
}

pub fn _try_parse_text<'doc>(parser: &Parser<'doc>) -> Result<Option<(Text, Parser<'doc>)>> {
    let begin = parser.get_position();

    if parser.is_eod() {
        return Ok(None);
    }
    
    // Stop if we see a tag or anchor start
    if parser.remain().starts_with('@') || parser.remain().starts_with("<!--") {
        return Ok(None);
    }

    let mut p_current = parser.clone();
    let mut content = String::new();
    
    loop {
        match p_current.advance_immutable() {
            None => break, // EOD
            Some((\'\n\', p_next)) => {
                content.push('\n');
                p_current = p_next;
                break; // Consumed newline and stopped
            }
            Some((c, p_next)) => {
                content.push(c);
                p_current = p_next;
            }
        }
    }

    if content.is_empty() {
        return Ok(None);
    }

    let end = p_current.get_position();
    let text = Text { range: super::types::Range { begin, end } };
    Ok(Some((text, p_current)))}
