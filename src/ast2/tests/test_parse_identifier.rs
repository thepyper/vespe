use super::identifier::_try_parse_identifier;
use super::parser::Parser;

#[test]
fn test_try_parse_identifier_valid() {
    let doc = "_my_identifier123 rest";
    let parser = Parser::new(doc);
    let (identifier, p_next) = _try_parse_identifier(&parser).unwrap().unwrap();

    assert_eq!(identifier, "_my_identifier123");
    assert_eq!(p_next.remain(), " rest");
}

#[test]
fn test_try_parse_identifier_starts_with_digit() {
    let doc = "123identifier";
    let parser = Parser::new(doc);
    let result = _try_parse_identifier(&parser).unwrap();

    assert!(result.is_none());
    assert_eq!(parser.remain(), "123identifier");
}

#[test]
fn test_try_parse_identifier_empty() {
    let doc = "";
    let parser = Parser::new(doc);
    let result = _try_parse_identifier(&parser).unwrap();

    assert!(result.is_none());
}

#[test]
fn test_try_parse_identifier_with_invalid_char() {
    let doc = "my-identifier";
    let parser = Parser::new(doc);
    let (identifier, p_next) = _try_parse_identifier(&parser).unwrap().unwrap();

    assert_eq!(identifier, "my");
    assert_eq!(p_next.remain(), "-identifier");
}
