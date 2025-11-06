//! This module implements the `AnswerPolicy` for the `@answer` tag, a dynamic tag
//! that orchestrates interaction with an external model (LLM). It defines the
//! state machine for the `@answer` tag's lifecycle, from initial creation to
//! processing the model's response and injecting it into the document.
use super::Result;
use serde::{Deserialize, Serialize};

use super::content::ModelContent;
use super::execute::{Collector, Worker};
use super::tags::{DynamicPolicy, DynamicPolicyMonoResult};
//use super::variables::Variables;
use crate::ast2::{Arguments, Parameters};

/// Represents the current status of an `@answer` tag's execution.
///
/// This enum defines the different stages a dynamic `@answer` tag goes through
/// during the multi-pass execution, from being initially created to having its
/// response completed and injected.
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
pub enum AnswerStatus {
    /// The `@answer` tag has just been created and needs to be processed.
    #[default]
    JustCreated,
    /// The `@answer` tag is in a state where it needs to re-execute the model call.
    Repeat,
    /// The `@answer` tag needs to call the external model to get a response.
    NeedProcessing,
    /// The model has responded, and its reply needs to be injected into the document.
    NeedInjection,
    /// The `@answer` tag has completed its execution, and its response is in the document.
    Completed,
}

/// Represents the persistent state of an `@answer` tag.
///
/// This struct holds the current status of the answer process and the model's
/// reply, allowing the execution engine to resume processing across multiple passes.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct AnswerState {
    /// The current status of the `@answer` tag.
    pub status: AnswerStatus,
    /// The reply received from the external model.
    pub reply: String,
}

/// Implements the dynamic policy for the `@answer` tag.
///
/// This policy defines how the `@answer` tag behaves during the execution
/// process, managing its state transitions and interactions with the external model.
pub struct AnswerPolicy;

impl DynamicPolicy for AnswerPolicy {
    /// The associated state for the `AnswerPolicy` is [`AnswerState`].
    type State = AnswerState;

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
            "tag_answer::AnswerPolicy::mono\nState = {:?}\nreadonly = {}\n",
            state,
            readonly
        );
        let mut result = DynamicPolicyMonoResult::<Self::State>::new(collector);
        match state.status {
            AnswerStatus::JustCreated => {
                // Prepare the query
                state.status = AnswerStatus::NeedProcessing;
                state.reply = String::new();
                result.new_state = Some(state);
                result.do_next_pass = true;
            }
            AnswerStatus::NeedProcessing => {
                // Execute the model query
                let prompt = worker.prefix_content_from_parameters(input.clone(), parameters)?;
                let prompt = worker.postfix_content_from_parameters(prompt, parameters)?;
                let response = worker.call_model(parameters, &prompt)?;
                state.reply = response;
                state.status = AnswerStatus::NeedInjection;
                result.new_state = Some(state);
                result.do_next_pass = true;
            }
            AnswerStatus::NeedInjection => {
                // Inject the reply into the document
                let output = state.reply.clone();
                state.status = AnswerStatus::Completed;
                result.new_state = Some(state);
                result.new_output = Some(output);
                result.do_next_pass = true;
            }
            AnswerStatus::Completed => {
                // Nothing to do
            }
            AnswerStatus::Repeat => {
                // Prepare the query
                state.status = AnswerStatus::NeedProcessing;
                state.reply = String::new();
                result.new_state = Some(state);
                result.new_output = Some(String::new());
                result.do_next_pass = true;
            }
        }
        Ok(result)
    }
}
