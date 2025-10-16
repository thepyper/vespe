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
fn test_try_parse_parameters0_missing_comma_or_brace() -> Result<()> {
    let mut parser = Parser::new("{key1: \"value1\" key2: 123}");
    let result = _try_parse_parameters0(&mut parser);
    assert!(matches!(result, Err(e) if e.downcast_ref::<ParsingError>().unwrap().to_string().contains("Expected , or } ")));
    Ok(())
}