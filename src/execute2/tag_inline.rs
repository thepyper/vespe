
use super::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::content::{ModelContent, ModelContentItem};
use super::error::ExecuteError;
use super::execute::{Collector, Worker};
use super::tags::{DynamicPolicy, DynamicPolicyMonoResult};
use crate::ast2::{Arguments, JsonPlusEntity, JsonPlusObject, Parameters};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
pub enum InlineStatus {
    /// The `@answer` tag has just been created and needs to be processed.
    #[default]
    JustCreated,
    /// The `@answer` tag is in a state where it needs to re-execute the model call.
    Repeat,
    /// The `@answer` tag has completed its execution, and its response is in the document.
    Completed,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct InlineState {
    /// The current status of the `@answer` tag.
    pub status: InlineStatus,
}

/// Implements the dynamic policy for the `@answer` tag.
///
/// This policy defines how the `@answer` tag behaves during the execution
/// process, managing its state transitions and interactions with the external model.
pub struct InlinePolicy;

impl DynamicPolicy for InlinePolicy {
    /// The associated state for the `AnswerPolicy` is [`AnswerState`].
    type State = InlineState;

    /// Executes a single step of the `@answer` tag's lifecycle.
    ///
    /// This method handles the state transitions of an `@answer` tag:
    /// - `JustCreated`: Transitions to `NeedProcessing` and triggers a new pass.
    /// - `NeedProcessing`: Calls the external model, stores the response, transitions
    ///   to `NeedInjection`, and triggers a new pass.
    /// - `NeedInjection`: Prepares the model's reply for injection into the document,
    ///   transitions to `Completed`, and triggers a new pass.
    /// - `Completed`: No action, the tag is resolved.
    /// - `Repeat`: Resets the state to `NeedProcessing` and triggers a new pass to re-execute.
    ///
    /// # Arguments
    ///
    /// * `worker` - A reference to the [`Worker`] instance.
    /// * `collector` - The current [`Collector`] state.
    /// * `input` - The [`ModelContent`] collected so far, serving as input to the model.
    /// * `parameters` - The [`Parameters`] associated with the `@answer` tag.
    /// * `arguments` - The [`Arguments`] associated with the `@answer` tag.
    /// * `state` - The current [`AnswerState`] of the tag.
    /// * `readonly` - A boolean indicating if the current pass is read-only.
    ///
    /// # Returns
    ///
    /// A `Result` containing a [`DynamicPolicyMonoResult`] describing the outcome
    /// of this execution step.
    ///
    /// # Errors
    ///
    /// Returns an [`ExecuteError`] if the model call fails or if there are issues
    /// with parameters.
    fn mono(
        worker: &Worker,
        collector: Collector,
        input: &ModelContent,
        parameters: &Parameters,
        arguments: &Arguments,
        mut state: Self::State,
        readonly: bool,
    ) -> Result<DynamicPolicyMonoResult<Self::State>> {
        tracing::debug!(
            "tag_inline::InlinePolicy::mono\nState = {:?}\nreadonly = {}\n",
            state,
            readonly
        );
        let mut result = DynamicPolicyMonoResult::<Self::State>::new(collector);
        match state.status {
            InlineStatus::JustCreated => {
                let context_name = arguments.arguments.get(0).expect("TODO gestire errore!!").value.clone();                
                // Prepare the query
                state.status = InlineStatus::Completed;
                result.new_state = Some(state);
                result.new_output = Some(worker.read_context(&context_name)?);
                result.do_next_pass = true;
            }
            InlineStatus::Completed => {
                // Nothing to do
            }
            InlineStatus::Repeat => {
                // Prepare the query
                state.status = InlineStatus::JustCreated;
                result.new_state = Some(state);
                result.new_output = Some(String::new());
                result.do_next_pass = true;
            }
        }
        Ok(result)
    }
}

