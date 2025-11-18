
use super::parser::Parser;
use super::identifier::_try_parse_identifier;
use super::json_plus::_try_parse_jsonplus_object;
use super::values::_try_parse_value;
use super::super::{Result, Parameters, Range, Ast2Error};

use serde_json::json;

pub(crate) fn _try_parse_parameters<'doc>(
    parser: &Parser<'doc>,
) -> Result<Option<(Parameters, Parser<'doc>)>> {
    let begin = parser.get_position();

    if let Some((json_object, parser)) = _try_parse_jsonplus_object(parser)? {
        let end = parser.get_position();
        return Ok(Some((
            Parameters::from_json_object_range(json_object, Range { begin, end }),
            parser,
        )));
    } else {
        return Ok(None);
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
        None => return Ok(Some(((key, json!(true)), p2))), // No colon, so not a parameter. Treat as if key = true, continue from p2.
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