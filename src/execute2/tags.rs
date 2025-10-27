use anyhow::Result;
use uuid::Uuid;

use super::execute::Collector;
use super::execute::Worker;
use super::variables::Variables;
use super::content::ModelContent;

use crate::ast2::{Anchor, Position, Range, Tag};

//#[enum_dispatch]
pub trait TagBehavior {
    fn execute_tag(
        worker: &Worker,
        collector: Collector,
        tag: &Tag,
    ) -> Result<(Option<Collector>, Vec<(Range, String)>)>;
    fn collect_tag(
        worker: &Worker,
        collector: Collector,
        tag: &Tag,
    ) -> Result<Option<Collector>>;
    fn execute_anchor(
        worker: &Worker,
        collector: Collector,
        anchor: &Anchor,
        anchor_end: Position,
    ) -> Result<(Option<Collector>, Vec<(Range, String)>)>;
    fn collect_anchor(
        worker: &Worker,
        collector: Collector,
        anchor: &Anchor,
        anchor_end: Position,
    ) -> Result<Option<Collector>>;
}

trait StaticTagBehaviorTrait {
    fn collect_static_tag(
        worker: &Worker,
        collector: Collector,
        tag: &Tag,
    ) -> Result<Option<Collector>>;
}

struct<T> StaticTagBehavior<T> : StaticTagBehaviorTrait;

impl TagBehavior for StaticTagBehavior<T> {
    fn execute_anchor(
        worker: &Worker,
        mut collector: Collector,
        anchor: &Anchor,
        anchor_end: Position,
    ) -> Result<(Option<Collector>, Vec<(Range, String)>)> {
        panic!("StaticTag does not support execute_anchor");
    }
    fn collect_anchor(
        worker: &Worker,
        mut collector: Collector,
        anchor: &Anchor,
        anchor_end: Position,
    ) -> Result<Option<Collector>> {
        panic!("StaticTag does not support collect_anchor");
    }
    fn execute_tag(
        worker: &Worker,
        mut collector: Collector,
        tag: &Tag,
    ) -> Result<(Option<Collector>, Vec<(Range, String)>)> {
        let collector = T::collect_static_tag(worker, collector, tag);
        Ok((collector, vec![]))
    }
    fn collect_tag(
        worker: &Worker,
        collector: Collector,
        tag: &Tag,
    ) -> Result<Option<Collector>>; {
        T::collect_static_tag(worker, collector, tag)
    }
}

trait DynamicTagBehaviorTrait {
    type State;

    fn execute(
        worker: &Worker,
        mut collector: Collector,
        mut state: Self::State,
    ) -> Result<(Option<Collector>, Option<Self::State>, Option<String>)> {
        Self::mono(worker, collector, state)
    }
    fn collect(
        worker: &Worker,
        mut collector: Collector,
        mut state: Self::State,
    ) -> Result<(Option<Collector>, Option<Self::State>)> {
        let (collector, new_state, new_output) = Self::mono(worker, collector, state)?;
        match new_output {
            Some(_) => {
                // Cannot produce output during collect, new state discarded
                Ok((collector, None))
            }
            None => {
                // No new output produced, save new state
                Ok((collector, new_state))
            }
        }
    }
    fn mono(
        worker: &Worker,
        collector: Collector,
        state: Self::State,
    ) -> Result<(Option<Collector>, Option<Self::State>, Option<String>)>;
}

struct<T> DynamicTagBehavior<T> : DynamicTagBehaviorTrait;

impl TagBehavior for DynamicTagBehavior<T> {

    fn execute_tag(
        worker: &Worker,
        mut collector: Collector,
        tag: &Tag,
    ) -> Result<(Option<Collector>, Vec<(Range, String)>)> {
        let state: Self::State = Self::State::default();
        let (collector, new_state, new_output) = state.execute(worker, collector, state);
        // If there is a new state, save it
        match new_state {
            Some(new_state) => {
                worker.save_state(new_state, &tag.command.to_string(), &Uuid::new_v4())?;
            }
            None => {}
        }
        // If there is some output, patch into new anchor
        let patches = match new_output {
            Some(output) => worker.patch_tag_to_anchor(tag, &output)?,
            None => vec![],
        };
        // Return collector and patches
        Ok((collector, patches))
    }
    fn collect_tag(
        worker: &Worker,
        mut collector: Collector,
        tag: &Tag,
    ) -> Result<Option<Collector>> {
        // Dynamic tags do not support collect_tag because they always produce a new anchor during execution
        None
    }
    fn execute_anchor(
        worker: &Worker,
        mut collector: Collector,
        anchor: &Anchor,
        anchor_end: Position,
    ) -> Result<(Option<Collector>, Vec<(Range, String)>)> {
        let state = worker.load_state::<Self::State>(&anchor.command, &anchor.uuid)?;
        let (collector, new_state, new_output) = state.execute(worker, collector, state);
        // If there is a new state, save it
        match new_state {
            Some(new_state) => {
                worker.save_state(new_state, &anchor.command.to_string(), &anchor.uuid)?;
            }
            None => {}
        }
        // If there is some output, patch into new anchor
        let patches = match new_output {
            Some(output) => worker.patch_into_anchor(anchor, anchor_end, &output)?,
            None => vec![],
        };
        // Return collector and patches
        Ok((collector, patches))
    }
    fn collect_anchor(
        worker: &Worker,
        mut collector: Collector,
        anchor: &Anchor,
        anchor_end: Position,
    ) -> Result<Option<Collector>> {
        let state = worker.load_state::<Self::State>(&anchor.command, &anchor.uuid)?;
        let (collector, new_state) = state.collect(worker, collector, state);
        // If there is a new state, save it
        match new_state {
            Some(new_state) => {
                worker.save_state(new_state, &anchor.command.to_string(), &anchor.uuid)?;
            }
            None => {}
        }
        // Return collector
        Ok(collector)
    }
}

#[derive(Debug)]
enum AnswerStatus {
    JustCreated,
    Repeat,
    NeedProcessing,
    NeedInjection,
    Completed,
}

#[derive(Debug)]
struct AnswerState {
    pub status: AnswerStatus,
    pub query: ModelContent,
    pub reply: String,
    pub variables: Variables,
}

struct AnswerTagBehavior;

impl TagBehavior for AnswerTagBehavior {}

impl DynamicTagBehavior for AnswerTagBehavior {
    type State = AnswerState;
    fn mono(
        worker: &Worker,
        mut collector: Collector,
        mut state: Self::State,
    ) -> Result<(Option<Collector>, Option<Self::State>, Option<String>)> {
        match state.status {
            AnswerStatus::JustCreated => {
                // Prepare the query
                state.status = AnswerStatus::NeedProcessing;
                state.query = collector.context.clone();
                state.reply = String::new();
                state.variables = collector.variables.clone();
                Ok((None, Some(state), Some(String::new())))
            }
            AnswerStatus::NeedProcessing => {
                // Execute the model query
                let response = worker.call_model(&state.query, &state.variables)?;
                state.reply = response;
                state.status = AnswerStatus::NeedInjection;
                Ok((None, Some(state), None))
            }
            AnswerStatus::NeedInjection => {
                // Inject the reply into the document
                let output = state.reply.clone();
                state.status = AnswerStatus::Completed;
                Ok((None, Some(state), Some(output)))
            }
            AnswerStatus::Completed => {
                // Nothing to do
                Ok((collector, None, None))
            }
        }
    }
}

#[derive(Debug)]
struct IncludeTagBehaviour;

impl TagBehavior for IncludeTagBehaviour {}

impl StaticTagBehavior for IncludeTagBehaviour {
    fn collect_tag(
        worker: &Worker,
        mut collector: Collector,
        tag: &Tag,
    ) -> Result<Option<Collector>> {
        let included_context_name = tag
            .arguments
            .arguments
            .get(0)
            .ok_or_else(|| anyhow::anyhow!("Missing argument for include tag"))?
            .value
            .clone();
        worker.collect(collector, &included_context_name)
    }
}

struct SetTagBehaviour;

impl TagBehavior for SetTagBehaviour {}

impl StaticTagBehavior for SetTagBehaviour {
    fn collect_tag(
        worker: &Worker,
        mut collector: Collector,
        tag: &Tag,
    ) -> Result<Option<Collector>> {
        Ok(Some(collector.update(&tag.parameters)))
    }
}

/*
#[enum_dispatch(TagBehavior)]
#[derive(Debug)]
pub enum TagBehaviorDispatch {
    Answer(AnswerTagBehavior),
    Include(IncludeTagBehaviour),
    Static(StaticTagBehaviour),
}
*/