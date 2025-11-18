use crate::ast2::parser::Parser;
use crate::ast2::parser::values;

#[test]
fn test_try_parse_text_simple() {
    let parser = Parser::new("hello world");
    let (text, p_next) = values::_try_parse_text(&parser).unwrap().unwrap();
    assert_eq!(p_next.remain(), "");

    let text_str = "hello world";
    assert_eq!(text.range.begin.offset, 0);
    assert_eq!(text.range.end.offset, text_str.len());
}

#[test]
fn test_try_parse_text_until_tag() {
    let parser = Parser::new("hello @tag rest");
    let (text, p_next) = values::_try_parse_text(&parser).unwrap().unwrap();
    assert_eq!(p_next.remain(), "");

    let text_str = "hello @tag rest";
    assert_eq!(text.range.begin.offset, 0);
    assert_eq!(text.range.end.offset, text_str.len());
}

#[test]
fn test_try_parse_text_until_anchor() {
    let parser = Parser::new("hello <!-- anchor --> rest");
    let (text, p_next) = values::_try_parse_text(&parser).unwrap().unwrap();
    assert_eq!(p_next.remain(), "");

    let text_str = "hello <!-- anchor --> rest";
    assert_eq!(text.range.begin.offset, 0);
    assert_eq!(text.range.end.offset, text_str.len());
}

#[test]
fn test_try_parse_text_with_newline() {
    let parser = Parser::new("line1\nline2 rest");
    let (text, p_next) = values::_try_parse_text(&parser).unwrap().unwrap();
    assert_eq!(p_next.remain(), "line2 rest");
    assert_eq!(p_next.get_position().line, 2);
    assert_eq!(p_next.get_position().column, 1);

    let text_str = "line1\n";
    assert_eq!(text.range.begin.offset, 0);
    assert_eq!(text.range.end.offset, text_str.len());
}

#[test]
fn test_try_parse_text_empty() {
    let parser = Parser::new("");
    let result = values::_try_parse_text(&parser).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_try_parse_text_starts_with_tag() {
    let parser = Parser::new("@tag rest");
    let result = values::_try_parse_text(&parser).unwrap();
    assert!(!result.is_none());
}

#[test]
fn test_try_parse_text_starts_with_anchor() {
    let parser = Parser::new("<!-- anchor --> rest");
    let result = values::_try_parse_text(&parser).unwrap();
    assert!(!result.is_none());
}
