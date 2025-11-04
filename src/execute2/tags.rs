use anyhow::Result;
use enum_dispatch::enum_dispatch;
use handlebars::template::Parameter;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::content::{ModelContent, ModelContentItem};
use super::execute::Collector;
use super::execute::Worker;
use super::variables::Variables;
use super::REDIRECTED_OUTPUT_PLACEHOLDER;

use super::tag_answer::AnswerPolicy;
use super::tag_forget::ForgetPolicy;
use super::tag_include::IncludePolicy;
use super::tag_repeat::RepeatPolicy;
use super::tag_set::SetPolicy;

use crate::ast2::{Anchor, Arguments, CommandKind, Parameters, Position, Range, Tag};

// 1. HOST INTERFACE (TagBehavior)
// Tutti i metodi sono funzioni associate (statiche) come da tua intenzione.
pub trait TagBehavior {
    fn execute_tag(
        &self,
        worker: &Worker,
        collector: Collector,
        local_variables: &Variables,
        tag: &Tag,
    ) -> Result<(bool, Collector, Vec<(Range, String)>)>;
    fn collect_tag(
        &self,
        worker: &Worker,
        collector: Collector,
        local_variables: &Variables,
        tag: &Tag,
    ) -> Result<(bool, Collector)>;
    fn execute_anchor(
        &self,
        worker: &Worker,
        collector: Collector,
        local_variables: &Variables,
        anchor: &Anchor,
        anchor_end: Position,
    ) -> Result<(bool, Collector, Vec<(Range, String)>)>;
    fn collect_anchor(
        &self,
        worker: &Worker,
        collector: Collector,
        local_variables: &Variables,
        anchor: &Anchor,
        anchor_end: Position,
    ) -> Result<(bool, Collector)>;
}

pub trait StaticPolicy {
    fn collect_static_tag(
        worker: &Worker,
        collector: Collector,
        local_variables: &Variables,
        tag: &Tag,
    ) -> Result<Collector>;
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
        local_variables: &Variables,
        input: &ModelContent,
        parameters: &Parameters,
        arguments: &Arguments,
        state: Self::State,
        readonly: bool,
    ) -> Result<DynamicPolicyMonoResult<Self::State>>;
}

pub struct StaticTagBehavior<P: StaticPolicy>(P);

impl<P: StaticPolicy> TagBehavior for StaticTagBehavior<P> {
    fn execute_anchor(
        &self,
        _worker: &Worker,
        _collector: Collector,
        _local_variables: &Variables,
        _anchor: &Anchor,
        _anchor_end: Position,
    ) -> Result<(bool, Collector, Vec<(Range, String)>)> {
        panic!("StaticTag does not support execute_anchor");
    }

    fn collect_anchor(
        &self,
        _worker: &Worker,
        _collector: Collector,
        _local_variables: &Variables,
        _anchor: &Anchor,
        _anchor_end: Position,
    ) -> Result<(bool, Collector)> {
        panic!("StaticTag does not support collect_anchor");
    }

    fn execute_tag(
        &self,
        worker: &Worker,
        collector: Collector,
        local_variables: &Variables,
        tag: &Tag,
    ) -> Result<(bool, Collector, Vec<(Range, String)>)> {
        let collector = P::collect_static_tag(worker, collector, local_variables, tag)?;
        Ok((false, collector, vec![]))
    }

    fn collect_tag(
        &self,
        worker: &Worker,
        collector: Collector,
        local_variables: &Variables,
        tag: &Tag,
    ) -> Result<(bool, Collector)> {
        let collector = P::collect_static_tag(worker, collector, local_variables, tag)?;
        Ok((false, collector))
    }
}

pub struct DynamicTagBehavior<P: DynamicPolicy>(P);

impl<P: DynamicPolicy> TagBehavior for DynamicTagBehavior<P> {
    fn execute_tag(
        &self,
        worker: &Worker,
        collector: Collector,
        local_variables: &Variables,
        tag: &Tag,
    ) -> Result<(bool, Collector, Vec<(Range, String)>)> {
        let state: P::State = P::State::default();
        let input = worker.redirect_input(local_variables, collector.context().clone())?;
        let mono_result = P::mono(
            worker,
            collector,
            local_variables,
            &input,
            &tag.parameters,
            &tag.arguments,
            state,
            false,
        )?;
        // Mutate tag into a new anchor
        let (uuid, patches_2) = worker.tag_to_anchor(
            &mono_result.collector,
            local_variables,
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
        &self,
        _worker: &Worker,
        collector: Collector,
        local_variables: &Variables,
        _tag: &Tag,
    ) -> Result<(bool, Collector)> {
        // Dynamic tags do not support collect_tag because they always produce a new anchor during execution, then trigger a new pass
        Ok((true, collector))
    }

    fn execute_anchor(
        &self,
        worker: &Worker,
        collector: Collector,
        local_variables: &Variables,
        anchor: &Anchor,
        anchor_end: Position,
    ) -> Result<(bool, Collector, Vec<(Range, String)>)> {
        let state = worker.load_state::<P::State>(anchor.command, &anchor.uuid)?;
        let input = worker.redirect_input(local_variables, collector.context().clone())?;
        let mut mono_result = P::mono(
            worker,
            collector,
            local_variables,
            &input,
            &anchor.parameters,
            &anchor.arguments,
            state,
            false,
        )?;
        // If output has been redirected, place output redirected placeholder
        if let Some(_) = local_variables.output {
            mono_result
                .collector
                .push_item(ModelContentItem::system(REDIRECTED_OUTPUT_PLACEHOLDER));
        }
        // If there is a new state, save it
        if let Some(new_state) = mono_result.new_state {
            worker.save_state::<P::State>(anchor.command, &anchor.uuid, &new_state, None)?;
        }
        // If there is some output, patch into new anchor
        let patches_2 = if let Some(output) = mono_result.new_output {
            worker.inject_into_anchor(
                &mono_result.collector,
                local_variables,
                anchor,
                &anchor_end,
                &output,
            )?
        } else {
            vec![]
        };
        // Return collector and patches
        let mut patches = mono_result.new_patches;
        patches.extend(patches_2);
        Ok((mono_result.do_next_pass, mono_result.collector, patches))
    }

    fn collect_anchor(
        &self,
        worker: &Worker,
        collector: Collector,
        local_variables: &Variables,
        anchor: &Anchor,
        anchor_end: Position,
    ) -> Result<(bool, Collector)> {
        let state = worker.load_state::<P::State>(anchor.command, &anchor.uuid)?;
        let input = worker.redirect_input(local_variables, collector.context().clone())?;
        let mut mono_result = P::mono(
            worker,
            collector,
            local_variables,
            &input,
            &anchor.parameters,
            &anchor.arguments,
            state,
            true,
        )?;
        // If output has been redirected, place output redirected placeholder
        if let Some(_) = local_variables.output {
            mono_result
                .collector
                .push_item(ModelContentItem::system(REDIRECTED_OUTPUT_PLACEHOLDER));
        }
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
    fn get_behavior(command: CommandKind) -> Result<Box<dyn TagBehavior>> {
        match command {
            CommandKind::Answer => Ok(Box::new(DynamicTagBehavior(AnswerPolicy))),
            CommandKind::Repeat => Ok(Box::new(DynamicTagBehavior(RepeatPolicy))),
            CommandKind::Include => Ok(Box::new(StaticTagBehavior(IncludePolicy))),
            CommandKind::Set => Ok(Box::new(StaticTagBehavior(SetPolicy))),
            CommandKind::Forget => Ok(Box::new(StaticTagBehavior(ForgetPolicy))),
            _ => Err(anyhow::anyhow!("Unsupported command: {:?}", command)),
        }
    }

    pub fn execute_tag(
        worker: &Worker,
        collector: Collector,
        local_variables: &Variables,
        tag: &Tag,
    ) -> Result<(bool, Collector, Vec<(Range, String)>)> {
        let behavior = Self::get_behavior(tag.command)?;
        behavior.execute_tag(worker, collector, local_variables, tag)
    }

    pub fn collect_tag(
        worker: &Worker,
        collector: Collector,
        local_variables: &Variables,
        tag: &Tag,
    ) -> Result<(bool, Collector)> {
        let behavior = Self::get_behavior(tag.command)?;
        behavior.collect_tag(worker, collector, local_variables, tag)
    }

    pub fn execute_anchor(
        worker: &Worker,
        collector: Collector,
        local_variables: &Variables,
        anchor: &Anchor,
        anchor_end: Position,
    ) -> Result<(bool, Collector, Vec<(Range, String)>)> {
        let behavior = Self::get_behavior(anchor.command)?;
        behavior.execute_anchor(worker, collector, local_variables, anchor, anchor_end)
    }

    pub fn collect_anchor(
        worker: &Worker,
        collector: Collector,
        local_variables: &Variables,
        anchor: &Anchor,
        anchor_end: Position,
    ) -> Result<(bool, Collector)> {
        let behavior = Self::get_behavior(anchor.command)?;
        behavior.collect_anchor(worker, collector, local_variables, anchor, anchor_end)
    }
}
