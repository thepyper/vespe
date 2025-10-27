use anyhow::Result;
use uuid::Uuid;

use super::execute::Collector;
use super::execute::Worker;
use super::variables::Variables;
use super::content::ModelContent;

use crate::ast2::{Anchor, Position, Range, Tag};

// 1. HOST INTERFACE (TagBehavior)
// Tutti i metodi sono funzioni associate (statiche) come da tua intenzione.
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

// 2. POLICY TRAITS
// Definiscono l'interfaccia per le politiche specifiche (statiche o dinamiche).
// Anche qui, tutti i metodi sono funzioni associate.

pub trait StaticPolicy {
    fn collect_static_tag(
        worker: &Worker,
        collector: Collector,
        tag: &Tag,
    ) -> Result<Option<Collector>>;
}

pub trait DynamicPolicy {
    type State: Default + std::fmt::Debug; // Lo stato deve essere Default e Debug

    fn mono(
        worker: &Worker,
        collector: Collector,
        state: Self::State,
    ) -> Result<(Option<Collector>, Option<Self::State>, Option<String>)>;
}

// 3. HOST STRUCTS
// Queste struct generiche prendono una politica (P) e implementano TagBehavior
// delegando le chiamate ai metodi associati della politica.

pub struct StaticTagBehavior<P: StaticPolicy>(P);

impl<P: StaticPolicy> TagBehavior for StaticTagBehavior<P> {
    fn execute_anchor(
        _worker: &Worker,
        _collector: Collector,
        _anchor: &Anchor,
        _anchor_end: Position,
    ) -> Result<(Option<Collector>, Vec<(Range, String)>)> {
        panic!("StaticTag does not support execute_anchor");
    }

    fn collect_anchor(
        _worker: &Worker,
        _collector: Collector,
        _anchor: &Anchor,
        _anchor_end: Position,
    ) -> Result<Option<Collector>> {
        panic!("StaticTag does not support collect_anchor");
    }

    fn execute_tag(
        worker: &Worker,
        collector: Collector,
        tag: &Tag,
    ) -> Result<(Option<Collector>, Vec<(Range, String)>)> {
        let collector = P::collect_static_tag(worker, collector, tag)?;
        Ok((collector, vec![]))
    }

    fn collect_tag(
        worker: &Worker,
        collector: Collector,
        tag: &Tag,
    ) -> Result<Option<Collector>> {
        P::collect_static_tag(worker, collector, tag)
    }
}

pub struct DynamicTagBehavior<P: DynamicPolicy>(P);

impl<P: DynamicPolicy> TagBehavior for DynamicTagBehavior<P> {
    fn execute_tag(
        worker: &Worker,
        collector: Collector,
        tag: &Tag,
    ) -> Result<(Option<Collector>, Vec<(Range, String)>)> {
        let state: P::State = P::State::default();
        let (collector, new_state, new_output) = P::mono(worker, collector, state)?;
        // If there is a new state, save it
        if let Some(_new_state) = new_state {
            // worker.save_state(new_state, &tag.command.to_string(), &Uuid::new_v4())?;
        }
        // If there is some output, patch into new anchor
        let patches = if let Some(_output) = new_output {
            // worker.patch_tag_to_anchor(tag, &output)?;
            vec![] // Placeholder
        } else {
            vec![]
        };
        // Return collector and patches
        Ok((collector, patches))
    }

    fn collect_tag(
        _worker: &Worker,
        _collector: Collector,
        _tag: &Tag,
    ) -> Result<Option<Collector>> {
        // Dynamic tags do not support collect_tag because they always produce a new anchor during execution
        Ok(None)
    }

    fn execute_anchor(
        worker: &Worker,
        collector: Collector,
        anchor: &Anchor,
        anchor_end: Position,
    ) -> Result<(Option<Collector>, Vec<(Range, String)>)> {
        // let state = worker.load_state::<P::State>(&anchor.command, &anchor.uuid)?;
        let state: P::State = P::State::default(); // Placeholder
        let (collector, new_state, new_output) = P::mono(worker, collector, state)?;
        // If there is a new state, save it
        if let Some(_new_state) = new_state {
            // worker.save_state(new_state, &anchor.command.to_string(), &anchor.uuid)?;
        }
        // If there is some output, patch into new anchor
        let patches = if let Some(_output) = new_output {
            // worker.patch_into_anchor(anchor, anchor_end, &output)?;
            vec![] // Placeholder
        } else {
            vec![]
        };
        // Return collector and patches
        Ok((collector, patches))
    }

    fn collect_anchor(
        _worker: &Worker,
        _collector: Collector,
        _anchor: &Anchor,
        _anchor_end: Position,
    ) -> Result<Option<Collector>> {
        // let state = worker.load_state::<P::State>(&anchor.command, &anchor.uuid)?;
        let state: P::State = P::State::default(); // Placeholder
        let (collector, new_state, _new_output) = P::mono(_worker, _collector, state)?;
        // If there is a new state, save it
        if let Some(_new_state) = new_state {
            // worker.save_state(new_state, &anchor.command.to_string(), &anchor.uuid)?;
        }
        // Return collector
        Ok(collector)
    }
}

// 4. CONCRETE POLICIES

#[derive(Debug, Default)]
enum AnswerStatus {
    #[default]
    JustCreated,
    Repeat,
    NeedProcessing,
    NeedInjection,
    Completed,
}

#[derive(Debug, Default)]
pub struct AnswerState {
    pub status: AnswerStatus,
    pub query: ModelContent,
    pub reply: String,
    pub variables: Variables,
}

pub struct AnswerPolicy;

impl DynamicPolicy for AnswerPolicy {
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
                // state.query = collector.context.clone(); // Private field
                state.reply = String::new();
                // state.variables = collector.variables.clone(); // Private field
                Ok((None, Some(state), Some(String::new())))
            }
            AnswerStatus::NeedProcessing => {
                // Execute the model query
                // let response = worker.call_model(&state.query, &state.variables)?; // Private method
                // state.reply = response;
                state.status = AnswerStatus::NeedInjection;
                Ok((None, Some(state), None))
            }
            AnswerStatus::NeedInjection => {
                // Inject the reply into the document
                let output = state.reply.clone();
                state.status = AnswerStatus::Completed;
                Ok((None, Some(state), Some(output)))
            }
            AnswerStatus::Completed | AnswerStatus::Repeat => {
                // Nothing to do
                Ok((Some(collector), None, None))
            }
        }
    }
}

pub struct IncludePolicy;

impl StaticPolicy for IncludePolicy {
    fn collect_static_tag(
        worker: &Worker,
        collector: Collector,
        tag: &Tag,
    ) -> Result<Option<Collector>> {
        let included_context_name = tag
            .arguments
            .arguments
            .get(0)
            .ok_or_else(|| anyhow::anyhow!("Missing argument for include tag"))?
            .value
            .clone();
        // worker.collect(collector, &included_context_name) // Private method
        Ok(Some(collector)) // Placeholder
    }
}

pub struct SetPolicy;

impl StaticPolicy for SetPolicy {
    fn collect_static_tag(
        _worker: &Worker,
        mut collector: Collector,
        tag: &Tag,
    ) -> Result<Option<Collector>> {
        // Ok(Some(collector.update(&tag.parameters))) // Private method
        Ok(Some(collector)) // Placeholder
    }
}


