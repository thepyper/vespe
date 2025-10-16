use super::*
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
fn test_parser_skip_many_whitespaces() {
    let document = "  \t\rhello";
    let mut parser = Parser::new(document);
    parser.skip_many_whitespaces();
    assert_eq!(parser.get_position(), create_pos(4, 1, 5));
    assert_eq!(parser.remain(), "hello");
}
