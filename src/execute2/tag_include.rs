//! This module implements the `IncludePolicy` for the `@include` tag. The `@include`
//! tag is a static tag that allows for the inclusion of content from another context
//! file into the current execution flow. This mechanism supports modularity and
//! reusability of context definitions.
use super::{ExecuteError, Result};

use super::tags::{StaticPolicy, StaticPolicyMonoInput, StaticPolicyMonoResult};
use crate::ast2::JsonPlusEntity;

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
    fn mono(inputs: StaticPolicyMonoInput) -> Result<StaticPolicyMonoResult> {
        let (mut result, residual) = StaticPolicyMonoResult::from_inputs(inputs);
        let included_context_name = residual
            .arguments
            .arguments
            .get(0)
            .ok_or_else(|| ExecuteError::MissingParameter("include tag argument".to_string()))?
            .value
            .clone();
        let data = match residual.parameters.get("data") {
            Some(JsonPlusEntity::Object(data)) => Some(data),
            Some(_) => {
                return Err(ExecuteError::UnsupportedParameterValue("data".to_string()));
            }
            None => None,
        };
        result.collector =
            match residual
                .worker
                ._execute(result.collector, &included_context_name, 0, data)?
            {
                Some(collector) => collector,
                None => {
                    return Err(ExecuteError::Generic(
                        "Included context returned no collector".to_string(),
                    ));
                }
            };
        Ok(result)
    }
}
