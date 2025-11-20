use super::super::{Arguments, Parameters, Range, Result, Tag};
use super::arguments::_try_parse_arguments;
use super::command_kind::_try_parse_command_kind;
use super::parameters::_try_parse_parameters;
use super::parser::Parser;

pub(crate) fn _try_parse_tag<'doc>(parser: &Parser<'doc>) -> Result<Option<(Tag, Parser<'doc>)>> {
    let begin = parser.get_position();

    // Tags can be indented
    let p0 = parser.skip_many_whitespaces_immutable();

    // Must start with '@'
    let p1 = match p0.consume_matching_char_immutable('@') {
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

    let p7 = p6.skip_many_whitespaces_immutable();

    // Consume EOL if it's there, but don't fail if it's not (e.g. end of file)
    let p8 = p7.consume_matching_char_immutable('\n').unwrap_or(p7);

    let end = p8.get_position();

    let tag = Tag {
        command,
        parameters,
        arguments,
        range: Range { begin, end },
    };

    Ok(Some((tag, p8)))
}
