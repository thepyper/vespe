//! This module implements the `DonePolicy` for the `@done` tag.
//!
//! The `@done` tag is a static tag used in conjunction with the dynamic `@task` tag.
//! Its primary purpose is to signal the completion of a specific segment of work
//! within an active `@task` anchor. When an `@done` tag is encountered, it
//! instructs the most recently active `@task` to "eat" the content up to the
//! position of the `@done` tag.
//!
//! This mechanism allows for fine-grained control over the iterative processing
//! of content by a `@task`. By strategically placing `@done` tags, a user can
//! define the boundaries of content that a task should process in a single
//! "eating" step.
//!
//! # Interaction with `@task`
//!
//! When the `DonePolicy` processes an `@done` tag:
//! 1. It identifies the latest active `@task` anchor on the collector's stack.
//! 2. If the `@task` is in a `Waiting` state, the `@done` tag triggers a state
//!    change in the `@task` to `Eating`.
//! 3. The `eating_end` position of the `@task`'s state is updated to the
//!    beginning of the `@done` tag's range.
//! 4. The `@done` tag itself is then removed from the document, and a new
//!    execution pass is requested, allowing the `@task` to perform its
//!    "eating" operation.
//!
//! # Examples
//!
//! ```markdown
//! @task
//! This is the first part of the content.
//! @done
//! This is the second part of the content.
//! @done
//! @end task
//! ```
//!
//! In this example, the first `@done` tag would cause the `@task` to process
//! "This is the first part of the content." The second `@done` tag would then
//! cause it to process "This is the second part of the content."
use super::Result;

use super::tag_task::{TaskState, TaskStatus};
use super::tags::{StaticPolicy, StaticPolicyMonoInput, StaticPolicyMonoResult, TagOrAnchor};
use crate::ast2::CommandKind;

/// Implements the static policy for the `@done` tag.
///
/// This policy is responsible for handling the logic associated with the
/// `@done` tag, primarily by interacting with the state of an enclosing
/// `@task` anchor to control its content processing.
pub struct DonePolicy;

impl StaticPolicy for DonePolicy {
    /// Executes a single step of the `@done` tag's lifecycle.
    ///
    /// This method is invoked when an `@done` tag is processed. It attempts to
    /// find the most recently active `@task` anchor. If a `@task` anchor is found
    /// and is in the `Waiting` state, this method transitions its state to `Eating`,
    /// setting its `eating_end` to the beginning of the `@done` tag's position.
    /// This effectively tells the `@task` to consume all content up to this point.
    ///
    /// The `@done` tag itself is then removed from the document by generating a patch,
    /// and a new execution pass is requested to allow the `@task` to process the
    /// newly defined content segment.
    ///
    /// # Arguments
    ///
    /// * `inputs` - A `StaticPolicyMonoInput` struct containing all necessary context
    ///   for execution, including the current `Collector` state, `Worker` instance,
    ///   parameters, arguments, and the `TagOrAnchor` being processed.
    ///
    /// # Returns
    ///
    /// A `Result<StaticPolicyMonoResult>` indicating the outcome. On success, it
    /// contains `StaticPolicyMonoResult` with the generated patches and a flag
    /// `do_next_pass` set to `true` to trigger re-execution.
    ///
    /// # Panics
    ///
    /// * Panics if the `TagOrAnchor` provided in `inputs` is an `Anchor` instead of a `Tag`.
    ///   This should ideally not happen in the context of a static policy.
    /// * Panics if no active `@task` anchor is found on the collector's stack.
    /// * Panics if the latest anchor found is not of `CommandKind::Task`.
    ///
    /// # Errors
    ///
    /// Returns `ExecuteError` variants if state loading/saving for the `@task`
    /// anchor fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Assume a document structure like:
    /// // @task
    /// // Content for the first step.
    /// // @done
    /// // Content for the second step.
    /// // @end task
    ///
    /// // When `mono` is called for the first `@done` tag:
    /// // 1. It finds the enclosing `@task`.
    /// // 2. If the `@task` is `Waiting`, it sets its status to `Eating`
    /// //    and `eating_end` to the position of the `@done` tag.
    /// // 3. The `@done` tag is removed, and `do_next_pass` is set to true.
    /// // This causes the `@task` to process "Content for the first step."
    /// ```
    fn mono(inputs: StaticPolicyMonoInput) -> Result<StaticPolicyMonoResult> {
        let (mut result, residual) = StaticPolicyMonoResult::from_inputs(inputs);
        tracing::debug!("tag_done::DonePolicy: {:?}", residual);
        let tag = match residual.tag_or_anchor {
            TagOrAnchor::Tag(tag) => tag,
            _ => {
                panic!("!?!?!? cannot be anchor in static tag !?!?!?"); // better error TODO
            }
        };
        match result.collector.latest_task() {
            Some(anchor) => {
                match anchor.command {
                    CommandKind::Task => {
                        let mut task_state = residual
                            .worker
                            .load_state::<TaskState>(anchor.command, &anchor.uuid)?;
                        match task_state.status {
                            TaskStatus::Waiting => {
                                if !residual.readonly {
                                    task_state.status = TaskStatus::Eating;
                                    task_state.eating_end = tag.range.begin;
                                    residual.worker.save_state::<TaskState>(
                                        anchor.command,
                                        &anchor.uuid,
                                        &task_state,
                                        None,
                                    )?;
                                    result.new_patches = vec![(tag.range, String::new())];
                                }
                                result.do_next_pass = true;
                            }
                            _ => {}
                        }
                    }
                    _ => {
                        panic!("not a task anchor!=!=!="); // TODO better error
                    }
                }
            }
            None => {
                panic!("no previous task anchor!=!=!="); // TODO better error
            }
        }
        tracing::debug!("tag_done res {:?}", result);
        Ok(result)
    }
}
