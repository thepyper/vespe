//! Implements the behavior for the dynamic `@inline` tag.
//!
//! The `@inline` tag is used to dynamically include content from another context file.
//! Unlike the static `@include` tag, `@inline` is stateful, allowing its content
//! to be re-evaluated or "repeated" during the execution flow. This is useful when
//! the inlined content needs to be refreshed after other operations have modified
//! the execution state.

use serde::{Deserialize, Serialize};

use super::content::ModelContentItem;
use super::execute::Worker;
use super::tags::{DynamicPolicy, DynamicPolicyMonoInput, DynamicPolicyMonoResult, TagOrAnchor};
use super::Result;
use crate::ast2::{Position, Range};

/// Represents the execution status of an `@task` tag.
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
pub enum TaskStatus {
    // TODO doc
    #[default]
    JustCreated,
    // TODO doc
    Waiting,
    // TODO doc
    Eating,
}

/// Holds the persistent state for an `@task` anchor.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct TaskState {
    /// The current status of the `@task` anchor.
    pub status: TaskStatus,
    /// The ending position to reach in next Eating step
    pub eating_end: Position,
}

/// Implements the dynamic policy for the `@task` tag.
///
// TODO doc
pub struct TaskPolicy;

impl DynamicPolicy for TaskPolicy {
    /// The state object associated with this policy.
    type State = TaskState;

    /// Executes a single step of the `@task` tag's lifecycle.
    ///
    // TODO doc
    fn mono(
        inputs: DynamicPolicyMonoInput<Self::State>,
    ) -> Result<DynamicPolicyMonoResult<Self::State>> {
        tracing::debug!("tag_task::TaskPolicy::mono\nState = {:?}", inputs.state);
        let (mut result, mut residual) =
            DynamicPolicyMonoResult::<Self::State>::from_inputs(inputs);
        match residual.state.status {
            TaskStatus::JustCreated => {
                // Load content from the specified context
                residual.state.status = TaskStatus::Waiting;
                result.new_state = Some(residual.state);
                result.new_output = Some(String::new());
                result.do_next_pass = true;
            }
            TaskStatus::Waiting => {
                // Nothing to do
                result.collector = result
                    .collector
                    .push_item(ModelContentItem::system(super::TASK_ANCHOR_PLACEHOLDER));
            }
            TaskStatus::Eating => {
                // Eat a piece of text
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
                result.do_next_pass = true;
            }
        }
        Ok(result)
    }
}
