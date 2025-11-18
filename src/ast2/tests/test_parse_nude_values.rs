use super::parser::Parser;
use super::values::{_try_parse_nude_integer, _try_parse_nude_float, _try_parse_nude_bool, _try_parse_nude_string, _try_parse_value};
use serde_json::json;

#[test]
fn test_try_parse_nude_integer() {
    let doc = "123 rest";
    let parser = Parser::new(doc);
    let (value, p_next) = _try_parse_nude_integer(&parser).unwrap().unwrap();
    assert_eq!(value, 123);
    assert_eq!(p_next.remain(), " rest");

    let doc_no_int = "abc";
    let parser_no_int = Parser::new(doc_no_int);
    assert!(_try_parse_nude_integer(&parser_no_int)
        .unwrap()
        .is_none());

    let doc_empty = "";
    let parser_empty = Parser::new(doc_empty);
    assert!(_try_parse_nude_integer(&parser_empty)
        .unwrap()
        .is_none());
}

#[test]
fn test_try_parse_nude_float() {
    let doc = "123.45 rest";
    let parser = Parser::new(doc);
    let (value, p_next) = _try_parse_nude_float(&parser).unwrap().unwrap();
    assert_eq!(value, 123.45);
    assert_eq!(p_next.remain(), " rest");

    let doc_no_float = "123 rest";
    let parser_no_float = Parser::new(doc_no_float);
    assert!(_try_parse_nude_float(&parser_no_float)
        .unwrap()
        .is_none());

    let doc_just_dot = ". rest";
    let parser_just_dot = Parser::new(doc_just_dot);
    assert!(_try_parse_nude_float(&parser_just_dot)
        .unwrap()
        .is_none());

    let doc_empty = "";
    let parser_empty = Parser::new(doc_empty);
    assert!(_try_parse_nude_float(&parser_empty)
        .unwrap()
        .is_none());
}

#[test]
fn test_try_parse_nude_bool() {
    let doc_true = "true rest";
    let parser_true = Parser::new(doc_true);
    let (value_true, p_next_true) = _try_parse_nude_bool(&parser_true).unwrap().unwrap();
    assert_eq!(value_true, true);
    assert_eq!(p_next_true.remain(), " rest");

    let doc_false = "false rest";
    let parser_false = Parser::new(doc_false);
    let (value_false, p_next_false) = _try_parse_nude_bool(&parser_false).unwrap().unwrap();
    assert_eq!(value_false, false);
    assert_eq!(p_next_false.remain(), " rest");

    let doc_no_bool = "other rest";
    let parser_no_bool = Parser::new(doc_no_bool);
    assert!(_try_parse_nude_bool(&parser_no_bool)
        .unwrap()
        .is_none());
}

#[test]
fn test_try_parse_nude_string() {
    let doc = "hello/world.txt_123 rest";
    let parser = Parser::new(doc);
    let (value, p_next) = _try_parse_nude_string(&parser).unwrap().unwrap();
    assert_eq!(value, "hello/world.txt_123");
    assert_eq!(p_next.remain(), " rest");

    let doc_empty = "";
    let parser_empty = Parser::new(doc_empty);
    assert!(_try_parse_nude_string(&parser_empty)
        .unwrap()
        .is_none());

    let doc_with_space = "hello world";
    let parser_with_space = Parser::new(doc_with_space);
    let (value_space, p_next_space) = _try_parse_nude_string(&parser_with_space)
        .unwrap()
        .unwrap();
    assert_eq!(value_space, "hello");
    assert_eq!(p_next_space.remain(), " world");
}

#[test]
fn test_try_parse_value_nude() {
    let doc_int = "123 rest";
    let parser_int = Parser::new(doc_int);
    let (value_int, p_next_int) = _try_parse_value(&parser_int).unwrap().unwrap();
    assert_eq!(value_int, json!(123));
    assert_eq!(p_next_int.remain(), " rest");

    let doc_float = "123.45 rest";
    let parser_float = Parser::new(doc_float);
    let (value_float, p_next_float) = _try_parse_value(&parser_float).unwrap().unwrap();
    assert_eq!(value_float, json!(123.45));
    assert_eq!(p_next_float.remain(), " rest");

    let doc_bool = "true rest";
    let parser_bool = Parser::new(doc_bool);
    let (value_bool, p_next_bool) = _try_parse_value(&parser_bool).unwrap().unwrap();
    assert_eq!(value_bool, json!(true));
    assert_eq!(p_next_bool.remain(), " rest");

    let doc_string = "nude_string rest";
    let parser_string = Parser::new(doc_string);
    let (value_string, p_next_string) = _try_parse_value(&parser_string).unwrap().unwrap();
    assert_eq!(value_string, json!("nude_string"));
    assert_eq!(p_next_string.remain(), " rest");

    let doc_empty = "";
    let parser_empty = Parser::new(doc_empty);
    assert!(_try_parse_value(&parser_empty).unwrap().is_none());
}
