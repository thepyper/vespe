use super::super::{Comment, Range, Result};
use super::parser::Parser;

pub(crate) fn _try_parse_comment<'doc>(
    parser: &Parser<'doc>,
) -> Result<Option<(Comment, Parser<'doc>)>> {
    let begin = parser.get_position();

    // Must start with '%%'
    let parser = match parser.consume_matching_string_immutable("%%") {
        Some(p) => p,
        None => return Ok(None),
    };

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

    let end = p_current.get_position();
    let comment = Comment {
        content,
        range: Range { begin, end },
    };
    Ok(Some((comment, p_current)))
}
