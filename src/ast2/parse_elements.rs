use serde_json::json;
use crate::ast2::types::{Anchor, AnchorKind, Arguments, Ast2Error, CommandKind, Content, Parameters, Result, Tag, Text, Position, Range};
use crate::ast2::parser::Parser;
use crate::ast2::parse_utils::{_try_parse_identifier, _try_parse_value, _try_parse_uuid, _try_parse_enclosed_string, _try_parse_nude_string};

pub(crate) fn parse_content<'doc>(parser: Parser<'doc>) -> Result<(Vec<Content>, Parser<'doc>)> {
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

pub(crate) fn _try_parse_tag<'doc>(parser: &Parser<'doc>) -> Result<Option<(Tag, Parser<'doc>)>> {
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
        None => (Parameters::new(), p2.clone()), // No parameters found, use default and continue from p2
    };

    let p5 = p4.skip_many_whitespaces_immutable();

    // Then optional arguments
    let (arguments, p6) = match _try_parse_arguments(&p5)? {
        Some((a, p_next)) => (a, p_next),
        None => (
            Arguments {
                arguments: Vec::new(),
                range: Range::null(),
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
        range: Range { begin, end },
    };

    Ok(Some((tag, p8)))
}

pub(crate) fn _try_parse_anchor<'doc>(parser: &Parser<'doc>) -> Result<Option<(Anchor, Parser<'doc>)>> {
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
        None => (Parameters::new(), p8.clone()),
    };

    let p10 = p9.skip_many_whitespaces_immutable();

    let (arguments, p11) = match _try_parse_arguments(&p10)? {
        Some((a, p_next)) => (a, p_next),
        None => (
            Arguments {
                arguments: Vec::new(),
                range: Range::null(),
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
        range: Range { begin, end },
    };

    Ok(Some((anchor, p15)))
}

pub(crate) fn _try_parse_command_kind<'doc>(parser: &Parser<'doc>) -> Result<Option<(CommandKind, Parser<'doc>)>> {
    let tags_list = vec![
        ("tag", CommandKind::Tag),
        ("include", CommandKind::Include),
        ("inline", CommandKind::Inline),
        ("answer", CommandKind::Answer),
        ("summarize", CommandKind::Summarize),
        ("derive", CommandKind::Derive),
        ("repeat", CommandKind::Repeat),
    ];

    for (name, kind) in tags_list {
        if let Some(new_parser) = parser.consume_matching_string_immutable(name) {
            return Ok(Some((kind, new_parser)));
        }
    }

    Ok(None)
}

pub(crate) fn _try_parse_anchor_kind<'doc>(parser: &Parser<'doc>) -> Result<Option<(AnchorKind, Parser<'doc>)>> {
    let tags_list = vec![("begin", AnchorKind::Begin), ("end", AnchorKind::End)];

    for (name, kind) in tags_list {
        if let Some(new_parser) = parser.consume_matching_string_immutable(name) {
            return Ok(Some((kind, new_parser)));
        }
    }

    Ok(None)
}

pub(crate) fn _try_parse_parameters<'doc>(parser: &Parser<'doc>) -> Result<Option<(Parameters, Parser<'doc>)>> {
    let begin = parser.get_position();

    // Must start with '['
    let mut p_current = match parser.consume_matching_char_immutable('[') {
        Some(p) => p,
        None => return Ok(None),
    };
    p_current = p_current.skip_many_whitespaces_or_eol_immutable();

    // Check for empty parameters: []
    if let Some(p_final) = p_current.consume_matching_char_immutable(']') {
        let end = p_final.get_position();
        return Ok(Some((
            Parameters {
                parameters: serde_json::Map::new(),
                range: Range { begin, end },
            },
            p_final,
        )));
    }

    let mut parameters_map = serde_json::Map::new();

    // Loop to parse key-value pairs
    loop {
        // Parse a parameter
        let ((key, value), p_after_param) = match _try_parse_parameter(&p_current)? {
            Some((param, p_next)) => (param, p_next),
            None => {
                // This means we couldn't parse a parameter where one was expected.
                return Err(Ast2Error::ParameterNotParsed {
                    position: p_current.get_position(),
                });
            }
        };
        parameters_map.insert(key, value);
        p_current = p_after_param.skip_many_whitespaces_or_eol_immutable();

        // After a parameter, we expect either a ']' (end) or a ',' (continue)
        if let Some(p_final) = p_current.consume_matching_char_immutable(']') {
            // End of parameters
            let end = p_final.get_position();
            return Ok(Some((
                Parameters {
                    parameters: parameters_map,
                    range: Range { begin, end },
                },
                p_final,
            )));
        } else if let Some(p_after_comma) = p_current.consume_matching_char_immutable(',') {
            // Comma found, continue loop
            p_current = p_after_comma.skip_many_whitespaces_or_eol_immutable();
        } else {
            // Neither ']' nor ',' found after a parameter. Syntax error.
            return Err(Ast2Error::MissingCommaInParameters { // Or missing closing brace
                position: p_current.get_position(),
            });
        }
    }
}

pub(crate) fn _try_parse_parameter<'doc>(
    parser: &Parser<'doc>,
) -> Result<Option<((String, serde_json::Value), Parser<'doc>)>> {
    let p_initial = parser.skip_many_whitespaces_or_eol_immutable();

    let (key, p1) = match _try_parse_identifier(&p_initial)? {
        Some((k, p)) => (k, p),
        None => return Ok(None), // Not an error, just didn't find an identifier
    };

    let p2 = p1.skip_many_whitespaces_or_eol_immutable();

    let p3 = match p2.consume_matching_char_immutable('=') {
        Some(p) => p,
        None => return Ok(None), // No colon, so not a parameter. Let the caller decide what to do.
    };

    let p4 = p3.skip_many_whitespaces_or_eol_immutable();

    let (value, p5) = match _try_parse_value(&p4) {
        Ok(Some((v, p))) => (v, p),
        Ok(None) => {
            // Here, a key and colon were found, so a value is expected.
            // This IS a syntax error.
            return Err(Ast2Error::MissingParameterValue {
                position: p4.get_position(),
            });
        }
        Err(e) => return Err(e),
    };

    Ok(Some(((key, value), p5)))
}

pub(crate) fn _try_parse_arguments<'doc>(parser: &Parser<'doc>) -> Result<Option<(Arguments, Parser<'doc>)>> {
    let begin = parser.get_position();
    let mut p_current = parser.clone();
    let mut arguments = Vec::new();

    loop {
        let p_current_after_whitespaces = p_current.skip_many_whitespaces_immutable();

        // Check for anchor end, a special case for arguments
        if p_current_after_whitespaces.remain().starts_with("-->") {
            break;
        }

        match _try_parse_argument(&p_current_after_whitespaces)? {
            Some((arg, p_next)) => {
                arguments.push(arg);
                p_current = p_next;
            }
            None => break, // No more arguments to parse
        }
    }

    if arguments.is_empty() {
        return Ok(None);
    }

    let end = p_current.get_position();

    Ok(Some((
        Arguments {
            arguments,
            range: Range { begin, end },
        },
        p_current,
    )))
}

pub(crate) fn _try_parse_argument<'doc>(parser: &Parser<'doc>) -> Result<Option<(crate::ast2::types::Argument, Parser<'doc>)>> {
    let begin = parser.get_position();

    if let Some(p1) = parser.consume_matching_char_immutable('\'') {
        if let Some((value, p)) = _try_parse_enclosed_string(&p1, "'")? {
            let end = p.get_position();
            let arg = crate::ast2::types::Argument { value, range: Range { begin, end } };
            return Ok(Some((arg, p)));
        }
    }
    
    if let Some(p1) = parser.consume_matching_char_immutable('"') {
        if let Some((value, p)) = _try_parse_enclosed_string(&p1, "\"")? {
            let end = p.get_position();
            let arg = crate::ast2::types::Argument { value, range: Range { begin, end } };
            return Ok(Some((arg, p)));
        }
    }

    if let Some((value, p)) = _try_parse_nude_string(parser)? {
        let end = p.get_position();
        let arg = crate::ast2::types::Argument { value, range: Range { begin, end } };
        return Ok(Some((arg, p)));
    }

    Ok(None)
}

pub(crate) fn _try_parse_text<'doc>(parser: &Parser<'doc>) -> Result<Option<(Text, Parser<'doc>)>> {
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
            Some(('\n', p_next)) => {
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
    let text = Text { range: Range { begin, end } };
    Ok(Some((text, p_current)))
}