use serde_json::json;

use super::parser::Parser;
use super::error::{Ast2Error, Result};
use super::types::{CommandKind, AnchorKind, Parameters, Argument, Arguments};
use super::parse_primitives::{_try_parse_identifier, _try_parse_enclosed_string};

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
                range: super::types::Range { begin, end }, // Use super::types::Range
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
                    range: super::types::Range { begin, end }, // Use super::types::Range
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
            range: super::types::Range { begin, end }, // Use super::types::Range
        },
        p_current,
    )))
}

pub(crate) fn _try_parse_argument<'doc>(parser: &Parser<'doc>) -> Result<Option<(Argument, Parser<'doc>)>> {
    let begin = parser.get_position();

    if let Some(p1) = parser.consume_matching_char_immutable('\'') {
        if let Some((value, p)) = _try_parse_enclosed_string(&p1, "'")? {
            let end = p.get_position();
            let arg = Argument { value, range: super::types::Range { begin, end } }; // Use super::types::Range
            return Ok(Some((arg, p)));
        }
    }
    
    if let Some(p1) = parser.consume_matching_char_immutable('"') {
        if let Some((value, p)) = _try_parse_enclosed_string(&p1, "\"")? {
            let end = p.get_position();
            let arg = Argument { value, range: super::types::Range { begin, end } }; // Use super::types::Range
            return Ok(Some((arg, p)));
        }
    }

    if let Some((value, p)) = super::parse_primitives::_try_parse_nude_string(parser)? { // Use super::parse_primitives
        let end = p.get_position();
        let arg = Argument { value, range: super::types::Range { begin, end } }; // Use super::types::Range
        return Ok(Some((arg, p)));
    }

    Ok(None)
}

pub(crate) fn _try_parse_value<'doc>(parser: &Parser<'doc>) -> Result<Option<(serde_json::Value, Parser<'doc>)>> {
    if let Some(p1) = parser.consume_matching_char_immutable('"') {
        // try to parse a double-quoted string
        _try_parse_enclosed_value(&p1, "\"")
    } else if let Some(p1) = parser.consume_matching_char_immutable('\'') {
        // try to parse a single-quoted string
        _try_parse_enclosed_value(&p1, "'")
    } else {
        // try to parse a "nude" value (unquoted)
        _try_parse_nude_value(parser)
    }
}

pub(crate) fn _try_parse_enclosed_value<'doc>(
    parser: &Parser<'doc>,
    closure: &str,
) -> Result<Option<(serde_json::Value, Parser<'doc>)>> {
    match _try_parse_enclosed_string(parser, closure)? {
        Some((s, p)) => Ok(Some((serde_json::Value::String(s), p))),
        None => Ok(None),
    }
}

pub(crate) fn _try_parse_nude_value<'doc>(parser: &Parser<'doc>) -> Result<Option<(serde_json::Value, Parser<'doc>)>> {
    if let Some((x, p)) = super::parse_primitives::_try_parse_nude_float(parser)? { // Use super::parse_primitives
        return Ok(Some((json!(x), p)));
    } 
    if let Some((x, p)) = super::parse_primitives::_try_parse_nude_integer(parser)? { // Use super::parse_primitives
        return Ok(Some((json!(x), p)));
    } 
    if let Some((x, p)) = super::parse_primitives::_try_parse_nude_bool(parser)? { // Use super::parse_primitives
        return Ok(Some((json!(x), p)));
    } 
    if let Some((x, p)) = super::parse_primitives::_try_parse_nude_string(parser)? { // Use super::parse_primitives
        return Ok(Some((json!(x), p)));
    } 
    Ok(None)
}
