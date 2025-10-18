use crate::ast2::{Position, Range};

pub fn create_position(offset: usize, line: usize, column: usize) -> Position {
    Position { offset, line, column }
}

pub fn create_range(begin_offset: usize, begin_line: usize, begin_column: usize,
                    end_offset: usize, end_line: usize, end_column: usize) -> Range {
    Range {
        begin: Position { offset: begin_offset, line: begin_line, column: begin_column },
        end: Position { offset: end_offset, line: end_line, column: end_column },
    }
}
