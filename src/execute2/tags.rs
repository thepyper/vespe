use anyhow::Result;
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::content::ModelContent;
use super::execute::Collector;
use super::execute::Worker;
use super::variables::Variables;

use crate::ast2::{Anchor, CommandKind, Position, Range, Tag};

// 1. HOST INTERFACE (TagBehavior)
// Tutti i metodi sono funzioni associate (statiche) come da tua intenzione.
pub trait TagBehavior {
    fn execute_tag(
        worker: &Worker,
        collector: Collector,
        tag: &Tag,
    ) -> Result<(bool, Collector, Vec<(Range, String)>)>;
    fn collect_tag(worker: &Worker, collector: Collector, tag: &Tag) -> Result<(bool, Collector)>;
    fn execute_anchor(
        worker: &Worker,
        collector: Collector,
        anchor: &Anchor,
        anchor_end: Position,
    ) -> Result<(bool, Collector, Vec<(Range, String)>)>;
    fn collect_anchor(
        worker: &Worker,
        collector: Collector,
        anchor: &Anchor,
        anchor_end: Position,
    ) -> Result<(bool, Collector)>;
}

// 2. POLICY TRAITS
// Definiscono l'interfaccia per le politiche specifiche (statiche o dinamiche).
// Anche qui, tutti i metodi sono funzioni associate.

pub trait StaticPolicy {
    fn collect_static_tag(worker: &Worker, collector: Collector, tag: &Tag) -> Result<Collector>;
}

pub trait DynamicPolicy {
    type State: Default + std::fmt::Debug + Serialize + for<'de> Deserialize<'de>; // Lo stato deve essere Default e Debug

    fn mono(
        worker: &Worker,
        collector: Collector,
        state: Self::State,
    ) -> Result<(bool, Collector, Option<Self::State>, Option<String>)>;
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
    ) -> Result<(bool, Collector, Vec<(Range, String)>)> {
        panic!("StaticTag does not support execute_anchor");
    }

    fn collect_anchor(
        _worker: &Worker,
        _collector: Collector,
        _anchor: &Anchor,
        _anchor_end: Position,
    ) -> Result<(bool, Collector)> {
        panic!("StaticTag does not support collect_anchor");
    }

    fn execute_tag(
        worker: &Worker,
        collector: Collector,
        tag: &Tag,
    ) -> Result<(bool, Collector, Vec<(Range, String)>)> {
        let collector = P::collect_static_tag(worker, collector, tag)?;
        Ok((false, collector, vec![]))
    }

    fn collect_tag(worker: &Worker, collector: Collector, tag: &Tag) -> Result<(bool, Collector)> {
        let collector = P::collect_static_tag(worker, collector, tag)?;
        Ok((false, collector))
    }
}

pub struct DynamicTagBehavior<P: DynamicPolicy>(P);

impl<P: DynamicPolicy> TagBehavior for DynamicTagBehavior<P> {
    fn execute_tag(
        worker: &Worker,
        collector: Collector,
        tag: &Tag,
    ) -> Result<(bool, Collector, Vec<(Range, String)>)> {
        let state: P::State = P::State::default();
        let (do_next_pass, collector, new_state, new_output) = P::mono(worker, collector, state)?;
        // If there is some output, patch into new anchor
        let (uuid, patches) =
            worker.tag_to_anchor(&collector, tag, &new_output.unwrap_or(String::new()))?;
        // If there is a new state, save it
        if let Some(new_state) = new_state {
            worker.save_state::<P::State>(tag.command, &uuid, &new_state, None)?;
        }
        // Return collector and patches
        Ok((do_next_pass, collector, patches))
    }

    fn collect_tag(
        _worker: &Worker,
        _collector: Collector,
        _tag: &Tag,
    ) -> Result<(bool, Collector)> {
        // Dynamic tags do not support collect_tag because they always produce a new anchor during execution
        panic!("Dynamic tags do not support collect_tag because they always produce a new anchor during execution");
    }

    fn execute_anchor(
        worker: &Worker,
        collector: Collector,
        anchor: &Anchor,
        anchor_end: Position,
    ) -> Result<(bool, Collector, Vec<(Range, String)>)> {
        let state = worker.load_state::<P::State>(anchor.command, &anchor.uuid)?;
        let (do_next_pass, collector, new_state, new_output) = P::mono(worker, collector, state)?;
        // If there is a new state, save it
        if let Some(new_state) = new_state {
            worker.save_state::<P::State>(anchor.command, &anchor.uuid, &new_state, None)?;
        }
        // If there is some output, patch into new anchor
        let patches = if let Some(output) = new_output {
            worker.inject_into_anchor(&collector, anchor, &anchor_end, &output)?
        } else {
            vec![]
        };
        // Return collector and patches
        Ok((do_next_pass, collector, patches))
    }

    fn collect_anchor(
        worker: &Worker,
        collector: Collector,
        anchor: &Anchor,
        anchor_end: Position,
    ) -> Result<(bool, Collector)> {
        let state = worker.load_state::<P::State>(anchor.command, &anchor.uuid)?;
        let (do_next_pass, collector, new_state, _new_output) = P::mono(worker, collector, state)?;
        // If there is a new state, save it
        if let Some(new_state) = new_state {
            worker.save_state::<P::State>(anchor.command, &anchor.uuid, &new_state, None)?;
        }
        // Return collector
        Ok((do_next_pass, collector))
    }
}

// 4. CONCRETE POLICIES

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

pub struct IncludePolicy;

impl StaticPolicy for IncludePolicy {
    fn collect_static_tag(worker: &Worker, collector: Collector, tag: &Tag) -> Result<Collector> {
        let included_context_name = tag
            .arguments
            .arguments
            .get(0)
            .ok_or_else(|| anyhow::anyhow!("Missing argument for include tag"))?
            .value
            .clone();
        tracing::debug!("Including context: {}", included_context_name);
        match worker.execute(collector, &included_context_name, 0)? {
            Some(collector) => Ok(collector),
            None => Err(anyhow::anyhow!("Included context returned no collector")),
        }
    }
}

pub struct SetPolicy;

impl StaticPolicy for SetPolicy {
    fn collect_static_tag(_worker: &Worker, collector: Collector, tag: &Tag) -> Result<Collector> {
        tracing::debug!("Setting variables: {:?}", tag.parameters);
        Ok(collector.update(&tag.parameters))
    }
}

pub(crate) struct TagBehaviorDispatch;

impl TagBehaviorDispatch {
    pub fn execute_tag(
        worker: &Worker,
        collector: Collector,
        tag: &Tag,
    ) -> Result<(bool, Collector, Vec<(Range, String)>)> {
        tracing::debug!("Executing tag: {:?}", tag);
        match tag.command {
            crate::ast2::CommandKind::Answer => {
                DynamicTagBehavior::<AnswerPolicy>::execute_tag(worker, collector, tag)
            }
            crate::ast2::CommandKind::Include => {
                StaticTagBehavior::<IncludePolicy>::execute_tag(worker, collector, tag)
            }
            crate::ast2::CommandKind::Set => {
                StaticTagBehavior::<SetPolicy>::execute_tag(worker, collector, tag)
            }
            _ => Err(anyhow::anyhow!("Unsupported tag command")),
        }
    }
    pub fn collect_tag(
        worker: &Worker,
        collector: Collector,
        tag: &Tag,
    ) -> Result<(bool, Collector)> {
        tracing::debug!("Collecting tag: {:?}", tag);
        match tag.command {
            crate::ast2::CommandKind::Answer => {
                DynamicTagBehavior::<AnswerPolicy>::collect_tag(worker, collector, tag)
            }
            crate::ast2::CommandKind::Include => {
                StaticTagBehavior::<IncludePolicy>::collect_tag(worker, collector, tag)
            }
            crate::ast2::CommandKind::Set => {
                StaticTagBehavior::<SetPolicy>::collect_tag(worker, collector, tag)
            }
            _ => Err(anyhow::anyhow!("Unsupported tag command")),
        }
    }
    pub fn execute_anchor(
        worker: &Worker,
        collector: Collector,
        anchor: &Anchor,
        anchor_end: Position,
    ) -> Result<(bool, Collector, Vec<(Range, String)>)> {
        tracing::debug!("Executing anchor: {:?}", anchor);
        match anchor.command {
            crate::ast2::CommandKind::Answer => DynamicTagBehavior::<AnswerPolicy>::execute_anchor(
                worker, collector, anchor, anchor_end,
            ),
            _ => Err(anyhow::anyhow!("Unsupported anchor command")),
        }
    }
    pub fn collect_anchor(
        worker: &Worker,
        collector: Collector,
        anchor: &Anchor,
        anchor_end: Position,
    ) -> Result<(bool, Collector)> {
        tracing::debug!("Collecting anchor: {:?}", anchor);
        match anchor.command {
            crate::ast2::CommandKind::Answer => DynamicTagBehavior::<AnswerPolicy>::collect_anchor(
                worker, collector, anchor, anchor_end,
            ),
            _ => Err(anyhow::anyhow!("Unsupported anchor command")),
        }
    }
}
