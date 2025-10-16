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
fn test_try_parse_argument_quoted() -> Result<()> {
    let mut parser = Parser::new("\"hello world\"");
    let arg = _try_parse_argument(&mut parser)?.unwrap();
    assert_eq!(arg.value, "hello world");
    assert_eq!(arg.range, create_range(0, 1, 1, 13, 1, 14));
    Ok(())
}