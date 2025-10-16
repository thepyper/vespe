use crate::ast2::{Position, Range};
use anyhow::Result;

pub fn create_pos(offset: usize, line: usize, column: usize) -> Position {
    Position { offset, line, column }
}

pub fn create_range(begin_offset: usize, begin_line: usize, begin_column: usize, end_offset: usize, end_line: usize, end_column: usize) -> Range {
    Range {
        begin: create_pos(begin_offset, begin_line, begin_column),
        end: create_pos(end_offset, end_line, end_column),
    }
}
