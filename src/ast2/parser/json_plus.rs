use super::super::{Ast2Error, JsonPlusEntity, JsonPlusObject, Result};
use super::identifier::_try_parse_identifier;
use super::parser::Parser;
use super::values::_try_parse_enclosed_string;
use super::values::_try_parse_nude_bool;
use super::values::_try_parse_nude_float;
use super::values::_try_parse_nude_integer;
use super::values::_try_parse_nude_string;

use std::collections::BTreeMap;

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
