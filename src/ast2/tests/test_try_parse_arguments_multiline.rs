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
fn test_try_parse_arguments_multiline() -> Result<()> {
    let mut parser = Parser::new("arg1\narg2");
    let args = _try_parse_arguments(&mut parser)?.unwrap();
    assert_eq!(args.arguments.len(), 1);
    assert_eq!(args.arguments[0].value, "arg1");
    assert_eq!(args.range, create_range(0, 1, 1, 4, 1, 5));
    assert_eq!(parser.get_position(), create_pos(4, 1, 5)); // Should stop before newline
    Ok(())
}

