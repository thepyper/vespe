use crate::ast2::error::Ast2Error;
use crate::ast2::parser::Parser;
use crate::ast2::parser::parameters;
use serde_json::json;

#[test]
fn test_try_parse_parameter_valid() {
    let parser = Parser::new("key=value rest");
    let ((key, value), p_next) = parameters::_try_parse_parameter(&parser).unwrap().unwrap();
    assert_eq!(key, "key");
    assert_eq!(value, json!("value"));
    assert_eq!(p_next.remain(), " rest");
}

#[test]
fn test_try_parse_parameter_with_spaces() {
    let parser = Parser::new("key=\"value with spaces\"  rest");
    let ((key, value), p_next) = parameters::_try_parse_parameter(&parser).unwrap().unwrap();
    assert_eq!(key, "key");
    assert_eq!(value, json!("value with spaces"));
    assert_eq!(p_next.remain(), "  rest");
}

#[test]
fn test_try_parse_parameter_missing_value() {
    let parser = Parser::new("key= rest");
    let ((key, value), p_next) = parameters::_try_parse_parameter(&parser).unwrap().unwrap();
    assert_eq!(key, "key");
    assert_eq!(value, json!("rest"));
    assert_eq!(p_next.remain(), "");
}

#[test]
fn test_try_parse_parameter_no_equal() {
    let parser = Parser::new("key rest");
    let (params, p_next) = parameters::_try_parse_parameter(&parser).unwrap().unwrap();
    assert!(params.0 == "key");
}

#[test]
fn test_try_parse_parameters_empty() {
    let parser = Parser::new("{} rest");
    let (params, p_next) = parameters::_try_parse_parameters(&parser).unwrap().unwrap();
    assert!(params.parameters.properties.is_empty());
    assert_eq!(p_next.remain(), " rest");

    let begin_str = "{";
    let end_str = "}";
    assert_eq!(params.range.begin.offset, 0);
    assert_eq!(params.range.end.offset, begin_str.len() + end_str.len());
}

#[test]
fn test_try_parse_parameters_single() {
    let parser = Parser::new("{key=value} rest");
    let (params, p_next) = parameters::_try_parse_parameters(&parser).unwrap().unwrap();
    assert_eq!(params.parameters.properties.len(), 1);
    // TODO assert_eq!(params.parameters["key"], json!("value"));
    assert_eq!(p_next.remain(), " rest");

    let full_str = "{key=value}";
    assert_eq!(params.range.begin.offset, 0);
    assert_eq!(params.range.end.offset, full_str.len());
}

#[test]
fn test_try_parse_parameters_multiple() {
    let parser = Parser::new("{key1=value1, key2=\"value 2\"} rest");
    let (params, p_next) = parameters::_try_parse_parameters(&parser).unwrap().unwrap();
    assert_eq!(params.parameters.properties.len(), 2);
    //assert_eq!(params.parameters["key1"], json!("value1"));
    //assert_eq!(params.parameters["key2"], json!("value 2"));
    assert_eq!(p_next.remain(), " rest");

    let full_str = "{key1=value1, key2=\"value 2\"}";
    assert_eq!(params.range.begin.offset, 0);
    assert_eq!(params.range.end.offset, full_str.len());
}

#[test]
fn test_try_parse_parameters_missing_comma() {
    let parser = Parser::new("{key1=value1 key2=value2}");
    let result = parameters::_try_parse_parameters(&parser);
    assert!(matches!(
        result,
        Err(Ast2Error::MissingCommaInParameters { .. })
    ));
}

#[test]
fn test_try_parse_parameters_unclosed() {
    let parser = Parser::new("{key=value");
    let result = parameters::_try_parse_parameters(&parser);
    assert!(matches!(
        result,
        Err(Ast2Error::MissingCommaInParameters { .. })
    )); // Currently reports missing comma
}

#[test]
fn test_try_parse_parameters_no_opening_bracket() {
    let parser = Parser::new("key=value}");
    let result = parameters::_try_parse_parameters(&parser).unwrap();
    assert!(result.is_none());
}
