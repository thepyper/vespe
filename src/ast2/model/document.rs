use serde::{Deserialize, Serialize};

use super::content::Content;
use super::range::Range;

/// The root of the Abstract Syntax Tree, representing a fully parsed document.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Document {
    /// A vector of the top-level content items that make up the document.
    pub content: Vec<Content>,
    /// The range spanning the entire document.
    pub range: Range,
}