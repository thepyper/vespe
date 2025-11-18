use crate::ast2::parser::Parser;
use crate::ast2::parser::values;
use serde_json::json;

#[test]
fn test_try_parse_nude_integer() {
    let (value, p_next) = values::_try_parse_nude_integer(&parser).unwrap().unwrap();
    assert_eq!(value, 123);
    assert_eq!(p_next.remain(), " rest");

    let doc_no_int = "abc";
    let parser_no_int = Parser::new(doc_no_int);
    assert!(values::_try_parse_nude_integer(&parser_no_int)
        .unwrap()
        .is_none());

    let doc_empty = "";
    let parser_empty = Parser::new(doc_empty);
    assert!(values::_try_parse_nude_integer(&parser_empty)
        .unwrap()
        .is_none());
}

#[test]
fn test_try_parse_nude_float() {
    let (value, p_next) = values::_try_parse_nude_float(&parser).unwrap().unwrap();
    assert_eq!(value, 123.45);
    assert_eq!(p_next.remain(), " rest");

    let doc_no_float = "123 rest";
    let parser_no_float = Parser::new(doc_no_float);
    assert!(values::_try_parse_nude_float(&parser_no_float)
        .unwrap()
        .is_none());

    let doc_just_dot = ". rest";
    let parser_just_dot = Parser::new(doc_just_dot);
    assert!(values::_try_parse_nude_float(&parser_just_dot)
        .unwrap()
        .is_none());

    let doc_empty = "";
    let parser_empty = Parser::new(doc_empty);
    assert!(values::_try_parse_nude_float(&parser_empty)
        .unwrap()
        .is_none());
}

#[test]
fn test_try_parse_nude_bool() {
    let (value_true, p_next_true) = values::_try_parse_nude_bool(&parser_true).unwrap().unwrap();
    assert_eq!(value_true, true);
    assert_eq!(p_next_true.remain(), " rest");

    let (value_false, p_next_false) = values::_try_parse_nude_bool(&parser_false).unwrap().unwrap();
    assert_eq!(value_false, false);
    assert_eq!(p_next_false.remain(), " rest");

    let doc_no_bool = "other rest";
    let parser_no_bool = Parser::new(doc_no_bool);
    assert!(values::_try_parse_nude_bool(&parser_no_bool)
        .unwrap()
        .is_none());
}

#[test]
fn test_try_parse_nude_string() {
    let (value, p_next) = values::_try_parse_nude_string(&parser).unwrap().unwrap();
    assert_eq!(value, "hello/world.txt_123");
    assert_eq!(p_next.remain(), " rest");

    let doc_empty = "";
    let parser_empty = Parser::new(doc_empty);
    assert!(values::_try_parse_nude_string(&parser_empty)
        .unwrap()
        .is_none());

    let doc_with_space = "hello world";
    let parser_with_space = Parser::new(doc_with_space);
    let (value_space, p_next_space) = values::_try_parse_nude_string(&parser_with_space)
        .unwrap()
        .unwrap();
    assert_eq!(value_space, "hello");
    assert_eq!(p_next_space.remain(), " world");
}

#[test]
fn test_try_parse_value_nude() {
    let (value_int, p_next_int) = values::_try_parse_value(&parser_int).unwrap().unwrap();
    assert_eq!(value_int, json!(123));
    assert_eq!(p_next_int.remain(), " rest");

    let (value_float, p_next_float) = values::_try_parse_value(&parser_float).unwrap().unwrap();
    assert_eq!(value_float, json!(123.45));
    assert_eq!(p_next_float.remain(), " rest");

    let (value_bool, p_next_bool) = values::_try_parse_value(&parser_bool).unwrap().unwrap();
    assert_eq!(value_bool, json!(true));
    assert_eq!(p_next_bool.remain(), " rest");

    let (value_string, p_next_string) = values::_try_parse_value(&parser_string).unwrap().unwrap();
    assert_eq!(value_string, json!("nude_string"));
    assert_eq!(p_next_string.remain(), " rest");

    let doc_empty = "";
    let parser_empty = Parser::new(doc_empty);
    assert!(values::_try_parse_value(&parser_empty).unwrap().is_none());
}
