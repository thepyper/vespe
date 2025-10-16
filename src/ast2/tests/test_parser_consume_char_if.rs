use super::*;
use anyhow::Result;

fn create_pos(offset: usize, line: usize, column: usize) -> Position {
    Position { offset, line, column }
}

fn create_range(begin_offset: usize, begin_line: usize, begin_column: usize, end_offset: usize, end_line: usize, end_column: usize) -> Range {
    Range {
        begin: create_pos(begin_offset, begin_line, begin_column),
        end: create_pos(end_offset, end_line, end_column),
    }
}

#[test]
fn test_parser_consume_char_if() {
    let document = "123abc";
    let mut parser = Parser::new(document);
    assert_eq!(parser.consume_char_if(|c| c.is_ascii_digit()), Some('1'));
    assert_eq!(parser.get_position(), create_pos(1, 1, 2));
    assert_eq!(parser.consume_char_if(|c| c.is_ascii_digit()), Some('2'));
    assert_eq!(parser.get_position(), create_pos(2, 1, 3));
    assert_eq!(parser.consume_char_if(|c| c.is_ascii_alphabetic()), None);
    assert_eq!(parser.get_position(), create_pos(2, 1, 3));
}
