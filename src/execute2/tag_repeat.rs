//! This module implements the `RepeatPolicy` for the `@repeat` tag.
//!
//! The `@repeat` tag is a dynamic tag designed to re-trigger the execution of a dynamic anchor
//! (such as `@answer` or `@inline`) that it encloses. This allows for iterative refinement
//! or regeneration of content based on previous model outputs or updated context.
//!
//! When an `@repeat` tag is encountered within a dynamic anchor, it signals the system
//! to re-evaluate and re-execute that anchor. This is particularly useful in scenarios
//! where the output of an anchor needs to be refined multiple times, or when external
//! conditions change, necessitating a fresh generation of content.
//!
//! The `RepeatPolicy` handles the logic for identifying the enclosing anchor,
//! updating its state to `Repeat`, and applying any specified parameter mutations
//! to the anchor before triggering a new execution pass.
//!
//! # Examples
//!
//! Consider an `@answer` tag that generates a code snippet. If the initial snippet
//! is not satisfactory, an `@repeat` tag can be used to prompt the model to
//! regenerate it, potentially with modified parameters.
//!
//! ```markdown
//! @answer
//! Here is a code snippet:
//! ```rust
//! fn main() {
//!     println!("Hello, world!");
//! }
//! ```
//! @repeat
//! ```
//!
//! In this example, the `@repeat` tag would cause the `@answer` block to be
//! re-executed, generating a new code snippet.
//!
//! Similarly, with `@inline` tags:
//!
//! ```markdown
//! @inline
//! Initial thought.
//! @repeat
//! ```
//!
//! This would cause the `@inline` block to be re-evaluated.
use super::{ExecuteError, Result};

use super::tag_answer::{AnswerState, AnswerStatus};
use super::tag_inline::{InlineState, InlineStatus};
use super::tags::{Container, StaticPolicy, StaticPolicyMonoInput, StaticPolicyMonoResult};

use crate::ast2::CommandKind;

/// Implements the static policy for the `@repeat` tag.
///
/// This policy defines how the `@repeat` tag behaves during the execution
/// process. Its primary function is to identify an enclosing dynamic anchor
/// (such as `@answer` or `@inline`) and modify its state to trigger a re-execution.
///
/// The `RepeatPolicy` is stateless in itself, as its actions are focused on
/// manipulating the state of other dynamic anchors.
pub struct RepeatPolicy;

impl StaticPolicy for RepeatPolicy {
    /// Executes a single step of the `@repeat` tag's lifecycle.
    ///
    /// This method is invoked when the `@repeat` tag is processed. It identifies
    /// the nearest enclosing dynamic anchor (e.g., `@answer` or `@inline`) in the
    /// `anchor_stack`. If a repeatable anchor is found, its internal state is
    /// updated to `Repeat`, signaling that it should be re-executed in a subsequent
    /// pass. Additionally, if the `@repeat` tag carries any parameters or arguments,
    /// these are used to mutate the enclosing anchor's parameters, allowing for
    /// dynamic adjustments during re-execution.
    ///
    /// After successfully marking the anchor for repetition and applying mutations,
    /// the `mono` function generates patches to update the document: one to replace
    /// the original anchor with its mutated version, and another to elide the
    /// `@repeat` tag itself from the output. It then requests a new execution pass.
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
    /// Panics if the `TagOrAnchor` provided in `inputs` is an `Anchor` instead of a `Tag`.
    /// This should ideally not happen in the context of a static policy.
    ///
    /// # Errors
    ///
    /// * `ExecuteError::Generic` if the `@repeat` tag is not found within any
    ///   enclosing dynamic anchor.
    /// * `ExecuteError` variants related to state loading/saving or anchor mutation
    ///   failures if the underlying `Worker` operations encounter issues.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Assume a context where an @answer tag is active:
    /// // @answer
    /// // This is some generated content.
    /// // @repeat
    ///
    /// // When `mono` is called for the @repeat tag, it would:
    /// // 1. Identify the @answer tag as the enclosing anchor.
    /// // 2. Set the @answer tag's status to `AnswerStatus::Repeat`.
    /// // 3. Generate patches to update the @answer tag and remove the @repeat tag.
    /// // 4. Set `do_next_pass` to true to re-execute the @answer tag.
    /// ```
    fn mono(inputs: StaticPolicyMonoInput) -> Result<StaticPolicyMonoResult> {
        let (mut result, residual) = StaticPolicyMonoResult::from_inputs(inputs);
        let tag = match residual.container {
            Container::Tag(tag) => tag,
            _ => {
                panic!("!?!?!? cannot be anchor in static tag !?!?!?"); // better error TODO
            }
        };
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
                if !is_anchor_repeatable {
                    return Err(ExecuteError::Generic(
                        "@repeat must be used inside a repeatable anchor".to_string(),
                    ));
                }
                if !residual.readonly {
                    // Mutate anchor parameters
                    let mutated_anchor = anchor.update(residual.parameters, residual.arguments);
                    // Patch mutated anchor
                    let mutated_anchor_patch = residual.worker.mutate_anchor(&mutated_anchor)?;
                    let elide_repeat_patch = (tag.range, String::new());
                    result.new_patches = vec![mutated_anchor_patch, elide_repeat_patch];
                }
                result.do_next_pass = true;
            }
            None => {
                return Err(ExecuteError::Generic(
                    "@repeat must be used inside an anchor".to_string(),
                ));
            }
        };
        // Prepare the query
        Ok(result)
    }
}
