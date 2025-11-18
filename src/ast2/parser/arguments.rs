use super::Parser;
use crate::ast2::{Argument, Arguments, Range, Result};

pub(crate) fn _try_parse_arguments<'doc>(
    parser: &Parser<'doc>,
) -> Result<Option<(Arguments, Parser<'doc>)>> {
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

pub(crate) fn _try_parse_argument<'doc>(
    parser: &Parser<'doc>,
) -> Result<Option<(Argument, Parser<'doc>)>> {
    let begin = parser.get_position();

    //if let Some(p1) = parser.consume_matching_char_immutable('\'') {
    if let Some((value, p)) = super::values::_try_parse_enclosed_string(&parser, "'")? {
        let end = p.get_position();
        let arg = Argument {
            value,
            range: Range { begin, end },
        };
        return Ok(Some((arg, p)));
    }
    //}

    //if let Some(p1) = parser.consume_matching_char_immutable('"') {
    if let Some((value, p)) = super::values::_try_parse_enclosed_string(&parser, "\"")? {
        let end = p.get_position();
        let arg = Argument {
            value,
            range: Range { begin, end },
        };
        return Ok(Some((arg, p)));
    }
    //}

    if let Some((value, p)) = super::values::_try_parse_nude_string(parser)? {
        let end = p.get_position();
        let arg = Argument {
            value,
            range: Range { begin, end },
        };
        return Ok(Some((arg, p)));
    }

    Ok(None)
}
