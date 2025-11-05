//! This module implements the `ForgetPolicy` for the `@forget` tag. The `@forget`
//! tag is a static tag that instructs the execution engine to clear the currently
//! accumulated `ModelContent` in the [`Collector`]. This is useful for resetting
//! the context provided to an external model.
use super::Result;

use super::execute::{Collector, Worker};
use super::tags::StaticPolicy;
//use super::variables::Variables;
use crate::ast2::Tag;

/// Implements the static policy for the `@forget` tag.
///
/// The `@forget` tag clears the accumulated [`ModelContent`] in the [`Collector`].
pub struct ForgetPolicy;

impl StaticPolicy for ForgetPolicy {
    /// Clears the accumulated [`ModelContent`] in the [`Collector`].
    ///
    /// This method is called when an `@forget` tag is encountered. It resets
    /// the `context` field of the [`Collector`], effectively discarding all
    /// previously collected content.
    ///
    /// # Arguments
    ///
    /// * `worker` - A reference to the [`Worker`] instance (unused in this policy).
    /// * `collector` - The current [`Collector`] state.
    /// * `tag` - The [`Tag`] being processed (unused in this policy).
    ///
    /// # Returns
    ///
    /// A `Result` containing the updated [`Collector`] with its context cleared.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use crate::execute2::{Collector, Worker, ModelContentItem, ForgetPolicy};
    /// # use crate::execute2::tags::StaticPolicy;
    /// # use crate::ast2::{Tag, CommandKind, Parameters, Arguments, Range, Position};
    /// # use std::sync::Arc;
    /// # use crate::file::MockFileAccessor;
    /// # use crate::path::MockPathResolver;
    /// let mut collector = Collector::new();
    /// collector = collector.push_item(ModelContentItem::user("Some content."));
    /// assert!(!collector.context().0.is_empty());
    ///
    /// let worker = Worker::new(Arc::new(MockFileAccessor::new()), Arc::new(MockPathResolver::new()));
    /// let tag = Tag { command: CommandKind::Forget, parameters: Parameters::new(), arguments: Arguments::new(), range: Range::new(Position::new(0,0,0), Position::new(0,0,0)) };
    ///
    /// let updated_collector = ForgetPolicy::collect_static_tag(&worker, collector, &tag).unwrap();
    /// assert!(updated_collector.context().0.is_empty());
    /// ```
    fn collect_static_tag(worker: &Worker, collector: Collector, tag: &Tag) -> Result<Collector> {
        tracing::debug!(
            "tag_forget::ForgetPolicy::collect_static_tag\nTag = {:?}\n",
            tag
        );
        Ok(collector.forget())
    }
}
