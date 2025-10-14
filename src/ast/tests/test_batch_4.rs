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
fn test_parse_parameters_multiple_quoted() {
    let mut parser = Parser::new("{\"key1\": \"value1\", 'key2': 123, key3: true}");
    let (params, r) = parse_parameters(&mut parser).unwrap();
    assert_eq!(params.len(), 3);
    assert_eq!(params["key1"], ParameterValue::String("value1".to_string()));
    assert_eq!(params["key2"], ParameterValue::Integer(123));
    assert_eq!(params["key3"], ParameterValue::Boolean(true));
    assert_eq!(r, create_range(0, 1, 1, 43, 1, 44));
}

#[test]
fn test_parse_parameters_invalid_syntax() {
    let mut parser = Parser::new("{key: }");
    let err = parse_parameters(&mut parser).unwrap_err();
    assert!(matches!(err, ParsingError::InvalidSyntax { .. }));
}

#[test]
fn test_parse_argument_quoted() {
    let mut parser = Parser::new("\"arg1\" rest");
    let (arg, r) = parse_argument(&mut parser).unwrap().unwrap();
    assert_eq!(arg, "arg1");
    assert_eq!(r, create_range(0, 1, 1, 6, 1, 7));
    assert_eq!(parser.remaining_slice(), " rest");
}

#[test]
fn test_parse_argument_unquoted() {
    let mut parser = Parser::new("arg1 rest");
    let (arg, r) = parse_argument(&mut parser).unwrap().unwrap();
    assert_eq!(arg, "arg1");
    assert_eq!(r, create_range(0, 1, 1, 4, 1, 5));
    assert_eq!(parser.remaining_slice(), " rest");
}

#[test]
fn test_parse_arguments_multiple() {
    let mut parser = Parser::new("arg1 \"arg2 with spaces\" arg3");
    let (args, r) = parse_arguments(&mut parser).unwrap();
    assert_eq!(args, vec!["arg1", "arg2 with spaces", "arg3"]);
    assert_eq!(r, create_range(0, 1, 1, 28, 1, 29));
}

