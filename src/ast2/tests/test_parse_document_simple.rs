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
fn test_parse_document_simple() -> Result<()> {
    let document_str = "Hello world";
    let document = parse_document(document_str)?;
    assert_eq!(document.content.len(), 1);
    assert_eq!(document.range, create_range(0, 1, 1, 11, 1, 12));
    Ok(())
}
