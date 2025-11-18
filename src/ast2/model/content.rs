use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use uuid::{uuid, Uuid};

use super::text::Text;
use super::tag::Tag;
use super::anchor::Anchor;

/// An enum representing any of the top-level content types in a document.
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Content {
    /// A plain text block.
    Text(Text),
    /// A command tag (`@...`).
    Tag(Tag),
    /// A processing anchor (`<!-- ... -->`).
    Anchor(Anchor),
}