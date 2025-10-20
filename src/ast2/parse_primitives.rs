use uuid::Uuid;
use std::str::FromStr;

use super::parser::Parser;
use super::error::{Ast2Error, Result};

pub fn _try_parse_identifier<'doc>(parser: &Parser<'doc>) -> Result<Option<(String, Parser<'doc>)>> {
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

pub fn _try_parse_uuid<'doc>(parser: &Parser<'doc>) -> Result<Option<(Uuid, Parser<'doc>)>> {
    let start_pos = parser.get_position();

    let (uuid_str, new_parser) = parser.consume_many_if_immutable(|c| c.is_ascii_hexdigit() || c == '-');

    match Uuid::parse_str(&uuid_str) {
        Ok(uuid) => Ok(Some((uuid, new_parser))),
        Err(_) => Err(Ast2Error::InvalidUuid {
            position: start_pos,
        }),
    }
}

pub fn _try_parse_nude_integer<'doc>(parser: &Parser<'doc>) -> Result<Option<(i64, Parser<'doc>)>> {
    let (number_str, new_parser) = parser.consume_many_if_immutable(|x| x.is_digit(10));

    if number_str.is_empty() {
        return Ok(None);
    }

    match i64::from_str_radix(&number_str, 10) {
        Ok(num) => Ok(Some((num, new_parser))),
        Err(e) => Err(Ast2Error::ParseIntError(e)),
    }
}

pub fn _try_parse_nude_float<'doc>(parser: &Parser<'doc>) -> Result<Option<(f64, Parser<'doc>)>> {
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

pub fn _try_parse_nude_bool<'doc>(parser: &Parser<'doc>) -> Result<Option<(bool, Parser<'doc>)>> {
    if let Some(p) = parser.consume_matching_string_immutable("true") {
        return Ok(Some((true, p)));
    } else if let Some(p) = parser.consume_matching_string_immutable("false") {
        return Ok(Some((false, p)));
    } else {
        return Ok(None);
    }
}

pub fn _try_parse_nude_string<'doc>(parser: &Parser<'doc>) -> Result<Option<(String, Parser<'doc>)>> {
    let (result, new_parser) = parser.consume_many_if_immutable(|x| x.is_alphanumeric() || x == '/' || x == '.' || x == '_');

    if result.is_empty() {
        Ok(None)
    } else {
        Ok(Some((result, new_parser)))
    }
}

pub fn _try_parse_enclosed_string<'doc>(
    parser: &Parser<'doc>,
    closure: &str,
) -> Result<Option<(String, Parser<'doc>)>> {
    let begin_pos = parser.get_position();
    let mut value = String::new();
    let mut current_parser = parser.clone();

    loop {
        if let Some(p) = current_parser.consume_matching_string_immutable(closure) {
            return Ok(Some((value, p)));
        } else if let Some(p_after_backslash) = current_parser.consume_matching_char_immutable('\\') {
            // Handle escaped characters
            if let Some((escaped_char, p_after_escaped_char)) = p_after_backslash.advance_immutable() {
                match escaped_char {
                    'n' => value.push('\n'),
                    'r' => value.push('\r'),
                    't' => value.push('\t'),
                    '\\' => value.push('\\'),
                    '"' => value.push('"'),
                    '\'' => value.push('\''),
                    _ => {
                        // If it's an unknown escape sequence, just push the backslash and the char
                        value.push('\\');
                        value.push(escaped_char);
                    }
                }
                current_parser = p_after_escaped_char;
            } else {
                // Backslash at the end of the document
                return Err(Ast2Error::UnclosedString {
                    position: begin_pos,
                });
            }
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
