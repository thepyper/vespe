use super::super::{Anchor, AnchorKind, Arguments, Ast2Error, Parameters, Range, Result};
use super::arguments::_try_parse_arguments;
use super::command_kind::_try_parse_command_kind;
use super::identifier::_try_parse_identifier;
use super::parameters::_try_parse_parameters;
use super::parser::Parser;
use super::values::_try_parse_uuid;

pub(crate) fn _try_parse_anchor<'doc>(
    parser: &Parser<'doc>,
) -> Result<Option<(Anchor, Parser<'doc>)>> {
    let begin = parser.get_position();

    // Anchors can be indented
    let p0 = parser.skip_many_whitespaces_immutable();

    let p1 = match p0.consume_matching_string_immutable("<!--") {
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

    let mut status: Option<String> = None;
    let p_after_status_parse = if let Some(p_plus_start) = p8.consume_matching_char_immutable('+') {
        if let Some((parsed_status, p_status_content)) = _try_parse_identifier(&p_plus_start)? {
            if let Some(p_plus_end) = p_status_content.consume_matching_char_immutable('+') {
                status = Some(parsed_status);
                p_plus_end
            } else {
                return Err(Ast2Error::ParsingError {
                    position: p_status_content.get_position(),
                    message: "Expected '+' after status in anchor".to_string(),
                });
            }
        } else {
            return Err(Ast2Error::ParsingError {
                position: p_plus_start.get_position(),
                message: "Expected status identifier after '+' in anchor".to_string(),
            });
        }
    } else {
        p8 // No status found, continue from p8
    };

    let p8_5 = p_after_status_parse.skip_many_whitespaces_immutable();

    let (parameters, p9) = match _try_parse_parameters(&p8_5)? {
        Some((p, p_next)) => (p, p_next),
        None => (Parameters::new(), p_after_status_parse.clone()),
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
            return Err(Ast2Error::UnclosedString {
                // Using UnclosedString for a missing -->
                position: p12.get_position(),
            });
        }
    };

    let p14 = p13.skip_many_whitespaces_immutable();

    // Consume EOL if it's there
    let p15 = p14.consume_matching_char_immutable('\n').unwrap_or(p14);

    let end = p15.get_position();

    let anchor = Anchor {
        command,
        uuid,
        kind,
        status,
        parameters,
        arguments,
        range: Range { begin, end },
    };

    Ok(Some((anchor, p15)))
}

pub(crate) fn _try_parse_anchor_kind<'doc>(
    parser: &Parser<'doc>,
) -> Result<Option<(AnchorKind, Parser<'doc>)>> {
    let tags_list = vec![("begin", AnchorKind::Begin), ("end", AnchorKind::End)];

    for (name, kind) in tags_list {
        if let Some(new_parser) = parser.consume_matching_string_immutable(name) {
            return Ok(Some((kind, new_parser)));
        }
    }

    Ok(None)
}
