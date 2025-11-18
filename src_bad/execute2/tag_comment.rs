//! Implements the behavior for the `@comment` tag.
//!
//! The `@comment` tag allows developers to write comments within the context files
//! that are completely ignored by the execution engine. This is useful for adding
//! notes, explanations, or reminders that should not be part of the final model
//! content.

use super::Result;

use super::tags::{StaticPolicy, StaticPolicyMonoInput, StaticPolicyMonoResult};

/// Implements the static policy for the `@comment` tag.
///
/// This policy ensures that any `@comment` tag encountered during processing
/// is simply ignored, having no effect on the `Collector` or the final output.
pub struct CommentPolicy;

impl StaticPolicy for CommentPolicy {
    /// Handles the `@comment` tag by performing no action.
    ///
    /// This function receives the collector and the tag, and returns the collector
    /// unmodified. This effectively makes the tag a no-op.
    ///
    /// # Arguments
    ///
    /// * `_worker` - A reference to the [`Worker`], unused in this policy.
    /// * `collector` - The current [`Collector`] state.
    /// * `_tag` - The `@comment` [`Tag`] being processed, unused.
    ///
    /// # Returns
    ///
    /// The original, unmodified `Collector`.
    fn mono(inputs: StaticPolicyMonoInput) -> Result<StaticPolicyMonoResult> {
        // The @comment tag does nothing, so we just return the collector as is.
        let (result, _residual) = StaticPolicyMonoResult::from_inputs(inputs);
        Ok(result)
    }
}
