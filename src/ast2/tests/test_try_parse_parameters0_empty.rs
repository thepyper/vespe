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
fn test_try_parse_parameters0_empty() -> Result<()> {
    let mut parser = Parser::new("{}");
    let params = _try_parse_parameters0(&mut parser)?.unwrap();
    assert!(params.parameters.as_object().unwrap().is_empty());
    assert_eq!(params.range, create_range(0, 1, 1, 2, 1, 3));
    Ok(())
}