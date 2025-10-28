use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::execute::{Worker, Collector};
use super::tags::DynamicPolicy;
use crate::ast2::{Tag, Anchor, Position};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
enum AnswerStatus {
    #[default]
    JustCreated,
    Repeat,
    NeedProcessing,
    NeedInjection,
    Completed,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct AnswerState {
    pub status: AnswerStatus,
    pub reply: String,
}

pub struct AnswerPolicy;

impl DynamicPolicy for AnswerPolicy {
    type State = AnswerState;

    fn mono(
        worker: &Worker,
        collector: Collector,
        mut state: Self::State,
    ) -> Result<(bool, Collector, Option<Self::State>, Option<String>)> {
        tracing::debug!("AnswerPolicy::mono with state: {:?}", state);
        match state.status {
            AnswerStatus::JustCreated => {
                tracing::debug!("AnswerStatus::JustCreated");
                // Prepare the query
                state.status = AnswerStatus::NeedProcessing;
                state.reply = String::new();
                Ok((true, collector, Some(state), Some(String::new())))
            }
            AnswerStatus::NeedProcessing => {
                tracing::debug!("AnswerStatus::NeedProcessing");
                // Execute the model query
                let response = worker.call_model(&collector, vec![collector.context().clone()])?;
                state.reply = response;
                state.status = AnswerStatus::NeedInjection;
                Ok((true, collector, Some(state), None))
            }
            AnswerStatus::NeedInjection => {
                tracing::debug!("AnswerStatus::NeedInjection");
                // Inject the reply into the document
                let output = state.reply.clone();
                state.status = AnswerStatus::Completed;
                Ok((true, collector, Some(state), Some(output)))
            }
            AnswerStatus::Completed | AnswerStatus::Repeat => {
                tracing::debug!("AnswerStatus::Completed or AnswerStatus::Repeat");
                // Nothing to do
                Ok((false, collector, None, None))
            }
        }
    }
}
