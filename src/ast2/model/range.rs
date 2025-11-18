use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use uuid::{uuid, Uuid};

use super::position::Position;

/// Represents a span of text in the source document, from a `begin` to an `end`
/// `Position`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub struct Range {
    pub begin: Position,
    pub end: Position,
}

impl Ord for Range {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.begin.cmp(&other.begin) {
            Ordering::Equal => self.end.cmp(&other.end),
            x => x,
        }
    }
}

impl Default for Range {
    fn default() -> Self {
        Range::null()
    }
}

impl Range {
    /// Creates a "null" or invalid range.
    pub fn null() -> Self {
        Range {
            begin: Position::null(),
            end: Position::null(),
        }
    }
    /// Checks if the range is valid.
    pub fn is_valid(&self) -> bool {
        if !self.begin.is_valid() {
            false
        } else if !self.end.is_valid() {
            false
        } else {
            self.begin.offset <= self.end.offset
        }
    }
}

