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
fn test_try_parse_tag0_simple() -> Result<()> {
    let mut parser = Parser::new("@answer arg1 arg2");
    let tag = _try_parse_tag0(&mut parser)?.unwrap();
    assert!(matches!(tag.command, CommandKind::Answer));
    assert_eq!(tag.arguments.arguments.len(), 2);
    assert_eq!(tag.arguments.arguments[0].value, "arg1");
    assert_eq!(tag.arguments.arguments[1].value, "arg2");
    assert_eq!(tag.range, create_range(0, 1, 1, 17, 1, 18));
    Ok(())
}
