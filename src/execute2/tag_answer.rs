use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::content::ModelContent;
use super::execute::{Collector, Worker};
use super::tags::{DynamicPolicy, DynamicPolicyMonoResult};
//use super::variables::Variables;
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
        collector: Collector,
        input: &ModelContent,
        parameters: &Parameters,
        arguments: &Arguments,
        mut state: Self::State,
        readonly: bool,
    ) -> Result<DynamicPolicyMonoResult<Self::State>> {
        tracing::debug!(
            "tag_answer::AnswerPolicy::mono\nState = {:?}\nreadonly = {}\n",
            state,
            readonly
        );
        let mut result = DynamicPolicyMonoResult::<Self::State>::new(collector);
        match state.status {
            AnswerStatus::JustCreated => {
                // Prepare the query
                state.status = AnswerStatus::NeedProcessing;
                state.reply = String::new();
                result.new_state = Some(state);
                result.do_next_pass = true;
            }
            AnswerStatus::NeedProcessing => {
                // Execute the model query
                let response = worker.call_model(parameters, input)?; 
                state.reply = response;
                state.status = AnswerStatus::NeedInjection;
                result.new_state = Some(state);
                result.do_next_pass = true;
            }
            AnswerStatus::NeedInjection => {
                // Inject the reply into the document
                let output = state.reply.clone();
                state.status = AnswerStatus::Completed;
                result.new_state = Some(state);
                result.new_output = Some(output);
                result.do_next_pass = true;
            }
            AnswerStatus::Completed => {
                // Nothing to do
            }
            AnswerStatus::Repeat => {
                // Prepare the query
                state.status = AnswerStatus::NeedProcessing;
                state.reply = String::new();
                result.new_state = Some(state);
                result.new_output = Some(String::new());
                result.do_next_pass = true;
            }
        }
        Ok(result)
    }
}
