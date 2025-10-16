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
fn test_parser_advance() {
    let document = "Hello";
    let mut parser = Parser::new(document);
    assert_eq!(parser.advance(), Some('H'));
    assert_eq!(parser.get_position(), create_pos(1, 1, 2));
    assert_eq!(parser.advance(), Some('e'));
    assert_eq!(parser.get_position(), create_pos(2, 1, 3));
    parser.advance();
    parser.advance();
    parser.advance();
    assert_eq!(parser.advance(), None);
    assert_eq!(parser.get_position(), create_pos(5, 1, 6));
}
