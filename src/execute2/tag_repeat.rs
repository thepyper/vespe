//! This module implements the `RepeatPolicy` for the `@repeat` tag. The `@repeat`
//! tag is a dynamic tag designed to re-trigger the execution of a dynamic anchor
//! (such as `@answer`) that it encloses. This allows for iterative refinement
//! or regeneration of content based on previous model outputs or updated context.
use super::{ExecuteError, Result};
use serde::{Deserialize, Serialize};

use super::content::ModelContent;
use super::execute::{Collector, Worker};
use super::tag_answer::{AnswerState, AnswerStatus};
use super::tag_inline::{InlineState, InlineStatus};
use super::tags::{DynamicPolicy, DynamicPolicyMonoInput, DynamicPolicyMonoResult};

use crate::ast2::{Arguments, CommandKind, Parameters};

/// Represents the current status of an `@repeat` tag's execution.
///
/// This enum tracks the lifecycle of the `@repeat` tag, primarily to ensure
/// that its action (triggering a repeat) is performed once.
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
pub enum RepeatStatus {
    /// The `@repeat` tag has just been created and needs to perform its action.
    #[default]
    JustCreated,
    /// The `@repeat` tag has completed its action.
    Completed,
}

/// Represents the persistent state of an `@repeat` tag.
///
/// This struct holds the current status of the repeat process.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct RepeatState {
    /// The current status of the `@repeat` tag.
    pub status: RepeatStatus,
}

/// Implements the dynamic policy for the `@repeat` tag.
///
/// This policy defines how the `@repeat` tag behaves during the execution
/// process, primarily by modifying the state of an enclosing dynamic anchor
/// to force its re-execution.
pub struct RepeatPolicy;

impl DynamicPolicy for RepeatPolicy {
    /// The associated state for the `RepeatPolicy` is [`RepeatState`].
    type State = RepeatState;

    /// Executes a single step of the `@repeat` tag's lifecycle.
    ///
    /// When in the `JustCreated` state, this method attempts to find an enclosing
    /// dynamic anchor (e.g., an `@answer` anchor). If found and it's a repeatable
    /// type, it updates the state of that anchor to `Repeat` (for `@answer` tags)
    /// and applies patches to mutate the anchor's parameters if specified by `@repeat`.
    /// It then transitions its own state to `Completed` and triggers a new pass.
    ///
    /// # Arguments
    ///
    /// * `worker` - A reference to the [`Worker`] instance.
    /// * `collector` - The current [`Collector`] state.
    /// * `input` - The [`ModelContent`] collected so far (unused in this policy).
    /// * `parameters` - The [`Parameters`] associated with the `@repeat` tag.
    /// * `arguments` - The [`Arguments`] associated with the `@repeat` tag.
    /// * `state` - The current [`RepeatState`] of the tag.
    /// * `readonly` - A boolean indicating if the current pass is read-only.
    ///
    /// # Returns
    ///
    /// A `Result` containing a [`DynamicPolicyMonoResult`] describing the outcome
    /// of this execution step.
    ///
    /// # Errors
    ///
    /// Returns [`ExecuteError::Generic`] if `@repeat` is not used inside an anchor.
    /// Returns other [`ExecuteError`] variants if state loading/saving fails or
    /// anchor mutation fails.
    fn mono(
        mut inputs: DynamicPolicyMonoInput<Self::State>,
    ) -> Result<DynamicPolicyMonoResult<Self::State>> {
        let (mut result, mut residual) =
            DynamicPolicyMonoResult::<Self::State>::from_inputs(inputs);
        match residual.state.status {
            RepeatStatus::JustCreated => {
                // Find anchor to repeat if any
                match result.collector.anchor_stack().last() {
                    Some(anchor) => {
                        let is_anchor_repeatable = match anchor.command {
                            CommandKind::Answer => {
                                let mut answer_state = residual
                                    .worker
                                    .load_state::<AnswerState>(anchor.command, &anchor.uuid)?;
                                answer_state.status = AnswerStatus::Repeat;
                                residual.worker.save_state::<AnswerState>(
                                    anchor.command,
                                    &anchor.uuid,
                                    &answer_state,
                                    None,
                                )?;
                                true
                            }
                            CommandKind::Inline => {
                                let mut inline_state = residual
                                    .worker
                                    .load_state::<InlineState>(anchor.command, &anchor.uuid)?;
                                inline_state.status = InlineStatus::Repeat;
                                residual.worker.save_state::<InlineState>(
                                    anchor.command,
                                    &anchor.uuid,
                                    &inline_state,
                                    None,
                                )?;
                                true
                            }
                            _ => false,
                        };
                        if is_anchor_repeatable {
                            // Mutate anchor parameters
                            let mutated_anchor =
                                anchor.update(residual.parameters, residual.arguments);
                            result
                                .new_patches
                                .extend(residual.worker.mutate_anchor(&mutated_anchor)?);
                        }
                    }
                    None => {
                        return Err(ExecuteError::Generic(
                            "@repeat must be used inside an anchor".to_string(),
                        ));
                    }
                };
                // Prepare the query
                residual.state.status = RepeatStatus::Completed;
                result.new_state = Some(residual.state);
                result.do_next_pass = true;
            }
            RepeatStatus::Completed => {
                // Nothing to do
            }
        }
        Ok(result)
    }
}
