use serde::{Deserialize, Serialize};

use super::anchor::Anchor;
use super::tag::Tag;
use super::text::Text;
use super::comment::Comment;

/// An enum representing any of the top-level content types in a document.
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Content {
    /// A plain text block.
    Text(Text),
    /// A command tag (`@...`).
    Tag(Tag),
    /// A processing anchor (`<!-- ... -->`).
    Anchor(Anchor),
    /// A comment to ignore
    Comment(Comment),
}
