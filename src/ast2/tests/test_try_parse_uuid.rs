use super::*;
use anyhow::Result;
use uuid::Uuid;

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
fn test_try_parse_uuid() -> Result<()> {
    let uuid_str = "123e4567-e89b-12d3-a456-426614174000";
    let mut parser = Parser::new(uuid_str);
    let uuid = _try_parse_uuid(&mut parser)?.unwrap();
    assert_eq!(uuid.to_string(), uuid_str);
    assert_eq!(parser.get_position(), create_pos(36, 1, 37));

    let mut parser = Parser::new("invalid-uuid");
    let result = _try_parse_uuid(&mut parser);
    assert!(matches!(result, Err(e) if e.downcast_ref::<ParsingError>().unwrap().to_string().contains("Invalid UUID format")));
    Ok(())
}
