//! This module implements the `SetPolicy` for the `@set` tag. The `@set` tag
//! is a static tag that allows for setting default parameters within the execution
//! context. These parameters can then be inherited or overridden by subsequent
//! tags and anchors, providing a way to configure behavior globally or locally.
use super::Result;

use super::execute::{Collector, Worker};
use super::tags::{
    StaticPolicy, StaticPolicyMonoInput, StaticPolicyMonoInputResidual, StaticPolicyMonoResult,
};
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
    fn mono(inputs: StaticPolicyMonoInput) -> Result<StaticPolicyMonoResult> {
        let (mut result, residual) = StaticPolicyMonoResult::from_inputs(inputs);
        result.collector = result.collector.set_default_parameters(residual.parameters);
        Ok(result)
    }
}
