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
fn test_try_parse_nude_float() -> Result<()> {
    let mut parser = Parser::new("123.45");
    assert_eq!(_try_parse_nude_float(&mut parser)?,
               Some(123.45));
    assert_eq!(parser.get_position(), create_pos(6, 1, 7));

    let mut parser = Parser::new(".5");
    assert_eq!(_try_parse_nude_float(&mut parser)?,
               Some(0.5));
    assert_eq!(parser.get_position(), create_pos(2, 1, 3));

    let mut parser = Parser::new("123");
    assert_eq!(_try_parse_nude_float(&mut parser)?,
               None);
    assert_eq!(parser.get_position(), create_pos(0, 1, 1));

    let mut parser = Parser::new("abc");
    assert_eq!(_try_parse_nude_float(&mut parser)?,
               None);
    assert_eq!(parser.get_position(), create_pos(0, 1, 1));

    let mut parser = Parser::new("");
    assert_eq!(_try_parse_nude_float(&mut parser)?,
               None);
    assert_eq!(parser.get_position(), create_pos(0, 1, 1));
    Ok(())
}
