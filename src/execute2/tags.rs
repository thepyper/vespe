use anyhow::Result;
use enum_dispatch::enum_dispatch;
use handlebars::template::Parameter;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::content::ModelContent;
use super::execute::Collector;
use super::execute::Worker;
use super::variables::Variables;

use super::tag_answer::AnswerPolicy;
use super::tag_include::IncludePolicy;
use super::tag_repeat::RepeatPolicy;
use super::tag_set::SetPolicy;

use crate::ast2::{Anchor, Arguments, CommandKind, Parameters, Position, Range, Tag};

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

pub trait StaticPolicy {
    fn collect_static_tag(worker: &Worker, collector: Collector, tag: &Tag) -> Result<Collector>;
}

pub struct DynamicPolicyMonoResult<State> {
    pub do_next_pass: bool,
    pub collector: Collector,
    pub new_state: Option<State>,
    pub new_output: Option<String>,
    pub new_patches: Vec<(Range, String)>,
}

impl<T> DynamicPolicyMonoResult<T> {
    pub fn new<State>(collector: Collector) -> DynamicPolicyMonoResult<State> {
        DynamicPolicyMonoResult {
            do_next_pass: false,
            collector,
            new_state: None,
            new_output: None,
            new_patches: vec![],
        }
    }
}

pub trait DynamicPolicy {
    type State: Default + std::fmt::Debug + Serialize + for<'de> Deserialize<'de>; // Lo stato deve essere Default e Debug

    fn mono(
        worker: &Worker,
        collector: Collector,
        parameters: &Parameters,
        arguments: &Arguments,
        state: Self::State,
        readonly: bool,
    ) -> Result<DynamicPolicyMonoResult<Self::State>>;
}

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
        let mono_result = P::mono(
            worker,
            collector,
            &tag.parameters,
            &tag.arguments,
            state,
            false,
        )?;
        // Mutate tag into a new anchor
        let (uuid, patches_2) = worker.tag_to_anchor(
            &mono_result.collector,
            tag,
            &mono_result.new_output.unwrap_or(String::new()),
        )?;
        // If there is a new state, save it
        if let Some(new_state) = mono_result.new_state {
            worker.save_state::<P::State>(tag.command, &uuid, &new_state, None)?;
        } // TODO se nnn c'e', errore!! deve mutare in anchor!!
          // Return collector and patches
        let mut patches = mono_result.new_patches;
        patches.extend(patches_2);
        Ok((mono_result.do_next_pass, mono_result.collector, patches))
    }

    fn collect_tag(
        _worker: &Worker,
        collector: Collector,
        _tag: &Tag,
    ) -> Result<(bool, Collector)> {
        // Dynamic tags do not support collect_tag because they always produce a new anchor during execution, then trigger a new pass
        Ok((true, collector))
    }

    fn execute_anchor(
        worker: &Worker,
        collector: Collector,
        anchor: &Anchor,
        anchor_end: Position,
    ) -> Result<(bool, Collector, Vec<(Range, String)>)> {
        let state = worker.load_state::<P::State>(anchor.command, &anchor.uuid)?;
        let mono_result = P::mono(
            worker,
            collector,
            &anchor.parameters,
            &anchor.arguments,
            state,
            false,
        )?;
        // If there is a new state, save it
        if let Some(new_state) = mono_result.new_state {
            worker.save_state::<P::State>(anchor.command, &anchor.uuid, &new_state, None)?;
        }
        // If there is some output, patch into new anchor
        let patches_2 = if let Some(output) = mono_result.new_output {
            worker.inject_into_anchor(&mono_result.collector, anchor, &anchor_end, &output)?
        } else {
            vec![]
        };
        // Return collector and patches
        let mut patches = mono_result.new_patches;
        patches.extend(patches_2);
        Ok((mono_result.do_next_pass, mono_result.collector, patches))
    }

    fn collect_anchor(
        worker: &Worker,
        collector: Collector,
        anchor: &Anchor,
        anchor_end: Position,
    ) -> Result<(bool, Collector)> {
        let state = worker.load_state::<P::State>(anchor.command, &anchor.uuid)?;
        let mono_result = P::mono(
            worker,
            collector,
            &anchor.parameters,
            &anchor.arguments,
            state,
            true,
        )?;
        // If there is some patches, just discard them and new state as well as it cannot be applied
        if !mono_result.new_patches.is_empty() {
            tracing::warn!("Warning, anchor produced some patches even on readonly phase.\nAnchor = {:?}\nPatches = {:?}\n", anchor, mono_result.new_patches);
            return Ok((true, mono_result.collector));
        }
        // If there is new output, just discard it and new state as well as it cannot be injected
        if let Some(output) = mono_result.new_output {
            tracing::warn!("Warning, anchor produced some output even on readonly phase.\nAnchor = {:?}\nOutput = {:?}\n", anchor, output);
            return Ok((true, mono_result.collector));
        };
        // If there is a new state, save it
        if let Some(new_state) = mono_result.new_state {
            worker.save_state::<P::State>(anchor.command, &anchor.uuid, &new_state, None)?;
        }
        // Return collector
        Ok((mono_result.do_next_pass, mono_result.collector))
    }
}

pub(crate) struct TagBehaviorDispatch;

impl TagBehaviorDispatch {
    pub fn execute_tag(
        worker: &Worker,
        collector: Collector,
        tag: &Tag,
    ) -> Result<(bool, Collector, Vec<(Range, String)>)> {
        match tag.command {
            CommandKind::Answer => {
                DynamicTagBehavior::<AnswerPolicy>::execute_tag(worker, collector, tag)
            }
            CommandKind::Repeat => {
                DynamicTagBehavior::<RepeatPolicy>::execute_tag(worker, collector, tag)
            }
            CommandKind::Include => {
                StaticTagBehavior::<IncludePolicy>::execute_tag(worker, collector, tag)
            }
            CommandKind::Set => StaticTagBehavior::<SetPolicy>::execute_tag(worker, collector, tag),
            _ => Err(anyhow::anyhow!("Unsupported tag command")),
        }
    }
    pub fn collect_tag(
        worker: &Worker,
        collector: Collector,
        tag: &Tag,
    ) -> Result<(bool, Collector)> {
        match tag.command {
            CommandKind::Answer => {
                DynamicTagBehavior::<AnswerPolicy>::collect_tag(worker, collector, tag)
            }
            CommandKind::Repeat => {
                DynamicTagBehavior::<RepeatPolicy>::collect_tag(worker, collector, tag)
            }
            CommandKind::Include => {
                StaticTagBehavior::<IncludePolicy>::collect_tag(worker, collector, tag)
            }
            CommandKind::Set => StaticTagBehavior::<SetPolicy>::collect_tag(worker, collector, tag),
            _ => Err(anyhow::anyhow!("Unsupported tag command")),
        }
    }
    pub fn execute_anchor(
        worker: &Worker,
        collector: Collector,
        anchor: &Anchor,
        anchor_end: Position,
    ) -> Result<(bool, Collector, Vec<(Range, String)>)> {
        match anchor.command {
            CommandKind::Answer => DynamicTagBehavior::<AnswerPolicy>::execute_anchor(
                worker, collector, anchor, anchor_end,
            ),
            CommandKind::Repeat => DynamicTagBehavior::<RepeatPolicy>::execute_anchor(
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
        match anchor.command {
            CommandKind::Answer => DynamicTagBehavior::<AnswerPolicy>::collect_anchor(
                worker, collector, anchor, anchor_end,
            ),
            CommandKind::Repeat => DynamicTagBehavior::<RepeatPolicy>::collect_anchor(
                worker, collector, anchor, anchor_end,
            ),
            _ => Err(anyhow::anyhow!("Unsupported anchor command")),
        }
    }
}
