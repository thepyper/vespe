use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Represents a specific point in the source document.
///
/// All locations are 1-based for user-facing error reporting, while the offset
/// is 0-based for internal use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub struct Position {
    /// 0-based character offset from the beginning of the file.
    pub offset: usize,
    /// 1-based line number.
    pub line: usize,
    /// 1-based column number.
    pub column: usize,
}

impl Position {
    /// Creates a "null" or invalid position. Useful for initializing ranges
    /// before a valid position is known.
    pub fn null() -> Self {
        Position {
            offset: 0,
            line: 0,
            column: 0,
        }
    }
    /// Checks if the position is valid (line and column are not zero).
    pub fn is_valid(&self) -> bool {
        (self.line > 0) && (self.column > 0)
    }
}

impl Ord for Position {
    fn cmp(&self, other: &Self) -> Ordering {
        self.offset.cmp(&other.offset)
    }
}

impl Default for Position {
    fn default() -> Self {
        Position::null()
    }
}
