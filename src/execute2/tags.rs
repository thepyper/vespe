//! This module defines the core traits and dispatch mechanisms for handling various
//! tags and anchors within the execution engine. It establishes a clear interface
//! for implementing both static and dynamic tag behaviors, enabling the engine
//! to process directives like `@include`, `@set`, `@answer`, and `@repeat`.
use super::{ExecuteError, Result};

use serde::{Deserialize, Serialize};

use super::content::{ModelContent, ModelContentItem};
use super::execute::Collector;
use super::execute::Worker;
use super::REDIRECTED_OUTPUT_PLACEHOLDER;

use super::tag_answer::AnswerPolicy;
use super::tag_comment::CommentPolicy;
use super::tag_done::DonePolicy;
use super::tag_forget::ForgetPolicy;
use super::tag_include::IncludePolicy;
use super::tag_inline::InlinePolicy;
use super::tag_repeat::RepeatPolicy;
use super::tag_set::SetPolicy;
use super::tag_task::TaskPolicy;

use crate::ast2::{Anchor, Arguments, CommandKind, Parameters, Position, Range, Tag};

/// Defines the behavior for processing a tag or an anchor.
///
/// This trait is implemented by different policy wrappers ([`StaticTagBehavior`], [`DynamicTagBehavior`])
/// to provide a unified interface for the `Worker` to interact with various command types.
pub trait TagBehavior {
    /// Executes the behavior of a tag in a modifying pass.
    ///
    /// This method is called when the engine is in an `execute` (non-readonly) phase
    /// and encounters a tag. It can produce patches to modify the source file.
    ///
    /// # Arguments
    ///
    /// * `worker` - A reference to the [`Worker`] instance, providing access to file system,
    ///              path resolution, and state management utilities.
    /// * `collector` - The current [`Collector`] state.
    /// * `tag` - The [`Tag`] being processed.
    ///
    /// # Returns
    ///
    /// A `Result` containing a tuple: `(do_next_pass, updated_collector, patches)`.
    /// `do_next_pass` indicates if another pass is required. `updated_collector` is the
    /// [`Collector`] after this operation. `patches` are modifications to be applied to the source file.
    ///
    /// # Errors
    ///
    /// Returns an [`ExecuteError`] if any operation fails during tag execution.
    fn execute_tag(
        &self,
        worker: &Worker,
        collector: Collector,
        document: &str,
        tag: &Tag,
    ) -> Result<(bool, Collector, Vec<(Range, String)>)>;

    /// Collects the content associated with a tag in a read-only pass.
    ///
    /// This method is called when the engine is in a `collect` (readonly) phase
    /// and encounters a tag. It should not produce any file modifications.
    ///
    /// # Arguments
    ///
    /// * `worker` - A reference to the [`Worker`] instance.
    /// * `collector` - The current [`Collector`] state.
    /// * `tag` - The [`Tag`] being processed.
    ///
    /// # Returns
    ///
    /// A `Result` containing a tuple: `(do_next_pass, updated_collector)`.
    /// `do_next_pass` indicates if another pass is required. `updated_collector` is the
    /// [`Collector`] after this operation.
    ///
    /// # Errors
    ///
    /// Returns an [`ExecuteError`] if any operation fails during tag collection.
    fn collect_tag(
        &self,
        worker: &Worker,
        collector: Collector,
        tag: &Tag,
    ) -> Result<(bool, Collector)>;

    /// Executes the behavior of an anchor in a modifying pass.
    ///
    /// This method is called when the engine is in an `execute` (non-readonly) phase
    /// and encounters an anchor. It can produce patches to modify the source file.
    ///
    /// # Arguments
    ///
    /// * `worker` - A reference to the [`Worker`] instance.
    /// * `collector` - The current [`Collector`] state.
    /// * `anchor` - The [`Anchor`] being processed.
    /// * `anchor_end` - The [`Position`] marking the end of the anchor's content.
    ///
    /// # Returns
    ///
    /// A `Result` containing a tuple: `(do_next_pass, updated_collector, patches)`.
    /// `do_next_pass` indicates if another pass is required. `updated_collector` is the
    /// [`Collector`] after this operation. `patches` are modifications to be applied to the source file.
    ///
    /// # Errors
    ///
    /// Returns an [`ExecuteError`] if any operation fails during anchor execution.
    fn execute_anchor(
        &self,
        worker: &Worker,
        collector: Collector,
        document: &str,
        anchor_begin: &Anchor,
        anchor_end: &Anchor,
    ) -> Result<(bool, Collector, Vec<(Range, String)>)>;

    /// Collects the content associated with an anchor in a read-only pass.
    ///
    /// This method is called when the engine is in a `collect` (readonly) phase
    /// and encounters an anchor. It should not produce any file modifications.
    ///
    /// # Arguments
    ///
    /// * `worker` - A reference to the [`Worker`] instance.
    /// * `collector` - The current [`Collector`] state.
    /// * `anchor` - The [`Anchor`] being processed.
    /// * `anchor_end` - The [`Position`] marking the end of the anchor's content.
    ///
    /// # Returns
    ///
    /// A `Result` containing a tuple: `(do_next_pass, updated_collector)`.
    /// `do_next_pass` indicates if another pass is required. `updated_collector` is the
    /// [`Collector`] after this operation.
    ///
    /// # Errors
    ///
    /// Returns an [`ExecuteError`] if any operation fails during anchor collection.
    fn collect_anchor(
        &self,
        worker: &Worker,
        collector: Collector,
        document: &str,
        anchor_begin: &Anchor,
        anchor_end: &Anchor,
    ) -> Result<(bool, Collector)>;
}

#[derive(Clone)]
pub enum TagOrAnchor<'a> {
    Tag(&'a Tag),
    Anchor((&'a Anchor, &'a Anchor)),
}

pub struct StaticPolicyMonoInput<'a> {
    pub readonly: bool,
    pub worker: &'a Worker,
    pub collector: Collector,
    pub tag_or_anchor: TagOrAnchor<'a>,
}

pub struct StaticPolicyMonoInputResidual<'a> {
    pub readonly: bool,
    pub worker: &'a Worker,
    pub tag_or_anchor: TagOrAnchor<'a>,
    pub parameters: &'a Parameters,
    pub arguments: &'a Arguments,
}

pub struct StaticPolicyMonoResult {
    /// Indicates if another execution pass is required after this step.
    pub do_next_pass: bool,
    /// The updated [`Collector`] state.
    pub collector: Collector,
    /// A vector of patches to be applied to the source file.
    pub new_patches: Vec<(Range, String)>,
}

impl StaticPolicyMonoResult {
    pub fn from_inputs(inputs: StaticPolicyMonoInput) -> (Self, StaticPolicyMonoInputResidual) {
        let tag_or_anchor = inputs.tag_or_anchor.clone();
        let parameters = match &tag_or_anchor {
            TagOrAnchor::Tag(tag) => &tag.parameters,
            TagOrAnchor::Anchor((anchor, _)) => &anchor.parameters,
        };
        let arguments = match &tag_or_anchor {
            TagOrAnchor::Tag(tag) => &tag.arguments,
            TagOrAnchor::Anchor((anchor, _)) => &anchor.arguments,
        };
        let residual = StaticPolicyMonoInputResidual {
            readonly: inputs.readonly,
            worker: inputs.worker,
            tag_or_anchor,
            parameters,
            arguments,
        };
        let result = StaticPolicyMonoResult {
            do_next_pass: false,
            collector: inputs.collector,
            new_patches: vec![],
        };
        (result, residual)
    }
}

/// Trait for defining the behavior of static tags.
///
/// Static tags are processed in a single pass and do not involve complex state
/// management or file modifications. They primarily affect the `Collector`'s state.
pub trait StaticPolicy {
    /// Collects the content and updates the collector for a static tag.
    ///
    /// This method is responsible for the core logic of a static tag, such as
    /// including content from another file or setting default parameters.
    ///
    /// # Arguments
    ///
    /// * `worker` - A reference to the [`Worker`] instance.
    /// * `collector` - The current [`Collector`] state.
    /// * `tag` - The [`Tag`] being processed.
    ///
    /// # Returns
    ///
    /// A `Result` containing the updated [`Collector`] state.
    ///
    /// # Errors
    ///
    /// Returns an [`ExecuteError`] if any operation fails during tag processing.
    fn mono(inputs: StaticPolicyMonoInput) -> Result<StaticPolicyMonoResult>;
}

pub struct DynamicPolicyMonoInput<'a, State> {
    pub readonly: bool,
    pub worker: &'a Worker,
    pub collector: Collector,
    pub document: &'a str,
    pub state: State,
    pub tag_or_anchor: TagOrAnchor<'a>,
    pub input: ModelContent,
    pub input_hash: String,
}

impl<'a, T> DynamicPolicyMonoInput<'a, T> {
    pub fn parameters(&self) -> &Parameters {
        match &self.tag_or_anchor {
            TagOrAnchor::Tag(tag) => &tag.parameters,
            TagOrAnchor::Anchor((anchor, _)) => &anchor.parameters,
        }
    }
    pub fn arguments(&self) -> &Arguments {
        match &self.tag_or_anchor {
            TagOrAnchor::Tag(tag) => &tag.arguments,
            TagOrAnchor::Anchor((anchor, _)) => &anchor.arguments,
        }
    }
}

pub struct DynamicPolicyMonoInputResidual<'a, State> {
    pub readonly: bool,
    pub worker: &'a Worker,
    pub document: &'a str,
    pub state: State,
    pub tag_or_anchor: TagOrAnchor<'a>,
    pub parameters: &'a Parameters,
    pub arguments: &'a Arguments,
    pub input: ModelContent,
    pub input_hash: String,
}

/// The result of a single execution step for a dynamic policy.
///
/// This struct encapsulates all possible outcomes of a `mono` call for a dynamic tag,
/// including whether another pass is needed, the updated collector, new state to save,
/// new output to inject, and patches to apply.
pub struct DynamicPolicyMonoResult<State> {
    /// Indicates if another execution pass is required after this step.
    pub do_next_pass: bool,
    /// The updated [`Collector`] state.
    pub collector: Collector,
    /// Optional new state to be saved for the dynamic tag.
    pub new_state: Option<State>,
    /// Optional new output to be injected into the document.
    pub new_output: Option<String>,
    /// A vector of patches to be applied to the source file.
    pub new_patches: Vec<(Range, String)>,
}

impl<'a, T> DynamicPolicyMonoResult<T> {
    pub fn from_inputs(
        inputs: DynamicPolicyMonoInput<'a, T>,
    ) -> (Self, DynamicPolicyMonoInputResidual<'a, T>) {
        let tag_or_anchor = inputs.tag_or_anchor.clone();
        let parameters = match &tag_or_anchor {
            TagOrAnchor::Tag(tag) => &tag.parameters,
            TagOrAnchor::Anchor((anchor, _)) => &anchor.parameters,
        };
        let arguments = match &tag_or_anchor {
            TagOrAnchor::Tag(tag) => &tag.arguments,
            TagOrAnchor::Anchor((anchor, _)) => &anchor.arguments,
        };
        let residual = DynamicPolicyMonoInputResidual {
            readonly: inputs.readonly,
            worker: inputs.worker,
            document: inputs.document,
            state: inputs.state,
            tag_or_anchor,
            parameters,
            arguments,
            input: inputs.input,
            input_hash: inputs.input_hash,
        };
        let result = DynamicPolicyMonoResult::<T> {
            do_next_pass: false,
            collector: inputs.collector,
            new_state: None,
            new_output: None,
            new_patches: vec![],
        };
        (result, residual)
    }
}

/// Trait for defining the behavior of dynamic tags.
///
/// Dynamic tags (e.g., `@answer`, `@repeat`) involve complex, multi-pass execution
/// and state management. They can trigger external model calls and modify the
/// source document over several steps.
pub trait DynamicPolicy {
    /// The state type associated with this dynamic policy.
    ///
    /// This state is persisted between execution passes and allows dynamic tags
    /// to maintain context and progress through their lifecycle.
    type State: Default + std::fmt::Debug + Serialize + for<'de> Deserialize<'de>;

    /// Executes a single step of the dynamic policy's behavior.
    ///
    /// This method is the core logic for dynamic tags, handling state transitions,
    /// model calls, and content generation. It is called repeatedly during the
    /// multi-pass execution until the dynamic tag reaches a stable state.
    ///
    /// # Arguments
    ///
    /// * `worker` - A reference to the [`Worker`] instance.
    /// * `collector` - The current [`Collector`] state.
    /// * `input` - The [`ModelContent`] collected so far, serving as input to the dynamic tag.
    /// * `input_hash` - The SHA256 hash of the `input` content. This is used to detect
    ///                  changes in the input that might require re-execution.
    /// * `parameters` - The [`Parameters`] associated with the tag or anchor.
    /// * `arguments` - The [`Arguments`] associated with the tag or anchor.
    /// * `state` - The current state of the dynamic tag.
    /// * `readonly` - A boolean indicating if the current pass is read-only.
    ///
    /// # Returns
    ///
    /// A `Result` containing a [`DynamicPolicyMonoResult`] which describes the outcome
    /// of this execution step.
    ///
    /// # Errors
    ///
    /// Returns an [`ExecuteError`] if any operation fails during the dynamic policy's execution.
    fn mono(
        inputs: DynamicPolicyMonoInput<Self::State>,
        /* worker: &Worker,
        collector: Collector,
        input: &ModelContent,
        input_hash: String,
        parameters: &Parameters,
        arguments: &Arguments,
        state: Self::State,
        readonly: bool, */
    ) -> Result<DynamicPolicyMonoResult<Self::State>>;
}

/// A wrapper for [`StaticPolicy`] implementations to conform to the [`TagBehavior`] trait.
///
/// This struct allows static tags to be handled uniformly by the `TagBehaviorDispatch`.
pub struct StaticTagBehavior<P: StaticPolicy>(P);

impl<P: StaticPolicy> TagBehavior for StaticTagBehavior<P> {
    /// Panics: Static tags do not support anchor execution.
    ///
    /// This method should never be called for static tags as they do not create or manage anchors.
    ///
    /// # Panics
    ///
    /// Always panics with a message indicating that static tags do not support `execute_anchor`.
    fn execute_anchor(
        &self,
        _worker: &Worker,
        _collector: Collector,
        _document: &str,
        _anchor_begin: &Anchor,
        _anchor_end: &Anchor,
    ) -> Result<(bool, Collector, Vec<(Range, String)>)> {
        panic!("StaticTag does not support execute_anchor");
    }

    /// Panics: Static tags do not support anchor collection.
    ///
    /// This method should never be called for static tags as they do not create or manage anchors.
    ///
    /// # Panics
    ///
    /// Always panics with a message indicating that static tags do not support `collect_anchor`.
    fn collect_anchor(
        &self,
        _worker: &Worker,
        _collector: Collector,
        _document: &str,
        _anchor_begin: &Anchor,
        _anchor_end: &Anchor,
    ) -> Result<(bool, Collector)> {
        panic!("StaticTag does not support collect_anchor");
    }

    /// Executes a static tag, updating the collector.
    ///
    /// This method delegates to the underlying [`StaticPolicy::collect_static_tag`] and
    /// returns `(false, collector, vec![])` as static tags do not trigger new passes
    /// or produce patches in this context.
    ///
    /// # Arguments
    ///
    /// * `worker` - A reference to the [`Worker`] instance.
    /// * `collector` - The current [`Collector`] state.
    /// * `tag` - The [`Tag`] being processed.
    ///
    /// # Returns
    ///
    /// A `Result` containing `(false, updated_collector, [])`.
    ///
    /// # Errors
    ///
    /// Returns an [`ExecuteError`] if the underlying static policy fails.
    fn execute_tag(
        &self,
        worker: &Worker,
        collector: Collector,
        _document: &str,
        tag: &Tag,
    ) -> Result<(bool, Collector, Vec<(Range, String)>)> {
        let mono_inputs = StaticPolicyMonoInput {
            readonly: false,
            worker,
            collector,
            tag_or_anchor: TagOrAnchor::Tag(tag),
        };
        let mono_result = P::mono(mono_inputs)?;
        Ok((
            mono_result.do_next_pass,
            mono_result.collector,
            mono_result.new_patches,
        ))
    }

    /// Collects a static tag, updating the collector.
    ///
    /// This method delegates to the underlying [`StaticPolicy::collect_static_tag`] and
    /// returns `(false, collector)` as static tags do not trigger new passes.
    ///
    /// # Arguments
    ///
    /// * `worker` - A reference to the [`Worker`] instance.
    /// * `collector` - The current [`Collector`] state.
    /// * `tag` - The [`Tag`] being processed.
    ///
    /// # Returns
    ///
    /// A `Result` containing `(false, updated_collector)`.
    ///
    /// # Errors
    ///
    /// Returns an [`ExecuteError`] if the underlying static policy fails.
    fn collect_tag(
        &self,
        worker: &Worker,
        collector: Collector,
        tag: &Tag,
    ) -> Result<(bool, Collector)> {
        let mono_inputs = StaticPolicyMonoInput {
            readonly: true,
            worker,
            collector,
            tag_or_anchor: TagOrAnchor::Tag(tag),
        };
        let mono_result = P::mono(mono_inputs)?;
        Ok((false, mono_result.collector))
    }
}

/// A wrapper for [`DynamicPolicy`] implementations to conform to the [`TagBehavior`] trait.
///
/// This struct handles the lifecycle of dynamic tags, including state persistence,
/// model calls, and document modifications over multiple execution passes.
pub struct DynamicTagBehavior<P: DynamicPolicy>(P);

impl<P: DynamicPolicy> TagBehavior for DynamicTagBehavior<P> {
    /// Executes a dynamic tag, converting it into an anchor and initiating its lifecycle.
    ///
    /// This method is called when a dynamic tag (e.g., `@answer`) is first encountered
    /// in an `execute` pass. It calls the `mono` method of the underlying dynamic policy
    /// to get an initial result, then transforms the tag into a pair of anchors in the
    /// document and saves the initial state. It always triggers a new pass.
    ///
    /// # Arguments
    ///
    /// * `worker` - A reference to the [`Worker`] instance.
    /// * `collector` - The current [`Collector`] state.
    /// * `tag` - The [`Tag`] being processed.
    ///
    /// # Returns
    ///
    /// A `Result` containing a tuple: `(do_next_pass, updated_collector, patches)`.
    /// `do_next_pass` is always `true` to trigger a new pass. `updated_collector` is the
    /// [`Collector`] after this operation. `patches` are modifications to be applied to the source file.
    ///
    /// # Errors
    ///
    /// Returns an [`ExecuteError`] if state loading/saving fails, input redirection fails,
    /// or the underlying dynamic policy's `mono` method fails.
    fn execute_tag(
        &self,
        worker: &Worker,
        collector: Collector,
        document: &str,
        tag: &Tag,
    ) -> Result<(bool, Collector, Vec<(Range, String)>)> {
        let state: P::State = P::State::default();
        let (input, input_hash) = worker.redirect_input(&collector, &tag.parameters)?;
        let mono_inputs = DynamicPolicyMonoInput::<P::State> {
            readonly: false,
            worker,
            collector,
            document,
            state,
            tag_or_anchor: TagOrAnchor::Tag(tag),
            input,
            input_hash,
        };
        let mono_result = P::mono(mono_inputs)?;
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

    /// Collects a dynamic tag, always triggering a new pass.
    ///
    /// Dynamic tags do not have a direct `collect_tag` behavior in the same way
    /// static tags do. When a dynamic tag is encountered in a `collect` pass,
    /// it signifies that it needs to be converted into an anchor in a subsequent
    /// `execute` pass. Therefore, this method always returns `true` for `do_next_pass`.
    ///
    /// # Arguments
    ///
    /// * `_worker` - A reference to the [`Worker`] instance (unused).
    /// * `collector` - The current [`Collector`] state.
    /// * `_tag` - The [`Tag`] being processed (unused).
    ///
    /// # Returns
    ///
    /// A `Result` containing `(true, collector)`.
    fn collect_tag(
        &self,
        _worker: &Worker,
        collector: Collector,
        _tag: &Tag,
    ) -> Result<(bool, Collector)> {
        // Dynamic tags do not support collect_tag because they always produce a new anchor during execution, then trigger a new pass
        Ok((true, collector))
    }

    /// Executes a dynamic anchor, processing its state and potentially injecting output.
    ///
    /// This method is called when a dynamic anchor (e.g., `<!-- @@answer...@@ -->`)
    /// is encountered in an `execute` pass. It loads the anchor's state, calls the
    /// underlying dynamic policy's `mono` method, and then potentially saves the
    /// updated state and injects new output into the document.
    ///
    /// # Arguments
    ///
    /// * `worker` - A reference to the [`Worker`] instance.
    /// * `collector` - The current [`Collector`] state.
    /// * `anchor` - The [`Anchor`] being processed.
    /// * `anchor_end` - The [`Position`] marking the end of the anchor's content.
    ///
    /// # Returns
    ///
    /// A `Result` containing a tuple: `(do_next_pass, updated_collector, patches)`.
    /// `do_next_pass` indicates if another pass is required. `updated_collector` is the
    /// [`Collector`] after this operation. `patches` are modifications to be applied to the source file.
    ///
    /// # Errors
    ///
    /// Returns an [`ExecuteError`] if state loading/saving fails, input redirection fails,
    /// or the underlying dynamic policy's `mono` method fails.
    fn execute_anchor(
        &self,
        worker: &Worker,
        collector: Collector,
        document: &str,
        anchor_begin: &Anchor,
        anchor_end: &Anchor,
    ) -> Result<(bool, Collector, Vec<(Range, String)>)> {
        let state = match worker.load_state::<P::State>(anchor_begin.command, &anchor_begin.uuid) {
            Ok(state) => state,
            Err(e) => {
                tracing::warn!("Anchor has been corrupted, deactivating it, error {:?}", e);
                return Ok((false, collector, vec![]));
            }
        };
        let (input, input_hash) = worker.redirect_input(&collector, &anchor_begin.parameters)?;
        let mono_inputs = DynamicPolicyMonoInput::<P::State> {
            readonly: false,
            worker,
            collector,
            document,
            state,
            tag_or_anchor: TagOrAnchor::Anchor((anchor_begin, anchor_end)),
            input,
            input_hash,
        };
        let mono_result = P::mono(mono_inputs)?;
        let mut collector = mono_result.collector;
        // If output has been redirected, place output redirected placeholder
        if let Some(_) = worker.is_output_redirected(&anchor_begin.parameters)? {
            collector =
                collector.push_item(ModelContentItem::system(REDIRECTED_OUTPUT_PLACEHOLDER));
        }
        // If there is a new state, save it
        if let Some(new_state) = mono_result.new_state {
            worker.save_state::<P::State>(anchor_begin.command, &anchor_begin.uuid, &new_state, None)?;
        }
        // If there is some output, patch into new anchor
        let patches_2 = if let Some(output) = mono_result.new_output {
            worker.inject_into_anchor(&collector, anchor_begin, &anchor_end, &output)?
        } else {
            vec![]
        };
        // Return collector and patches
        let mut patches = mono_result.new_patches;
        patches.extend(patches_2);
        Ok((mono_result.do_next_pass, collector, patches))
    }

    /// Collects a dynamic anchor, processing its state in a read-only manner.
    ///
    /// This method is called when a dynamic anchor is encountered in a `collect` pass.
    /// It loads the anchor's state, calls the underlying dynamic policy's `mono` method
    /// in `readonly` mode, and potentially saves the updated state. It discards any
    /// patches or new output generated, as file modifications are not allowed in this pass.
    ///
    /// # Arguments
    ///
    /// * `worker` - A reference to the [`Worker`] instance.
    /// * `collector` - The current [`Collector`] state.
    /// * `anchor` - The [`Anchor`] being processed.
    /// * `anchor_end` - The [`Position`] marking the end of the anchor's content.
    ///
    /// # Returns
    ///
    /// A `Result` containing a tuple: `(do_next_pass, updated_collector)`.
    /// `do_next_pass` indicates if another pass is required. `updated_collector` is the
    /// [`Collector`] after this operation.
    ///
    /// # Errors
    ///
    /// Returns an [`ExecuteError`] if state loading/saving fails, input redirection fails,
    /// or the underlying dynamic policy's `mono` method fails.
    fn collect_anchor(
        &self,
        worker: &Worker,
        collector: Collector,
        document: &str,
        anchor_begin: &Anchor,
        anchor_end: &Anchor,
    ) -> Result<(bool, Collector)> {
        let state = worker.load_state::<P::State>(anchor_begin.command, &anchor_begin.uuid)?;
        let (input, input_hash) = worker.redirect_input(&collector, &anchor_begin.parameters)?;
        let mono_inputs = DynamicPolicyMonoInput::<P::State> {
            readonly: true,
            worker,
            collector,
            document,
            state,
            tag_or_anchor: TagOrAnchor::Anchor((anchor_begin, anchor_end)),
            input,
            input_hash,
        };
        let mono_result = P::mono(mono_inputs)?;
        let mut collector = mono_result.collector;
        // If output has been redirected, place output redirected placeholder
        if let Some(_) = worker.is_output_redirected(&anchor_begin.parameters)? {
            collector =
                collector.push_item(ModelContentItem::system(REDIRECTED_OUTPUT_PLACEHOLDER));
        }
        // If there is some patches, just discard them and new state as well as it cannot be applied
        if !mono_result.new_patches.is_empty() {
            tracing::warn!("Warning, anchor produced some patches even on readonly phase.\nAnchor = {:?}\nPatches = {:?}\n", anchor_begin, mono_result.new_patches);
            return Ok((true, collector));
        }
        // If there is new output, just discard it and new state as well as it cannot be injected
        if let Some(output) = mono_result.new_output {
            tracing::warn!("Warning, anchor produced some output even on readonly phase.\nAnchor = {:?}\nOutput = {:?}\n", anchor_begin, output);
            return Ok((true, collector));
        };
        // If there is a new state, save it
        if let Some(new_state) = mono_result.new_state {
            worker.save_state::<P::State>(anchor_begin.command, &anchor_begin.uuid, &new_state, None)?;
        }
        // Return collector
        Ok((mono_result.do_next_pass, collector))
    }
}

/// Dispatches tag and anchor processing to the appropriate behavior implementation.
///
/// This struct acts as a factory, providing the correct [`TagBehavior`] instance
/// based on the [`CommandKind`] of the tag or anchor. This allows for a flexible
/// and extensible way to add new command types to the execution engine.
pub(crate) struct TagBehaviorDispatch;

impl TagBehaviorDispatch {
    /// Retrieves the appropriate [`TagBehavior`] implementation for a given [`CommandKind`].
    ///
    /// # Arguments
    ///
    /// * `command` - The [`CommandKind`] for which to retrieve the behavior.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Box<dyn TagBehavior>` for the specified command.
    ///
    /// # Errors
    ///
    /// Returns [`ExecuteError::UnsupportedCommand`] if the `command` is not recognized.
    fn get_behavior(command: CommandKind) -> Result<Box<dyn TagBehavior>> {
        match command {
            CommandKind::Answer => Ok(Box::new(DynamicTagBehavior(AnswerPolicy))),
            CommandKind::Repeat => Ok(Box::new(DynamicTagBehavior(RepeatPolicy))),
            CommandKind::Include => Ok(Box::new(StaticTagBehavior(IncludePolicy))),
            CommandKind::Inline => Ok(Box::new(DynamicTagBehavior(InlinePolicy))),
            CommandKind::Set => Ok(Box::new(StaticTagBehavior(SetPolicy))),
            CommandKind::Forget => Ok(Box::new(StaticTagBehavior(ForgetPolicy))),
            CommandKind::Comment => Ok(Box::new(StaticTagBehavior(CommentPolicy))),
            CommandKind::Task => Ok(Box::new(DynamicTagBehavior(TaskPolicy))),
            CommandKind::Done => Ok(Box::new(StaticTagBehavior(DonePolicy))),
            _ => Err(ExecuteError::UnsupportedCommand(command)),
        }
    }

    /// Dispatches the `execute_tag` call to the correct [`TagBehavior`] implementation.
    ///
    /// # Arguments
    ///
    /// * `worker` - A reference to the [`Worker`] instance.
    /// * `collector` - The current [`Collector`] state.
    /// * `tag` - The [`Tag`] to execute.
    ///
    /// # Returns
    ///
    /// A `Result` containing `(do_next_pass, updated_collector, patches)` from the executed tag behavior.
    ///
    /// # Errors
    ///
    /// Returns an [`ExecuteError`] if the command is unsupported or the tag execution fails.
    pub fn execute_tag(
        worker: &Worker,
        collector: Collector,
        document: &str,
        tag: &Tag,
    ) -> Result<(bool, Collector, Vec<(Range, String)>)> {
        let behavior = Self::get_behavior(tag.command)?;
        behavior.execute_tag(worker, collector, document, tag)
    }

    /// Dispatches the `collect_tag` call to the correct [`TagBehavior`] implementation.
    ///
    /// # Arguments
    ///
    /// * `worker` - A reference to the [`Worker`] instance.
    /// * `collector` - The current [`Collector`] state.
    /// * `tag` - The [`Tag`] to collect.
    ///
    /// # Returns
    ///
    /// A `Result` containing `(do_next_pass, updated_collector)` from the collected tag behavior.
    ///
    /// # Errors
    ///
    /// Returns an [`ExecuteError`] if the command is unsupported or the tag collection fails.
    pub fn collect_tag(
        worker: &Worker,
        collector: Collector,
        tag: &Tag,
    ) -> Result<(bool, Collector)> {
        let behavior = Self::get_behavior(tag.command)?;
        behavior.collect_tag(worker, collector, tag)
    }

    /// Dispatches the `execute_anchor` call to the correct [`TagBehavior`] implementation.
    ///
    /// # Arguments
    ///
    /// * `worker` - A reference to the [`Worker`] instance.
    /// * `collector` - The current [`Collector`] state.
    /// * `anchor` - The [`Anchor`] to execute.
    /// * `anchor_end` - The [`Position`] marking the end of the anchor's content.
    ///
    /// # Returns
    ///
    /// A `Result` containing `(do_next_pass, updated_collector, patches)` from the executed anchor behavior.
    ///
    /// # Errors
    ///
    /// Returns an [`ExecuteError`] if the command is unsupported or the anchor execution fails.
    pub fn execute_anchor(
        worker: &Worker,
        collector: Collector,
        document: &str,
        anchor_begin: &Anchor,
        anchor_end: &Anchor,
    ) -> Result<(bool, Collector, Vec<(Range, String)>)> {
        let behavior = Self::get_behavior(anchor_begin.command)?;
        behavior.execute_anchor(worker, collector, document, anchor_begin, anchor_end)
    }

    /// Dispatches the `collect_anchor` call to the correct [`TagBehavior`] implementation.
    ///
    /// # Arguments
    ///
    /// * `worker` - A reference to the [`Worker`] instance.
    /// * `collector` - The current [`Collector`] state.
    /// * `anchor` - The [`Anchor`] to collect.
    /// * `anchor_end` - The [`Position`] marking the end of the anchor's content.
    ///
    /// # Returns
    ///
    /// A `Result` containing `(do_next_pass, updated_collector)` from the collected anchor behavior.
    ///
    /// # Errors
    ///
    /// Returns an [`ExecuteError`] if the command is unsupported or the anchor collection fails.
    pub fn collect_anchor(
        worker: &Worker,
        collector: Collector,
        document: &str,
        anchor_begin: &Anchor,
        anchor_end: &Anchor,
    ) -> Result<(bool, Collector)> {
        let behavior = Self::get_behavior(anchor_begin.command)?;
        behavior.collect_anchor(worker, collector, document, anchor_begin, anchor_end)
    }
}
