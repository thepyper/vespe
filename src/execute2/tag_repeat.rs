use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::execute::{Collector, Worker};
use super::tag_answer::{AnswerState, AnswerStatus};
use super::tags::{DynamicPolicy, DynamicPolicyMonoResult};

use crate::ast2::{Anchor, Arguments, CommandKind, Parameters, Position, Range, Tag};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
enum RepeatStatus {
    #[default]
    JustCreated,
    Completed,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct RepeatState {
    pub status: RepeatStatus,
}

pub struct RepeatPolicy;

impl DynamicPolicy for RepeatPolicy {
    type State = RepeatState;

    fn mono(
        worker: &Worker,
        collector: Collector,
        parameters: &Parameters,
        arguments: &Arguments,
        mut state: Self::State,
        readonly: bool,
    ) -> Result<DynamicPolicyMonoResult<Self::State>> {
        tracing::debug!(
            "tag_repeat::RepeatPolicy::mono\nState = {:?}\nreadonly = {}\n",
            state,
            readonly
        );
        let mut result = DynamicPolicyMonoResult::<Self::State>::new(collector);
        match state.status {
            RepeatStatus::JustCreated => {
                // Find anchor to repeat if any
                let patches = match result.collector.anchor_stack().last() {
                    Some(anchor) => {
                        let is_anchor_repeatable = match anchor.command {
                            CommandKind::Answer => {
                                let mut answer_state = worker
                                    .load_state::<AnswerState>(anchor.command, &anchor.uuid)?;
                                answer_state.status = AnswerStatus::Repeat;
                                worker.save_state::<AnswerState>(
                                    anchor.command,
                                    &anchor.uuid,
                                    &answer_state,
                                    None,
                                )?;
                                true
                            }
                            _ => false,
                        };
                        if is_anchor_repeatable {
                            // Mutate anchor variables
                            let mutated_anchor = anchor.update(parameters, arguments);
                            result
                                .new_patches
                                .extend(worker.mutate_anchor(&mutated_anchor)?);
                        }
                    }
                    None => {
                        return Err(anyhow::anyhow!("@repeat inside no anchor!?"));
                    }
                };
                // Prepare the query
                state.status = RepeatStatus::Completed;
                result.new_state = Some(state);
                result.do_next_pass = true;
            }
            RepeatStatus::Completed => {
                // Nothing to do
            }
        }
        Ok(result)
    }
}
