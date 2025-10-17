use crate::ast::*;

fn create_pos(offset: usize, line: usize, column: usize) -> Position {
    Position {
        offset,
        line,
        column,
    }
}

fn create_range(
    start_offset: usize,
    start_line: usize,
    start_column: usize,
    end_offset: usize,
    end_line: usize,
    end_column: usize,
) -> Range {
    Range {
        start: create_pos(start_offset, start_line, start_column),
        end: create_pos(end_offset, end_line, end_column),
    }
}

#[test]
fn test_parser_new() {
    let document = "hello";
    let parser = Parser::new(document);
    assert_eq!(parser.current_pos, create_pos(0, 1, 1));
    assert_eq!(parser.document, "hello");
}

#[test]
fn test_parser_peek_consume_advance() {
    let mut parser = Parser::new("abc\n123");
    assert_eq!(parser.peek(), Some('a'));
    assert_eq!(parser.consume(), Some('a'));
    assert_eq!(parser.current_pos, create_pos(1, 1, 2));

    assert_eq!(parser.peek(), Some('b'));
    assert_eq!(parser.consume(), Some('b'));
    assert_eq!(parser.current_pos, create_pos(2, 1, 3));

    assert_eq!(parser.peek(), Some('c'));
    assert_eq!(parser.consume(), Some('c'));
    assert_eq!(parser.current_pos, create_pos(3, 1, 4));

    assert_eq!(parser.peek(), Some('\n'));
    assert_eq!(parser.consume(), Some('\n'));
    assert_eq!(parser.current_pos, create_pos(4, 2, 1)); // New line

    assert_eq!(parser.peek(), Some('1'));
    assert_eq!(parser.consume(), Some('1'));
    assert_eq!(parser.current_pos, create_pos(5, 2, 2));

    parser.advance_position_by_str("23");
    assert_eq!(parser.current_pos, create_pos(7, 2, 4));

    assert_eq!(parser.peek(), None);
    assert_eq!(parser.consume(), None);
}

#[test]
fn test_parser_take_while() {
    let mut parser = Parser::new("  hello world");
    parser.skip_whitespace();
    assert_eq!(parser.current_pos, create_pos(2, 1, 3));

    let word = parser.take_while(|c| c.is_alphabetic());
    assert_eq!(word, "hello");
    assert_eq!(parser.current_pos, create_pos(7, 1, 8));
}

#[test]
fn test_parser_parse_quoted_string_double_quotes() {
    let mut parser = Parser::new("\"hello world\"");
    let (s, r) = parser.parse_quoted_string('"').unwrap();
    assert_eq!(s, "hello world");
    assert_eq!(r, create_range(0, 1, 1, 13, 1, 14));
}

#[test]
fn test_parser_parse_quoted_string_single_quotes() {
    let mut parser = Parser::new("'hello world'");
    let (s, r) = parser.parse_quoted_string('\'').unwrap();
    assert_eq!(s, "hello world");
    assert_eq!(r, create_range(0, 1, 1, 13, 1, 14));
}
