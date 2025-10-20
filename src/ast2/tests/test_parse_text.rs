use super::{Parser, Ast2Error};
use super::{_try_parse_text};

#[test]
fn test_try_parse_text_simple() {
    let doc = "hello world";
    let parser = Parser::new(doc);
    let (text, p_next) = _try_parse_text(&parser).unwrap().unwrap();
    assert_eq!(p_next.remain(), "");

    let text_str = "hello world";
    assert_eq!(text.range.begin.offset, 0);
    assert_eq!(text.range.end.offset, text_str.len());
}

#[test]
fn test_try_parse_text_until_tag() {
    let doc = "hello @tag rest";
    let parser = Parser::new(doc);
    let (text, p_next) = _try_parse_text(&parser).unwrap().unwrap();
    assert_eq!(p_next.remain(), "");

    let text_str = "hello @tag rest";
    assert_eq!(text.range.begin.offset, 0);
    assert_eq!(text.range.end.offset, text_str.len());
}

#[test]
fn test_try_parse_text_until_anchor() {
    let doc = "hello <!-- anchor --> rest";
    let parser = Parser::new(doc);
    let (text, p_next) = _try_parse_text(&parser).unwrap().unwrap();
    assert_eq!(p_next.remain(), "");

    let text_str = "hello <!-- anchor --> rest";
    assert_eq!(text.range.begin.offset, 0);
    assert_eq!(text.range.end.offset, text_str.len());
}

#[test]
fn test_try_parse_text_with_newline() {
    let doc = "line1\nline2 rest";
    let parser = Parser::new(doc);
    let (text, p_next) = _try_parse_text(&parser).unwrap().unwrap();
    assert_eq!(p_next.remain(), "line2 rest");
    assert_eq!(p_next.get_position().line, 2);
    assert_eq!(p_next.get_position().column, 1);

    let text_str = "line1\n";
    assert_eq!(text.range.begin.offset, 0);
    assert_eq!(text.range.end.offset, text_str.len());
}

#[test]
fn test_try_parse_text_empty() {
    let doc = "";
    let parser = Parser::new(doc);
    let result = _try_parse_text(&parser).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_try_parse_text_starts_with_tag() {
    let doc = "@tag rest";
    let parser = Parser::new(doc);
    let result = _try_parse_text(&parser).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_try_parse_text_starts_with_anchor() {
    let doc = "<!-- anchor --> rest";
    let parser = Parser::new(doc);
    let result = _try_parse_text(&parser).unwrap();
    assert!(result.is_none());
}

