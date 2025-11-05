//! This module implements the `SetPolicy` for the `@set` tag. The `@set` tag
//! is a static tag that allows for setting default parameters within the execution
//! context. These parameters can then be inherited or overridden by subsequent
//! tags and anchors, providing a way to configure behavior globally or locally.
use super::Result;

use super::execute::{Collector, Worker};
use super::tags::StaticPolicy;
//use super::variables::Variables;
use crate::ast2::Tag;

/// Implements the static policy for the `@set` tag.
///
/// The `@set` tag updates the default parameters in the [`Collector`].
pub struct SetPolicy;

impl StaticPolicy for SetPolicy {
    /// Sets the default parameters in the [`Collector`].
    ///
    /// This method is called when a `@set` tag is encountered. It takes the
    /// parameters defined in the tag and uses them to update the `default_parameters`
    /// field of the [`Collector`]. These defaults will then be applied to subsequent
    /// tags and anchors unless explicitly overridden.
    ///
    /// # Arguments
    ///
    /// * `worker` - A reference to the [`Worker`] instance (unused in this policy).
    /// * `collector` - The current [`Collector`] state.
    /// * `tag` - The [`Tag`] being processed, which contains the parameters to set.
    ///
    /// # Returns
    ///
    /// A `Result` containing the updated [`Collector`] with the new default parameters.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use crate::execute2::{Collector, Worker, SetPolicy};
    /// # use crate::execute2::tags::StaticPolicy;
    /// # use crate::ast2::{Tag, CommandKind, Parameters, Arguments, Range, Position, JsonPlusEntity};
    /// # use std::sync::Arc;
    /// # use crate::file::MockFileAccessor;
    /// # use crate::path::MockPathResolver;
    /// let worker = Worker::new(Arc::new(MockFileAccessor::new()), Arc::new(MockPathResolver::new()));
    /// let collector = Collector::new();
    ///
    /// let mut params = Parameters::new();
    /// params.parameters.properties.insert("model".to_string(), JsonPlusEntity::NudeString("gpt-4".to_string()));
    ///
    /// let tag = Tag {
    ///     command: CommandKind::Set,
    ///     parameters: params,
    ///     arguments: Arguments::new(),
    ///     range: Range::null(),
    /// };
    ///
    /// let updated_collector = SetPolicy::collect_static_tag(&worker, collector, &tag).unwrap();
    /// assert_eq!(
    ///     updated_collector.default_parameters.parameters.properties.get("model").unwrap(),
    ///     &JsonPlusEntity::NudeString("gpt-4".to_string())
    /// );
    /// ```
    fn collect_static_tag(worker: &Worker, collector: Collector, tag: &Tag) -> Result<Collector> {
        tracing::debug!("tag_set::SetPolicy::collect_static_tag\nTag = {:?}\n", tag);
        Ok(collector.set_default_parameters(&tag.parameters))
    }
}
