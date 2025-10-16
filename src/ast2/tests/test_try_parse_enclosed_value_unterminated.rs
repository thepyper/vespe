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
fn test_try_parse_enclosed_value_unterminated() -> Result<()> {
    let mut parser = Parser::new(""hello");
    let result = _try_parse_enclosed_value(&mut parser, "");
    assert!(matches!(result, Err(e) if e.downcast_ref::<ParsingError>().unwrap().to_string().contains("Unterminated string")));
    Ok(())
}
