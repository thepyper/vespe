use crate::ast::*;

fn create_pos(offset: usize, line: usize, column: usize) -> Position {
    Position { offset, line, column }
}

fn create_range(start_offset: usize, start_line: usize, start_column: usize, end_offset: usize, end_line: usize, end_column: usize) -> Range {
    Range {
        start: create_pos(start_offset, start_line, start_column),
        end: create_pos(end_offset, end_line, end_column),
    }
}

#[test]
fn test_parser_parse_number_negative() {
    let mut parser = Parser::new("-123 rest");
    let (val, r) = parser.parse_number().unwrap().unwrap();
    assert_eq!(val, ParameterValue::Integer(-123));
    assert_eq!(r, create_range(0, 1, 1, 4, 1, 5));
    assert_eq!(parser.remaining_slice(), " rest");
}

#[test]
fn test_parser_parse_boolean_true() {
    let mut parser = Parser::new("true rest");
    let (val, r) = parser.parse_boolean().unwrap();
    assert_eq!(val, ParameterValue::Boolean(true));
    assert_eq!(r, create_range(0, 1, 1, 4, 1, 5));
    assert_eq!(parser.remaining_slice(), " rest");
}

#[test]
fn test_parser_parse_boolean_false() {
    let mut parser = Parser::new("false rest");
    let (val, r) = parser.parse_boolean().unwrap();
    assert_eq!(val, ParameterValue::Boolean(false));
    assert_eq!(r, create_range(0, 1, 1, 5, 1, 6));
    assert_eq!(parser.remaining_slice(), " rest");
}

#[test]
fn test_parse_parameters_empty() {
    let mut parser = Parser::new("{}");
    let (params, r) = parse_parameters(&mut parser).unwrap();
    assert!(params.is_empty());
    assert_eq!(r, create_range(0, 1, 1, 2, 1, 3));
}

#[test]
fn test_parse_parameters_single_unquoted() {
    let mut parser = Parser::new("{key: value}");
    let (params, r) = parse_parameters(&mut parser).unwrap();
    assert_eq!(params.len(), 1);
    assert_eq!(params["key"], ParameterValue::String("value".to_string()));
    assert_eq!(r, create_range(0, 1, 1, 12, 1, 13));
}
