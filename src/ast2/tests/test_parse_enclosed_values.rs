use crate::ast2::{Parser, Ast2Error};
use serde_json::json;

#[test]
fn test_try_parse_enclosed_string_double_quote() {
    let doc = r#""hello world" rest"#;
    let parser = Parser::new(doc);
    let (value, p_next) = super::super::_try_parse_enclosed_string(&parser, "\"").unwrap().unwrap();
    assert_eq!(value, "hello world");
    assert_eq!(p_next.remain(), " rest");

    let doc_escaped = r#""hello \"world\"" rest"#;
    let parser_escaped = Parser::new(doc_escaped);
    let (value_escaped, p_next_escaped) = super::super::_try_parse_enclosed_string(&parser_escaped, "\"").unwrap().unwrap();
    assert_eq!(value_escaped, "hello \"world\"");
    assert_eq!(p_next_escaped.remain(), " rest");

    let doc_unclosed = r#""hello"#;
    let parser_unclosed = Parser::new(doc_unclosed);
    let result = super::super::_try_parse_enclosed_string(&parser_unclosed, "\"");
    assert!(matches!(result, Err(Ast2Error::UnclosedString { .. })));
}

#[test]
fn test_try_parse_enclosed_string_single_quote() {
    let doc = "'hello world' rest";
    let parser = Parser::new(doc);
    let (value, p_next) = super::super::_try_parse_enclosed_string(&parser, "'").unwrap().unwrap();
    assert_eq!(value, "hello world");
    assert_eq!(p_next.remain(), " rest");

    let doc_escaped = "'hello \'world\'' rest";
    let parser_escaped = Parser::new(doc_escaped);
    let (value_escaped, p_next_escaped) = super::super::_try_parse_enclosed_string(&parser_escaped, "'").unwrap().unwrap();
    assert_eq!(value_escaped, "hello \'world\'");
    assert_eq!(p_next_escaped.remain(), " rest");
}

#[test]
fn test_try_parse_enclosed_value_double_quote() {
    let doc = r#""json value" rest"#;
    let parser = Parser::new(doc);
    let (value, p_next) = super::super::_try_parse_enclosed_value(&parser, "\"").unwrap().unwrap();
    assert_eq!(value, json!("json value"));
    assert_eq!(p_next.remain(), " rest");
}

#[test]
fn test_try_parse_enclosed_value_single_quote() {
    let doc = "'json value' rest";
    let parser = Parser::new(doc);
    let (value, p_next) = super::super::_try_parse_enclosed_value(&parser, "'").unwrap().unwrap();
    assert_eq!(value, json!("json value"));
    assert_eq!(p_next.remain(), " rest");
}

#[test]
fn test_try_parse_value_enclosed() {
    let doc_double = r#""double quoted" rest"#;
    let parser_double = Parser::new(doc_double);
    let (value_double, p_next_double) = super::super::_try_parse_value(&parser_double).unwrap().unwrap();
    assert_eq!(value_double, json!("double quoted"));
    assert_eq!(p_next_double.remain(), " rest");

    let doc_single = "'single quoted' rest";
    let parser_single = Parser::new(doc_single);
    let (value_single, p_next_single) = super::super::_try_parse_value(&parser_single).unwrap().unwrap();
    assert_eq!(value_single, json!("single quoted"));
    assert_eq!(p_next_single.remain(), " rest");
}
