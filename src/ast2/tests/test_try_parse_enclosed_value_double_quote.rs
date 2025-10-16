use super::*
use anyhow::Result;
use serde_json::json;

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
fn test_try_parse_enclosed_value_double_quote() -> Result<()> {
    let mut parser = Parser::new("\"hello world\"");
    let value = _try_parse_enclosed_value(&mut parser, "\"")?;
    assert_eq!(value, Some(json!("hello world")));
    assert_eq!(parser.get_position(), create_pos(13, 1, 14));
    Ok(())
}
