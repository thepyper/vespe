
use crate::ast2::parser::Parser;
use crate::ast2::parser::values;

#[test]
fn test_try_parse_identifier_valid() {
    let parser = Parser::new("_my_identifier123 rest");
    let (identifier, p_next) = values::_try_parse_identifier(&parser).unwrap().unwrap();

    assert_eq!(identifier, "_my_identifier123");
    assert_eq!(p_next.remain(), " rest");
}

#[test]
fn test_try_parse_identifier_starts_with_digit() {
    let parser = Parser::new("123identifier");
    let result = values::_try_parse_identifier(&parser).unwrap();

    assert!(result.is_none());
    assert_eq!(parser.remain(), "123identifier");
}

#[test]
fn test_try_parse_identifier_empty() {
    let parser = Parser::new("");
    let result = values::_try_parse_identifier(&parser).unwrap();

    assert!(result.is_none());
}

#[test]
fn test_try_parse_identifier_with_invalid_char() {
    let parser = Parser::new("my-identifier");
    let (identifier, p_next) = values::_try_parse_identifier(&parser).unwrap().unwrap();

    assert_eq!(identifier, "my");
    assert_eq!(p_next.remain(), "-identifier");
}
