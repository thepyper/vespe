//! Implements the behavior for the dynamic `@inline` tag.
//!
//! The `@inline` tag is used to dynamically include content from another context file.
//! Unlike the static `@include` tag, `@inline` is stateful, allowing its content
//! to be re-evaluated or "repeated" during the execution flow. This is useful when
//! the inlined content needs to be refreshed after other operations have modified
//! the execution state.

use serde::{Deserialize, Serialize};

use super::content::{ModelContent, ModelContentItem};
use super::error::ExecuteError;
use super::execute::{Collector, Worker};
use super::tags::{DynamicPolicy, DynamicPolicyMonoResult, DynamicPolicyMonoInput};
use super::Result;
use crate::ast2::{Arguments, JsonPlusEntity, Parameters, Range};

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
    /// The range to reach in next Eating step
    pub eating_range: Range,
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
        mut inputs: DynamicPolicyMonoInput<Self::State>,
    ) -> Result<DynamicPolicyMonoResult<Self::State>> {
        tracing::debug!("tag_task::TaskPolicy::mono\nState = {:?}", inputs.state);
        let mut result = DynamicPolicyMonoResult::<Self::State>::new(inputs.collector);
        match inputs.state.status {
            TaskStatus::JustCreated => {
                // Load content from the specified context
                inputs.state.status = TaskStatus::Waiting;
                result.new_state = Some(inputs.state);
                result.new_output = Some(String::new());
                result.do_next_pass = true;
            }
            TaskStatus::Waiting => {
                // Nothing to do
                result.collector = result.collector.push_item(ModelContentItem::system(super::TASK_ANCHOR_PLACEHOLDER));
            }
            TaskStatus::Eating => {
                // Eat a piece of text
                // TODO
            }
        }
        Ok(result)
    }
}
