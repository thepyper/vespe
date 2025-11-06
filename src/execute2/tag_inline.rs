//! Implements the behavior for the dynamic `@inline` tag.
//!
//! The `@inline` tag is used to dynamically include content from another context file.
//! Unlike the static `@include` tag, `@inline` is stateful, allowing its content
//! to be re-evaluated or "repeated" during the execution flow. This is useful when
//! the inlined content needs to be refreshed after other operations have modified
//! the execution state.

use serde::{Deserialize, Serialize};

use super::content::ModelContent;
use super::error::ExecuteError;
use super::execute::{Collector, Worker};
use super::tags::{DynamicPolicy, DynamicPolicyMonoResult};
use super::Result;
use crate::ast2::{Arguments, Parameters};

/// Represents the execution status of an `@inline` tag.
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
pub enum InlineStatus {
    /// The `@inline` tag has just been converted into an anchor and its content
    /// needs to be loaded for the first time.
    #[default]
    JustCreated,
    /// The `@inline` tag is in a state where it needs to be re-executed, forcing
    /// a reload of its content from the source file.
    Repeat,
    /// The `@inline` tag has successfully loaded its content, and no further
    /// action is needed unless its state is changed to `Repeat`.
    Completed,
}

/// Holds the persistent state for an `@inline` anchor.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct InlineState {
    /// The current status of the `@inline` anchor.
    pub status: InlineStatus,
}

/// Implements the dynamic policy for the `@inline` tag.
///
/// This policy defines how an `@inline` tag and its corresponding anchor behave
/// during the execution process, managing state transitions to allow for dynamic

/// content loading and refreshing.
pub struct InlinePolicy;

impl DynamicPolicy for InlinePolicy {
    /// The state object associated with this policy.
    type State = InlineState;

    /// Executes a single step of the `@inline` tag's lifecycle.
    ///
    /// This method handles the state transitions for an `@inline` anchor:
    /// - `JustCreated`: Reads the content from the file specified in the tag's
    ///   arguments, sets it as the new output, and transitions the state to `Completed`.
    ///   It triggers a new pass to process the injected content.
    /// - `Completed`: No action is taken, as the content is already present.
    /// - `Repeat`: Resets the state to `JustCreated` and clears the existing content,
    ///   triggering a new pass to force a reload of the content from the source file.
    ///
    /// # Arguments
    ///
    /// * `worker` - A reference to the [`Worker`] instance.
    /// * `collector` - The current [`Collector`] state.
    /// * `_input` - The [`ModelContent`] collected so far (unused in this policy).
    /// * `_parameters` - The [`Parameters`] associated with the tag (unused).
    /// * `arguments` - The [`Arguments`] containing the path to the context file to inline.
    /// * `state` - The current [`InlineState`] of the anchor.
    /// * `_readonly` - A boolean indicating if the current pass is read-only (unused).
    ///
    /// # Returns
    ///
    /// A `Result` containing a [`DynamicPolicyMonoResult`] describing the outcome
    /// of this execution step, including any new state or output.
    ///
    /// # Errors
    ///
    /// Returns an [`ExecuteError`] if the context name is missing from the arguments
    /// or if the file cannot be read.
    fn mono(
        worker: &Worker,
        collector: Collector,
        _input: &ModelContent,
        _parameters: &Parameters,
        arguments: &Arguments,
        mut state: Self::State,
        _readonly: bool,
    ) -> Result<DynamicPolicyMonoResult<Self::State>> {
        tracing::debug!("tag_inline::InlinePolicy::mono\nState = {:?}", state,);
        let mut result = DynamicPolicyMonoResult::<Self::State>::new(collector);
        match state.status {
            InlineStatus::JustCreated => {
                let context_name = arguments
                    .arguments
                    .get(0)
                    .ok_or_else(|| ExecuteError::MissingParameter("context_name".to_string()))?
                    .value
                    .clone();

                // Load content from the specified context
                state.status = InlineStatus::Completed;
                result.new_state = Some(state);
                result.new_output = Some(worker.read_context(&context_name)?);
                result.do_next_pass = true;
            }
            InlineStatus::Completed => {
                // Nothing to do
            }
            InlineStatus::Repeat => {
                // Reset state to force a reload in the next pass
                state.status = InlineStatus::JustCreated;
                result.new_state = Some(state);
                result.new_output = Some(String::new());
                result.do_next_pass = true;
            }
        }
        Ok(result)
    }
}
