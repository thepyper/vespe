use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use uuid::{uuid, Uuid};

/// A block of plain text content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Text {
    /// The raw string content of the text block.
    pub content: String,
    /// The location of the text block in the source document.
    pub range: Range,
}

