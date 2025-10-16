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
fn test_parser_new() {
    let document = "Hello";
    let parser = Parser::new(document);
    assert_eq!(parser.get_position(), create_pos(0, 1, 1));
    assert_eq!(parser.remain(), "Hello");
}
