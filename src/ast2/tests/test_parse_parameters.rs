use crate::ast2::{Parser, Ast2Error};
use serde_json::json;

#[test]
fn test_try_parse_parameter_valid() {
    let doc = "key=value rest";
    let parser = Parser::new(doc);
    let ((key, value), p_next) = super::super::_try_parse_parameter(&parser).unwrap().unwrap();
    assert_eq!(key, "key");
    assert_eq!(value, json!("value"));
    assert_eq!(p_next.remain(), " rest");
}

#[test]
fn test_try_parse_parameter_with_spaces() {
    let doc = "  key  =  \"value with spaces\"  rest";
    let parser = Parser::new(doc);
    let ((key, value), p_next) = super::super::_try_parse_parameter(&parser).unwrap().unwrap();
    assert_eq!(key, "key");
    assert_eq!(value, json!("value with spaces"));
    assert_eq!(p_next.remain(), "  rest");
}

#[test]
fn test_try_parse_parameter_missing_value() {
    let doc = "key= rest";
    let parser = Parser::new(doc);
    let result = super::super::_try_parse_parameter(&parser);
    assert!(matches!(result, Err(Ast2Error::MissingParameterValue { .. })));
}

#[test]
fn test_try_parse_parameter_no_equal() {
    let doc = "key value rest";
    let parser = Parser::new(doc);
    let result = super::super::_try_parse_parameter(&parser).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_try_parse_parameters_empty() {
    let doc = "[] rest";
    let parser = Parser::new(doc);
    let (params, p_next) = super::super::_try_parse_parameters(&parser).unwrap().unwrap();
    assert!(params.parameters.is_empty());
    assert_eq!(p_next.remain(), " rest");

    let begin_str = "[";
    let end_str = "]";
    assert_eq!(params.range.begin.offset, 0);
    assert_eq!(params.range.end.offset, begin_str.len() + end_str.len());
}

#[test]
fn test_try_parse_parameters_single() {
    let doc = "[key=value] rest";
    let parser = Parser::new(doc);
    let (params, p_next) = super::super::_try_parse_parameters(&parser).unwrap().unwrap();
    assert_eq!(params.parameters.len(), 1);
    assert_eq!(params.parameters["key"], json!("value"));
    assert_eq!(p_next.remain(), " rest");

    let full_str = "[key=value]";
    assert_eq!(params.range.begin.offset, 0);
    assert_eq!(params.range.end.offset, full_str.len());
}

#[test]
fn test_try_parse_parameters_multiple() {
    let doc = "[key1=value1, key2=\"value 2\"] rest";
    let parser = Parser::new(doc);
    let (params, p_next) = super::super::_try_parse_parameters(&parser).unwrap().unwrap();
    assert_eq!(params.parameters.len(), 2);
    assert_eq!(params.parameters["key1"], json!("value1"));
    assert_eq!(params.parameters["key2"], json!("value 2"));
    assert_eq!(p_next.remain(), " rest");

    let full_str = "[key1=value1, key2=\"value 2\"]";
    assert_eq!(params.range.begin.offset, 0);
    assert_eq!(params.range.end.offset, full_str.len());
}

#[test]
fn test_try_parse_parameters_missing_comma() {
    let doc = "[key1=value1 key2=value2]";
    let parser = Parser::new(doc);
    let result = super::super::_try_parse_parameters(&parser);
    assert!(matches!(result, Err(Ast2Error::MissingCommaInParameters { .. })));
}

#[test]
fn test_try_parse_parameters_unclosed() {
    let doc = "[key=value";
    let parser = Parser::new(doc);
    let result = super::super::_try_parse_parameters(&parser);
    assert!(matches!(result, Err(Ast2Error::MissingCommaInParameters { .. }))); // Currently reports missing comma
}

#[test]
fn test_try_parse_parameters_no_opening_bracket() {
    let doc = "key=value] rest";
    let parser = Parser::new(doc);
    let result = super::super::_try_parse_parameters(&parser).unwrap();
    assert!(result.is_none());
}
