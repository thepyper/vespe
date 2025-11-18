use serde_json::json;
use std::collections::BTreeMap;
use std::str::Chars;
use std::str::FromStr;
use uuid::Uuid;

use super::{
    Anchor, AnchorKind, Argument, Arguments, Ast2Error, CommandKind, Content, Document,
    JsonPlusEntity, JsonPlusObject, Parameters, Position, Range, Result, Tag, Text,
};
use super::parser::Parser;







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
    if let Some((value, p)) = _try_parse_enclosed_string(&parser, "'")? {
        let end = p.get_position();
        let arg = Argument {
            value,
            range: Range { begin, end },
        };
        return Ok(Some((arg, p)));
    }
    //}

    //if let Some(p1) = parser.consume_matching_char_immutable('"') {
    if let Some((value, p)) = _try_parse_enclosed_string(&parser, "\"")? {
        let end = p.get_position();
        let arg = Argument {
            value,
            range: Range { begin, end },
        };
        return Ok(Some((arg, p)));
    }
    //}

    if let Some((value, p)) = _try_parse_nude_string(parser)? {
        let end = p.get_position();
        let arg = Argument {
            value,
            range: Range { begin, end },
        };
        return Ok(Some((arg, p)));
    }

    Ok(None)
}

pub(crate) fn _try_parse_identifier<'doc>(
    parser: &Parser<'doc>,
) -> Result<Option<(String, Parser<'doc>)>> {
    let (first_char, parser1) =
        match parser.consume_char_if_immutable(|c| c.is_alphabetic() || c == '_') {
            Some((c, p)) => (c, p),
            None => return Ok(None),
        };

    let (rest, parser2) = parser1.consume_many_if_immutable(|c| c.is_alphanumeric() || c == '_');

    let mut identifier = String::new();
    identifier.push(first_char);
    identifier.push_str(&rest);

    Ok(Some((identifier, parser2)))
}

pub(crate) fn _try_parse_value<'doc>(
    parser: &Parser<'doc>,
) -> Result<Option<(serde_json::Value, Parser<'doc>)>> {
    if let Some(x) = _try_parse_enclosed_value(parser, "\"")? {
        Ok(Some(x))
    } else if let Some(x) = _try_parse_enclosed_value(parser, "\'")? {
        Ok(Some(x))
    } else {
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

pub(crate) fn _try_parse_enclosed_string<'doc>(
    parser: &Parser<'doc>,
    closure: &str,
) -> Result<Option<(String, Parser<'doc>)>> {
    let begin_pos = parser.get_position();
    let mut value = String::new();
    let mut current_parser = parser.clone();

    if let Some(p) = current_parser.consume_matching_string_immutable(closure) {
        current_parser = p;
        loop {
            if let Some(p) = current_parser.consume_matching_string_immutable("\\\"") {
                value.push('\"');
                current_parser = p;
            } else if let Some(p) = current_parser.consume_matching_string_immutable("\\\'") {
                value.push('\'');
                current_parser = p;
            } else if let Some(p) = current_parser.consume_matching_string_immutable("\\n") {
                value.push('\n');
                current_parser = p;
            } else if let Some(p) = current_parser.consume_matching_string_immutable("\\r") {
                value.push('\r');
                current_parser = p;
            } else if let Some(p) = current_parser.consume_matching_string_immutable("\\t") {
                value.push('\t');
                current_parser = p;
            } else if let Some(p) = current_parser.consume_matching_string_immutable("\\\\") {
                value.push('\\');
                current_parser = p;
            } else if let Some(p) = current_parser.consume_matching_string_immutable(closure) {
                return Ok(Some((value, p)));
            } else if current_parser.is_eod() {
                return Err(Ast2Error::UnclosedString {
                    position: begin_pos,
                });
            } else {
                match current_parser.advance_immutable() {
                    None => unreachable!("Checked is_eod() already"),
                    Some((x, p)) => {
                        value.push(x);
                        current_parser = p;
                    }
                }
            }
        }
    }
    Ok(None)
}

pub(crate) fn _try_parse_nude_value<'doc>(
    parser: &Parser<'doc>,
) -> Result<Option<(serde_json::Value, Parser<'doc>)>> {
    if let Some((x, p)) = _try_parse_nude_float(parser)? {
        return Ok(Some((json!(x), p)));
    }
    if let Some((x, p)) = _try_parse_nude_integer(parser)? {
        return Ok(Some((json!(x), p)));
    }
    if let Some((x, p)) = _try_parse_nude_bool(parser)? {
        return Ok(Some((json!(x), p)));
    }
    if let Some((x, p)) = _try_parse_nude_string(parser)? {
        return Ok(Some((json!(x), p)));
    }
    Ok(None)
}

pub(crate) fn _try_parse_nude_integer<'doc>(
    parser: &Parser<'doc>,
) -> Result<Option<(i64, Parser<'doc>)>> {
    let (number_str, new_parser) = parser.consume_many_if_immutable(|x| x.is_digit(10));

    if number_str.is_empty() {
        return Ok(None);
    }

    match i64::from_str_radix(&number_str, 10) {
        Ok(num) => Ok(Some((num, new_parser))),

        Err(e) => Err(Ast2Error::ParseIntError(e)),
    }
}

pub(crate) fn _try_parse_nude_float<'doc>(
    parser: &Parser<'doc>,
) -> Result<Option<(f64, Parser<'doc>)>> {
    let (int_part, p1) = parser.consume_many_if_immutable(|x| x.is_digit(10));

    if let Some(p2) = p1.consume_matching_char_immutable('.') {
        // Found a dot.

        let (frac_part, p3) = p2.consume_many_if_immutable(|x| x.is_digit(10));

        if int_part.is_empty() && frac_part.is_empty() {
            return Ok(None); // Just a dot, not a number
        }

        let num_str = format!("{}.{}", int_part, frac_part);

        match f64::from_str(&num_str) {
            Ok(n) => Ok(Some((n, p3))),

            Err(e) => Err(Ast2Error::ParseFloatError(e)),
        }
    } else {
        // No dot, not a float for our purposes.

        Ok(None)
    }
}

pub(crate) fn _try_parse_nude_bool<'doc>(
    parser: &Parser<'doc>,
) -> Result<Option<(bool, Parser<'doc>)>> {
    if let Some(p) = parser.consume_matching_string_immutable("true") {
        return Ok(Some((true, p)));
    } else if let Some(p) = parser.consume_matching_string_immutable("false") {
        return Ok(Some((false, p)));
    } else {
        return Ok(None);
    }
}

pub(crate) fn _try_parse_nude_string<'doc>(
    parser: &Parser<'doc>,
) -> Result<Option<(String, Parser<'doc>)>> {
    let (result, new_parser) = parser.consume_many_if_immutable(|x| {
        x.is_alphanumeric() || x == '/' || x == '.' || x == '_' || x == '-'
    });

    if result.is_empty() {
        Ok(None)
    } else {
        Ok(Some((result, new_parser)))
    }
}

pub(crate) fn _try_parse_uuid<'doc>(parser: &Parser<'doc>) -> Result<Option<(Uuid, Parser<'doc>)>> {
    let start_pos = parser.get_position();

    let (uuid_str, new_parser) =
        parser.consume_many_if_immutable(|c| c.is_ascii_hexdigit() || c == '-');

    match Uuid::parse_str(&uuid_str) {
        Ok(uuid) => Ok(Some((uuid, new_parser))),

        Err(_) => Err(Ast2Error::InvalidUuid {
            position: start_pos,
        }),
    }
}

pub(crate) fn _try_parse_text<'doc>(parser: &Parser<'doc>) -> Result<Option<(Text, Parser<'doc>)>> {
    let begin = parser.get_position();

    if parser.is_eod() {
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
    let text = Text {
        content,
        range: Range { begin, end },
    };
    Ok(Some((text, p_current)))
}

pub(crate) fn _try_parse_jsonplus_object<'doc>(
    parser: &Parser<'doc>,
) -> Result<Option<(JsonPlusObject, Parser<'doc>)>> {
    let mut p1 = match parser.consume_matching_char_immutable('{') {
        Some(p) => p,
        None => return Ok(None),
    };

    let mut properties = BTreeMap::new();

    while !p1.is_eod() {
        p1.skip_many_whitespaces_or_eol();

        if let Some(p2) = p1.consume_matching_string_immutable("}") {
            return Ok(Some((JsonPlusObject { properties }, p2)));
        }

        if let Some((key, mut p3)) = _try_parse_identifier(&p1)? {
            p3.skip_many_whitespaces_or_eol();
            if let Some((_, mut p3)) = p3.consume_char_if_immutable(|x| (x == ':') | (x == '=')) {
                p3.skip_many_whitespaces_or_eol();
                if let Some((object, p4)) = _try_parse_jsonplus_object(&p3)? {
                    properties.insert(key, JsonPlusEntity::Object(object));
                    p1 = p4;
                } else if let Some((array, p4)) = _try_parse_jsonplus_array(&p3)? {
                    properties.insert(key, JsonPlusEntity::Array(array));
                    p1 = p4;
                } else if let Some((single_quoted_content, p4)) =
                    _try_parse_enclosed_string(&p3, "'")?
                {
                    properties.insert(
                        key,
                        JsonPlusEntity::SingleQuotedString(single_quoted_content),
                    );
                    p1 = p4;
                } else if let Some((double_quoted_content, p4)) =
                    _try_parse_enclosed_string(&p3, "\"")?
                {
                    properties.insert(
                        key,
                        JsonPlusEntity::DoubleQuotedString(double_quoted_content),
                    );
                    p1 = p4;
                } else if let Some((x, p4)) = _try_parse_nude_float(&p3)? {
                    properties.insert(key, JsonPlusEntity::Float(x));
                    p1 = p4
                } else if let Some((x, p4)) = _try_parse_nude_integer(&p3)? {
                    properties.insert(key, JsonPlusEntity::Integer(x));
                    p1 = p4
                } else if let Some((x, p4)) = _try_parse_nude_bool(&p3)? {
                    properties.insert(key, JsonPlusEntity::Boolean(x));
                    p1 = p4
                } else if let Some((x, p4)) = _try_parse_nude_string(&p3)? {
                    properties.insert(key, JsonPlusEntity::NudeString(x));
                    p1 = p4
                } else {
                    // No value
                    return Err(Ast2Error::MissingParameterValue {
                        position: p1.get_position(),
                    });
                }
            } else {
                properties.insert(key, JsonPlusEntity::Flag);
                p1 = p3;
            }
        } else {
            // Missing identifier
            return Err(Ast2Error::MissingParameterKey {
                position: p1.get_position(),
            });
        }

        p1.skip_many_whitespaces_or_eol();
        if let Some(_p2) = p1.consume_matching_string_immutable("}") {
            // Will exit on next iteration
        } else if let Some(p2) = p1.consume_matching_string_immutable(",") {
            // Will consume another object on next iteration
            p1 = p2;
        } else {
            // Missing separator
            return Err(Ast2Error::MissingCommaInParameters {
                position: p1.get_position(),
            });
        }
    }

    Err(Ast2Error::UnterminatedObject {
        position: p1.get_position(),
    })
}

pub(crate) fn _try_parse_jsonplus_array<'doc>(
    parser: &Parser<'doc>,
) -> Result<Option<(Vec<JsonPlusEntity>, Parser<'doc>)>> {
    let mut p1 = match parser.consume_matching_char_immutable('[') {
        Some(p) => p,
        None => return Ok(None),
    };

    let mut array = Vec::new();

    while !p1.is_eod() {
        p1.skip_many_whitespaces_or_eol();

        if let Some(p2) = p1.consume_matching_string_immutable("]") {
            return Ok(Some((array, p2)));
        }

        if let Some((object, p4)) = _try_parse_jsonplus_object(&p1)? {
            array.push(JsonPlusEntity::Object(object));
            p1 = p4;
        } else if let Some((array2, p4)) = _try_parse_jsonplus_array(&p1)? {
            array.push(JsonPlusEntity::Array(array2));
            p1 = p4;
        } else if let Some((single_quoted_content, p4)) = _try_parse_enclosed_string(&p1, "'")? {
            array.push(JsonPlusEntity::SingleQuotedString(single_quoted_content));
            p1 = p4;
        } else if let Some((double_quoted_content, p4)) = _try_parse_enclosed_string(&p1, "\"")? {
            array.push(JsonPlusEntity::DoubleQuotedString(double_quoted_content));
            p1 = p4;
        } else if let Some((x, p4)) = _try_parse_nude_float(&p1)? {
            array.push(JsonPlusEntity::Float(x));
            p1 = p4
        } else if let Some((x, p4)) = _try_parse_nude_integer(&p1)? {
            array.push(JsonPlusEntity::Integer(x));
            p1 = p4
        } else if let Some((x, p4)) = _try_parse_nude_bool(&p1)? {
            array.push(JsonPlusEntity::Boolean(x));
            p1 = p4
        } else if let Some((x, p4)) = _try_parse_nude_string(&p1)? {
            array.push(JsonPlusEntity::NudeString(x));
            p1 = p4
        } else {
            return Err(Ast2Error::MalformedValue {
                position: p1.get_position(),
            });
        }

        p1.skip_many_whitespaces_or_eol();
        if let Some(_p2) = p1.consume_matching_string_immutable("]") {
            // Will exit on next iteration
        } else if let Some(p2) = p1.consume_matching_string_immutable(",") {
            // Will consume another object on next iteration
            p1 = p2;
        } else {
            // Missing separator
            return Err(Ast2Error::MissingCommaInParameters {
                position: p1.get_position(),
            });
        }
    }

    Err(Ast2Error::UnterminatedArray {
        position: p1.get_position(),
    })
}

#[cfg(test)]
#[path = "./tests/test_parse_anchor.rs"]
mod test_parse_anchor;

#[cfg(test)]
#[path = "./tests/test_parse_argument.rs"]
mod test_parse_argument;

#[cfg(test)]
#[path = "./tests/test_parse_arguments.rs"]
mod test_parse_arguments;

#[cfg(test)]
#[path = "./tests/test_parse_document.rs"]
mod test_parse_document;

#[cfg(test)]
#[path = "./tests/test_parse_enclosed_values.rs"]
mod test_parse_enclosed_values;

#[cfg(test)]
#[path = "./tests/test_position_range.rs"]
mod test_position_range;

#[cfg(test)]
#[path = "./tests/utils.rs"]
mod utils;

#[cfg(test)]
#[path = "./tests/test_parser_advance.rs"]
mod test_parser_advance;

#[cfg(test)]
#[path = "./tests/test_parser_consume.rs"]
mod test_parser_consume;

#[cfg(test)]
#[path = "./tests/test_parse_uuid.rs"]
mod test_parse_uuid;

#[cfg(test)]
#[path = "./tests/test_parse_text.rs"]
mod test_parse_text;

#[cfg(test)]
#[path = "./tests/test_parse_identifier.rs"]
mod test_parse_identifier;

#[cfg(test)]
#[path = "./tests/test_parse_kinds.rs"]
mod test_parse_kinds;

#[cfg(test)]
#[path = "./tests/test_parse_nude_values.rs"]
mod test_parse_nude_values;

#[cfg(test)]
#[path = "./tests/test_parse_parameters.rs"]
mod test_parse_parameters;

#[cfg(test)]
#[path = "./tests/test_parse_tag.rs"]
mod test_parse_tag;

#[cfg(test)]
#[path = "./tests/test_parse_jsonplus.rs"]
mod test_parse_jsonplus;
