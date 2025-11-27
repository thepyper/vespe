use serde::{Deserialize, Serialize};

use super::range::Range;

/// A block of plain text content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Comment {
    /// The raw string content of the comment block.
    pub content: String,
    /// The location of the text block in the source document.
    pub range: Range,
}
