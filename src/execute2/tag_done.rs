//! This module implements the `DonePolicy` for the `@done` tag. The `@done`
//! ... TODO doc
use super::{ExecuteError, Result};

use super::execute::{Collector, Worker};
use super::tags::StaticPolicy;
use crate::ast2::{JsonPlusEntity, Tag};

/// Implements the static policy for the `@done` tag.
///
// TODO doc
pub struct DonePolicy;

impl StaticPolicy for DonePolicy {
    // TODO doc
    fn collect_static_tag(_worker: &Worker, _collector: Collector, tag: &Tag) -> Result<Collector> {
        tracing::debug!(
            "tag_done::DonePolicy::collect_static_tag\nTag = {:?}\n",
            tag
        );
        // TODO
        unimplemented!()
    }
}
