//! This module provides utility functions and data structures that support the
//! core execution logic. These utilities are designed to simplify common tasks
//! such as indexing anchors within the Abstract Syntax Tree (AST).
use std::collections::HashMap;
use uuid::Uuid;

use crate::ast2::*;

/// A utility struct for efficiently looking up the positions of anchor tags
/// (both begin and end) within a document's content.
///
/// This index is built once from the parsed AST content and allows for quick
/// retrieval of an anchor's corresponding start or end tag based on its UUID.
pub struct AnchorIndex {
    /// Maps an anchor's UUID to the index of its beginning tag in the content vector.
    begin: HashMap<Uuid, usize>, // uid -> content index
    /// Maps an anchor's UUID to the index of its ending tag in the content vector.
    end: HashMap<Uuid, usize>, // uid -> content index
}

impl AnchorIndex {
    /// Creates a new `AnchorIndex` from a slice of `Content`.
    ///
    /// It iterates through the provided content, identifying all `Anchor` items
    /// and populating the `begin` and `end` hashmaps with their UUIDs and indices.
    ///
    /// # Arguments
    ///
    /// * `content` - A slice of [`Content`] representing the parsed document.
    ///
    /// # Returns
    ///
    /// A new `AnchorIndex` instance.
    ///
    /// # Examples
    pub fn new(content: &[Content]) -> Self {
        let mut begin = HashMap::new();
        let mut end = HashMap::new();

        for (i, line) in content.iter().enumerate() {
            match line {
                Content::Anchor(anchor) => match anchor.kind {
                    AnchorKind::Begin => {
                        let _ = begin.insert(anchor.uuid, i);
                    }
                    AnchorKind::End => {
                        let _ = end.insert(anchor.uuid, i);
                    }
                },
                _ => {}
            }
        }

        Self { begin, end }
    }

    /// Retrieves the index of the beginning tag for a given anchor UUID.
    ///
    /// # Arguments
    ///
    /// * `uid` - The UUID of the anchor.
    ///
    /// # Returns
    ///
    /// An `Option<usize>` containing the index if found, otherwise `None`.
    pub fn get_begin(&self, uid: &Uuid) -> Option<usize> {
        self.begin.get(uid).copied()
    }

    /// Retrieves the index of the ending tag for a given anchor UUID.
    ///
    /// # Arguments
    ///
    /// * `uid` - The UUID of the anchor.
    ///
    /// # Returns
    ///
    /// An `Option<usize>` containing the index if found, otherwise `None`.
    pub fn get_end(&self, uid: &Uuid) -> Option<usize> {
        self.end.get(uid).copied()
    }
}
