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
fn test_parser_parse_quoted_string_with_escapes() {
            let mut parser = Parser::new("\"Hello\nWorld\"");
            let (s, r) = parser.parse_quoted_string('"').unwrap();
            assert_eq!(s, "Hello\nWorld");
            assert_eq!(r, create_range(0, 1, 1, 15, 1, 16));}

#[test]
fn test_parser_parse_quoted_string_unterminated() {
    let mut parser = Parser::new("\"hello world");
    let err = parser.parse_quoted_string('"').unwrap_err();
    assert_eq!(err, ParsingError::UnterminatedString { range: create_range(0, 1, 1, 12, 1, 13) });
}

#[test]
fn test_parser_parse_unquoted_identifier() {
    let mut parser = Parser::new("my_identifier123 rest");
    let (s, r) = parser.parse_unquoted_identifier().unwrap();
    assert_eq!(s, "my_identifier123");
    assert_eq!(r, create_range(0, 1, 1, 16, 1, 17));
    assert_eq!(parser.remaining_slice(), " rest");
}

#[test]
fn test_parser_parse_number_integer() {
    let mut parser = Parser::new("12345 rest");
    let (val, r) = parser.parse_number().unwrap().unwrap();
    assert_eq!(val, ParameterValue::Integer(12345));
    assert_eq!(r, create_range(0, 1, 1, 5, 1, 6));
    assert_eq!(parser.remaining_slice(), " rest");
}

#[test]
fn test_parser_parse_number_float() {
    let mut parser = Parser::new("123.45 rest");
    let (val, r) = parser.parse_number().unwrap().unwrap();
    assert_eq!(val, ParameterValue::Float(123.45));
    assert_eq!(r, create_range(0, 1, 1, 6, 1, 7));
    assert_eq!(parser.remaining_slice(), " rest");
}

