//! Implements the behavior for the dynamic `@task` tag.
//!
//! The `@task` tag is a dynamic anchor used to manage and track the progress
//! of an ongoing task within the execution flow. It allows for iterative
//! processing of content, where the task can transition through different
//! states such as `JustCreated`, `Waiting`, and `Eating`.
//!
//! This tag is particularly useful for scenarios where content needs to be
//! consumed or generated incrementally, or when a multi-step process needs
//! to be managed and its state preserved across execution passes. The `@task`
//! tag can "eat" portions of the document, effectively processing them
//! and updating its internal state.
//!
//! # Examples
//!
//! A typical use case for `@task` might involve processing a long document
//! in chunks, or managing a conversation where the agent needs to perform
//! actions and then wait for further input.
//!
//! ```markdown
//! @task
//! This is the content to be processed by the task.
//! It can be consumed in multiple steps.
//! @end task
//! ```
//!
//! The `TaskPolicy` defines how the `@task` tag transitions between its
//! states and how it interacts with the document content.
use serde::{Deserialize, Serialize};

use super::content::ModelContentItem;
use super::execute::Worker;
use super::tags::{DynamicPolicy, DynamicPolicyMonoInput, DynamicPolicyMonoResult, TagOrAnchor};
use super::Result;
use crate::ast2::{Position, Range};

/// Represents the execution status of an `@task` tag.
///
/// The status dictates the current phase of the task's lifecycle, influencing
/// how the `TaskPolicy` processes the tag during execution passes.
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
pub enum TaskStatus {
    /// The initial state of a newly created `@task` anchor.
    ///
    /// In this state, the task has just been recognized and is awaiting its
    /// first processing step. It typically transitions to `Waiting` after
    /// initial setup.
    #[default]
    JustCreated,
    /// The task is currently waiting for external input or for a new execution pass.
    ///
    /// In this state, the task is not actively processing content but is
    /// holding its position, ready to resume work when conditions are met.
    Waiting,
    /// The task is actively "eating" or processing a portion of the document.
    ///
    /// This state indicates that the task is consuming content from the document,
    /// typically to update its internal state or generate new output.
    Eating,
}

/// Holds the persistent state for an `@task` anchor.
///
/// This struct stores the necessary information to track the progress and
/// current status of an `@task` throughout multiple execution passes.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct TaskState {
    /// The current status of the `@task` anchor.
    ///
    /// This field indicates the current phase of the task's lifecycle,
    /// such as `JustCreated`, `Waiting`, or `Eating`.
    pub status: TaskStatus,
    /// The ending position in the document that the task aims to "eat" up to
    /// in the current `Eating` step.
    ///
    /// This is used to define the boundary of the content being processed
    /// during an `Eating` phase.
    pub eating_end: Position,
}

/// Implements the dynamic policy for the `@task` tag.
///
/// This policy defines the behavior and state transitions for `@task` anchors.
/// It manages how tasks are created, wait for processing, and "eat" content
/// from the document, driving the iterative execution process.
pub struct TaskPolicy;

impl DynamicPolicy for TaskPolicy {
    /// The state object associated with this policy.
    type State = TaskState;

    /// Executes a single step of the `@task` tag's lifecycle based on its current state.
    ///
    /// This method is the core of the `TaskPolicy`, managing the state transitions
    /// and actions performed by an `@task` anchor. It reacts to the `TaskStatus`
    /// (`JustCreated`, `Waiting`, `Eating`) to either initialize the task,
    /// pause its processing, or actively consume content from the document.
    ///
    /// - In the `JustCreated` state, the task transitions to `Waiting` and signals
    ///   for a new pass.
    /// - In the `Waiting` state, it primarily acts as a placeholder, adding a
    ///   system item to the collector.
    /// - In the `Eating` state, it consumes a defined portion of the document,
    ///   updates the task's output, and transitions back to `Waiting`, also
    ///   requesting a new pass.
    ///
    /// # Arguments
    ///
    /// * `inputs` - A `DynamicPolicyMonoInput` struct containing the current
    ///   state of the task (`Self::State`), the `Collector`, `Worker`, and
    ///   other contextual information.
    ///
    /// # Returns
    ///
    /// A `Result<DynamicPolicyMonoResult<Self::State>>` indicating the outcome
    /// of the execution step. This includes any new state, output, patches,
    /// and whether a new pass is required.
    ///
    /// # Panics
    ///
    /// Panics if `tag_or_anchor` in `residual` is not an `Anchor` when in the
    /// `Eating` state. This indicates an internal inconsistency where a task
    /// is trying to "eat" without being properly represented as an anchor.
    ///
    /// # Errors
    ///
    /// Returns `ExecuteError` if there are issues retrieving content from the
    /// document using `Worker::get_range`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Assume an @task tag is present in the document:
    /// // @task
    /// // Some content to process.
    /// // @end task
    ///
    /// // When `mono` is called for a task in `JustCreated` state:
    /// // - It transitions the task to `Waiting`.
    /// // - Sets `do_next_pass` to true.
    ///
    /// // When `mono` is called for a task in `Eating` state:
    /// // - It "eats" a portion of the document defined by `eating_end`.
    /// // - Generates a patch to remove the eaten content.
    /// // - Updates the task's output with the eaten content.
    /// // - Transitions the task back to `Waiting`.
    /// // - Sets `do_next_pass` to true.
    /// ```
    fn mono(
        inputs: DynamicPolicyMonoInput<Self::State>,
    ) -> Result<DynamicPolicyMonoResult<Self::State>> {
        tracing::debug!("tag_task::TaskPolicy::mono\nState = {:?}", inputs.state);
        let (mut result, mut residual) =
            DynamicPolicyMonoResult::<Self::State>::from_inputs(inputs);
        match residual.state.status {
            TaskStatus::JustCreated => {
                if !residual.readonly {
                    // Load content from the specified context
                    residual.state.status = TaskStatus::Waiting;
                    result.new_state = Some(residual.state);
                    result.new_output = Some(String::new());
                }
                result.do_next_pass = true;
            }
            TaskStatus::Waiting => {
                // Nothing to do
                result.collector = result
                    .collector
                    .push_item(ModelContentItem::merge_downstream(super::TASK_ANCHOR_PLACEHOLDER));
            }
            TaskStatus::Eating => {
                // Eat a piece of text
                if !residual.readonly {
                    let (existing_output, eaten_output) = match residual.tag_or_anchor {
                        TagOrAnchor::Anchor((a0, a1)) => (
                            Range {
                                begin: a0.range.end,
                                end: a1.range.begin,
                            },
                            Range {
                                begin: a1.range.end,
                                end: residual.state.eating_end,
                            },
                        ),
                        _ => {
                            panic!("tag!?!?!?");
                        }
                    };
                    result.new_patches = vec![(eaten_output, String::new())];
                    let existing_output = Worker::get_range(residual.document, &existing_output)?;
                    let eaten_output = Worker::get_range(residual.document, &eaten_output)?;
                    result.new_output = Some(format!("{}{}", existing_output, eaten_output));
                    result.do_next_pass = true;
                    residual.state.status = TaskStatus::Waiting;
                    result.new_state = Some(residual.state);
                }
                result.do_next_pass = true;
            }
        }
        Ok(result)
    }
}
