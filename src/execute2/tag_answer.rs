use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::execute::{Collector, Worker};
use super::tags::DynamicPolicy;
use crate::ast2::{Anchor, Arguments, Parameters, Position, Range, Tag};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
pub enum AnswerStatus {
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
        tracing::debug!("AnswerPolicy::mono with state: {:?}", state);
        match state.status {
            AnswerStatus::JustCreated => {
                tracing::debug!("AnswerStatus::JustCreated");
                // Prepare the query
                state.status = AnswerStatus::NeedProcessing;
                state.reply = String::new();
                Ok((true, collector, Some(state), None, vec![]))
            }
            AnswerStatus::NeedProcessing => {
                tracing::debug!("AnswerStatus::NeedProcessing");
                // Execute the model query
                collector = collector.update(parameters);
                let response = worker.call_model(&collector, vec![collector.context().clone()])?;
                state.reply = response;
                state.status = AnswerStatus::NeedInjection;
                Ok((true, collector, Some(state), None, vec![]))
            }
            AnswerStatus::NeedInjection => {
                tracing::debug!("AnswerStatus::NeedInjection");
                // Inject the reply into the document
                let output = state.reply.clone();
                state.status = AnswerStatus::Completed;
                Ok((true, collector, Some(state), Some(output), vec![]))
            }
            AnswerStatus::Completed => {
                tracing::debug!("AnswerStatus::Completed");
                // Nothing to do
                Ok((false, collector, None, None, vec![]))
            }
            AnswerStatus::Repeat => {
                tracing::debug!("AnswerStatus::Repeat");
                // Prepare the query
                state.status = AnswerStatus::NeedProcessing;
                state.reply = String::new();
                Ok((true, collector, Some(state), Some(String::new()), vec![]))
            }
        }
    }
}
