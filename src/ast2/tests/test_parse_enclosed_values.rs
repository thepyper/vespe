use crate::ast2::error::Ast2Error;
use crate::ast2::parser::Parser;
use crate::ast2::parser::values;
use serde_json::json;

#[test]
fn test_try_parse_enclosed_string_double_quote() {
    let doc = r#""hello world" rest"#;
    let parser = Parser::new(doc);
    let (value, p_next) = values::_try_parse_enclosed_string(&parser, "\"")
        .unwrap()
        .unwrap();
    assert_eq!(value, "hello world");
    assert_eq!(p_next.remain(), " rest");

    let doc_escaped = r#""hello \"world\"" rest"#;
    let parser_escaped = Parser::new(doc_escaped);
    let (value_escaped, p_next_escaped) = values::_try_parse_enclosed_string(&parser_escaped, "\"")
        .unwrap()
        .unwrap();
    assert_eq!(value_escaped, "hello \"world\""); // Expect unescaped
    assert_eq!(p_next_escaped.remain(), " rest");

    let doc_unclosed = r#""hello"#;
    let parser_unclosed = Parser::new(doc_unclosed);
    let result = values::_try_parse_enclosed_string(&parser_unclosed, "\"");
    assert!(matches!(result, Err(Ast2Error::UnclosedString { .. })));
}

#[test]
fn test_try_parse_enclosed_string_single_quote() {
    let doc = r#"'hello world' rest"#;
    let parser = Parser::new(doc);
    let (value, p_next) = values::_try_parse_enclosed_string(&parser, "'")
        .unwrap()
        .unwrap();
    assert_eq!(value, "hello world");
    assert_eq!(p_next.remain(), " rest");

    let doc_escaped = r#"'hello \'world\'' rest"#;
    let parser_escaped = Parser::new(doc_escaped);
    let (value_escaped, p_next_escaped) = values::_try_parse_enclosed_string(&parser_escaped, "'")
        .unwrap()
        .unwrap();
    assert_eq!(value_escaped, "hello 'world'"); // Expect unescaped
    assert_eq!(p_next_escaped.remain(), " rest");
}

#[test]
fn test_try_parse_enclosed_value_double_quote() {
    let doc = r#""json value" rest"#;
    let parser = Parser::new(doc);
    let (value, p_next) = values::_try_parse_enclosed_value(&parser, "\"")
        .unwrap()
        .unwrap();
    assert_eq!(value, json!("json value"));
    assert_eq!(p_next.remain(), " rest");
}

#[test]
fn test_try_parse_enclosed_value_single_quote() {
    let doc = "'json value' rest";
    let parser = Parser::new(doc);
    let (value, p_next) = values::_try_parse_enclosed_value(&parser, "'")
        .unwrap()
        .unwrap();
    assert_eq!(value, json!("json value"));
    assert_eq!(p_next.remain(), " rest");
}

#[test]
fn test_try_parse_value_enclosed() {
    let (value_double, p_next_double) = values::_try_parse_value(&parser_double).unwrap().unwrap();
    assert_eq!(value_double, json!("double quoted"));
    assert_eq!(p_next_double.remain(), " rest");

    let (value_single, p_next_single) = values::_try_parse_value(&parser_single).unwrap().unwrap();
    assert_eq!(value_single, json!("single quoted"));
    assert_eq!(p_next_single.remain(), " rest");
}
