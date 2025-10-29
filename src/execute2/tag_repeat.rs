use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::execute::{Collector, Worker};
use super::tag_answer::{AnswerState, AnswerStatus};
use super::tags::DynamicPolicy;

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
        mut collector: Collector,
        parameters: &Parameters,
        arguments: &Arguments,
        mut state: Self::State,
    ) -> Result<(
        bool,
        Collector,
        Option<Self::State>,
        Option<String>,
        Vec<(Range, String)>,
    )> {
        tracing::debug!("RepeatPolicy::mono with state: {:?}", state);
        match state.status {
            RepeatStatus::JustCreated => {
                tracing::debug!("RepeatPolicy::JustCreated");
                // Find anchor to repeat if any
                let patches = match collector.anchor_stack().last() {
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
                            let mut mutated_anchor = anchor.clone();
                            mutated_anchor.parameters = parameters.clone();
                            mutated_anchor.arguments = arguments.clone();
                            worker.mutate_anchor(&mutated_anchor)?
                        } else {
                            vec![]
                        }
                    }
                    None => {
                        return Err(anyhow::anyhow!("@repeat inside no anchor!?"));
                    }
                };
                // Prepare the query
                state.status = RepeatStatus::Completed;
                Ok((true, collector, Some(state), None, patches))
            }
            RepeatStatus::Completed => {
                tracing::debug!("RepeatPolicy::Completed");
                // Nothing to do
                Ok((false, collector, None, None, vec![]))
            }
        }
    }
}
