use crate::ast2::{Position, Range};

pub fn create_pos(offset: usize, line: usize, column: usize) -> Position {
    Position { offset, line, column }
}

pub fn create_range(
    start_offset: usize,
    start_line: usize,
    start_column: usize,
    end_offset: usize,
    end_line: usize,
    end_column: usize,
) -> Range {
    Range {
        begin: create_pos(start_offset, start_line, start_column),
        end: create_pos(end_offset, end_line, end_column),
    }
}