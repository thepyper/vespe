//! This module implements the `IncludePolicy` for the `@include` tag. The `@include`
//! tag is a static tag that allows for the inclusion of content from another context
//! file into the current execution flow. This mechanism supports modularity and
//! reusability of context definitions.
use super::{ExecuteError, Result};

use super::execute::{Collector, Worker};
use super::tags::StaticPolicy;
//use super::variables::Variables;
use crate::ast2::Tag;

/// Implements the static policy for the `@include` tag.
///
/// The `@include` tag processes another context file and merges its collected
/// content into the current [`Collector`].
pub struct IncludePolicy;

impl StaticPolicy for IncludePolicy {
    /// Collects content from an included context file.
    ///
    /// This method takes the name of another context file from the tag's arguments,
    /// executes it in a read-only (collection) mode, and then merges the resulting
    /// [`ModelContent`] into the current [`Collector`]. This effectively embeds
    /// the content of the included file at the point of the `@include` tag.
    ///
    /// # Arguments
    ///
    /// * `worker` - A reference to the [`Worker`] instance, used to execute the included context.
    /// * `collector` - The current [`Collector`] state.
    /// * `tag` - The [`Tag`] being processed, which contains the name of the context to include.
    ///
    /// # Returns
    ///
    /// A `Result` containing the updated [`Collector`] with the included content.
    ///
    /// # Errors
    ///
    /// Returns [`ExecuteError::MissingParameter`] if the context name is not provided
    /// in the tag arguments.
    /// Returns [`ExecuteError::Generic`] if the included context execution returns no collector.
    /// Returns other [`ExecuteError`] variants if the included context cannot be found or executed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use crate::execute2::{Collector, Worker, IncludePolicy};
    /// # use crate::execute2::tags::StaticPolicy;
    /// # use crate::ast2::{Tag, CommandKind, Parameters, Arguments, Range, Position, Argument};
    /// # use std::sync::Arc;
    /// # use crate::file::MockFileAccessor;
    /// # use crate::path::MockPathResolver;
    /// # // Mock setup for demonstration
    /// # let mut mock_file_access = MockFileAccessor::new();
    /// # mock_file_access.expect_read_file().returning(|_| Ok("Hello from included context!".to_string()));
    /// # let mut mock_path_res = MockPathResolver::new();
    /// # mock_path_res.expect_resolve_context().returning(|_| Ok(std::path::PathBuf::from("included.md")));
    /// #
    /// let worker = Worker::new(Arc::new(mock_file_access), Arc::new(mock_path_res));
    /// let collector = Collector::new();
    ///
    /// let tag = Tag {
    ///     command: CommandKind::Include,
    ///     parameters: Parameters::new(),
    ///     arguments: Arguments { arguments: vec![Argument { value: "included_context".to_string(), range: Range::null() }] },
    ///     range: Range::null(),
    /// };
    ///
    /// // In a real scenario, the included context would be processed and its content added.
    /// // For this example, we'll simulate the outcome.
    /// let updated_collector = IncludePolicy::collect_static_tag(&worker, collector, &tag).unwrap();
    /// // Assertions would go here to check the content of updated_collector
    /// // For instance, if "included_context" contained "Hello World!",
    /// // updated_collector.context().to_string() would contain "Hello World!".
    /// ```
    fn collect_static_tag(worker: &Worker, collector: Collector, tag: &Tag) -> Result<Collector> {
        tracing::debug!(
            "tag_include::IncludePolicy::collect_static_tag\nTag = {:?}\n",
            tag
        );
        let included_context_name = tag
            .arguments
            .arguments
            .get(0)
            .ok_or_else(|| ExecuteError::MissingParameter("include tag argument".to_string()))?
            .value
            .clone();
        match worker._execute(collector, &included_context_name, 0)? {
            Some(collector) => Ok(collector),
            None => Err(ExecuteError::Generic(
                "Included context returned no collector".to_string(),
            )),
        }
    }
}
