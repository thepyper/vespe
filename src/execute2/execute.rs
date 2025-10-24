use crate::ast2::{
    Content, Anchor, AnchorKind, Arguments, CommandKind, Document, Parameters, Range, Tag, Text,
};
use crate::path;
use crate::utils;
use crate::{agent, file};
use anyhow::Result;
use std::collections::{self, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use std::ops::Drop;

use crate::file::FileAccessor;
use crate::path::PathResolver;

use super::*;

use crate::execute2::content::ModelContentItem;
use crate::execute2::state::{AnchorStatus, AnswerState, DeriveState, InlineState};
use crate::execute2::variables::Variables;

/// A RAII guard to ensure a file lock is released.
struct FileLock {
    file_access: Arc<dyn file::FileAccessor>,
    lock_id: Option<Uuid>,
}

impl FileLock {
    /// Creates a new `FileLock`, acquiring a lock on the given path.
    fn new(file_access: Arc<dyn file::FileAccessor>, path: &Path) -> Result<Self> {
        let lock_id = file_access.lock_file(path)?;
        Ok(Self {
            file_access,
            lock_id: Some(lock_id),
        })
    }
}

impl Drop for FileLock {
    /// Releases the file lock when the `FileLock` goes out of scope.
    fn drop(&mut self) {
        if let Some(lock_id) = self.lock_id.take() {
            if let Err(e) = self.file_access.unlock_file(&lock_id) {
                tracing::error!("Failed to unlock file with id {}: {}", lock_id, e);
            }
        }
    }
}

/// Executes a context and all its dependencies, processing all commands.
///
/// This function orchestrates the full, multi-pass execution of a context file.
/// It will resolve includes, execute model calls (`@answer`, `@derive`), and inject
/// the results back into the document by modifying the source files.
///
/// # Arguments
/// * `file_access` - A thread-safe file accessor.
/// * `path_res` - A thread-safe path resolver.
/// * `context_name` - The name of the root context to execute.
///
/// # Returns
/// The final, collected `ModelContent` after full execution.
pub fn execute_context(
    file_access: Arc<dyn file::FileAccessor>,
    path_res: Arc<dyn path::PathResolver>,
    context_name: &str,
) -> Result<ModelContent> {
    tracing::debug!("Executing context: {}", context_name);

    let exe = Worker::new(file_access, path_res);
    let collector = exe.execute(Collector::new(true), context_name)?;

    tracing::debug!("Finished executing context: {}", context_name);
    Ok(collector.context)
}

/// Collects a context's content without executing any commands that modify state or call models.
///
/// This function performs a single pass to gather all text content, resolving `@include`
/// tags, but it will not process `@answer`, `@derive`, or other state-changing anchors.
/// It is a 'read-only' version of `execute_context`.
///
/// # Arguments
/// * `file_access` - A thread-safe file accessor.
/// * `path_res` - A thread-safe path resolver.
/// * `context_name` - The name of the root context to collect.
///
/// # Returns
/// The collected `ModelContent`.
pub fn collect_context(
    file_access: Arc<dyn file::FileAccessor>,
    path_res: Arc<dyn path::PathResolver>,
    context_name: &str,
) -> Result<ModelContent> {
    tracing::debug!("Collecting context: {}", context_name);

    let exe = Worker::new(file_access, path_res);
    let collector = exe.collect(Collector::new(false), context_name)?;

    tracing::debug!("Finished collecting context: {}", context_name);
    Ok(collector.context)
}

/// Central state object for the execution engine.
///
/// The `Collector` accumulates the final `ModelContent` and tracks execution-time
/// variables and the call stack to prevent infinite recursion.
/// It is passed by value through the execution flow, ensuring a functional-style,
/// predictable state management.
#[derive(Clone)]
struct Collector {
    /// A stack of visited context file paths to detect and prevent circular includes.
    visit_stack: Vec<PathBuf>,
    /// A stack of entered anchors in current context file
    anchor_stack: Vec<Anchor>,
    /// The accumulated content that will be sent to the model.
    context: ModelContent,
    /// Execution-time variables and settings.
    variables: Variables,
    /// A flag indicating whether the engine is in `execute` (true) or `collect` (false) mode.
    can_execute: bool,
}

impl Collector {
    /// Creates a new, empty `Collector`.
    ///
    /// # Arguments
    /// * `can_execute` - Sets the execution mode. `true` for full execution, `false` for collect-only.
    fn new(can_execute: bool) -> Self {
        Collector {
            visit_stack: Vec::new(),
            anchor_stack: Vec::new(),
            context: ModelContent::new(),
            variables: Variables::new(),
            can_execute,
        }
    }

    /// Prepares the collector for descending into a new context file.
    ///
    /// It checks for circular dependencies. If the path is already in the `visit_stack`,
    /// it returns `None`. Otherwise, it returns a new `Collector` with the given path
    /// added to its stack and a cleared context, ready for the new file.
    fn descent(&self, context_path: &Path) -> Option<Self> {
        if self.visit_stack.contains(&context_path.to_path_buf()) {
            return None;
        }

        let mut visit_stack = self.visit_stack.clone();
        visit_stack.push(context_path.to_path_buf());
        Some(Collector {
            visit_stack,
            anchor_stack: Vec::new(),
            context: ModelContent::new(),
            variables: self.variables.clone(),
            can_execute: self.can_execute,
        })
    }

    /// Creates a new `Collector` with updated variables values, taken from Parameters
    ///
    /// # Arguments
    /// * `parameters` - Parameters to take variable values from
    fn update(&self, parameters: &Parameters) -> Self {
        let mut collector = self.clone();
        collector.variables = collector.variables.update(parameters);
        collector
    }
 
    // TODO doc (entra in anchor)
    fn enter(&self, anchor: &Anchor) -> Self {
        let mut collector = self.clone();
        collector.anchor_stack.push(anchor.clone());
        collector
    }

    // TODO doc (esci da anchor)
    fn exit(&self) -> Result<Self> {
        let mut collector = self.clone();
        collector.anchor_stack.pop().ok_or_else(|| anyhow::anyhow!("Pop on empty stack!?"))?;
        Ok(collector)
    }
}

/// The stateless engine that drives the context execution.
///
/// The `Worker` holds thread-safe handles to the tools needed for execution,
/// such as the file accessor and path resolver. It contains the core logic
/// for the multi-pass execution strategy.
struct Worker {
    file_access: Arc<dyn file::FileAccessor>,
    path_res: Arc<dyn path::PathResolver>,
}

impl Worker {
    /// Creates a new `Worker` with the necessary tools.
    fn new(
        file_access: Arc<dyn file::FileAccessor>,
        path_res: Arc<dyn path::PathResolver>,
    ) -> Self {
        Worker {
            file_access,
            path_res,
        }
    }

    /// Collects context from a file, resolving includes but not executing stateful anchors.
    /// This represents the entry point for a read-only collection pass.
    fn collect(&self, collector: Collector, context_name: &str) -> Result<Collector> {
        tracing::debug!("Worker::collect for context: {}", context_name);
        let context_path = self.path_res.resolve_context(context_name)?;

        match collector.descent(&context_path) {
            None => {
                tracing::debug!(
                    "Context {} already in visit stack, skipping collection.",
                    context_name
                );
                return Ok(collector);
            }
            Some(collector) => {
                // Read file, parse it, collect without allowing execution
                match self.pass_1(collector, &context_path)? {
                    Some(collector) => {
                        // Successfully collected everything without needing further processing
                        tracing::debug!(
                            "Worker::execute_step collected from pass_1 for path: {:?}",
                            context_path
                        );
                        return Ok(collector);
                    }
                    None => {
                        return Err(anyhow::anyhow!(
                            "Collection incomplete, execution needed for context: {}",
                            context_name
                        ));
                    }
                };
            }
        }
    }

    /// Executes a context fully, running a loop of `execute_step` until the state converges.
    ///
    /// This is the main entry point for a full execution, which can modify files.
    /// It orchestrates the two-pass strategy until all anchors are `Completed`.
    fn execute(&self, collector: Collector, context_name: &str) -> Result<Collector> {
        tracing::debug!("Worker::execute for context: {}", context_name);
        let context_path = self.path_res.resolve_context(context_name)?;

        match collector.descent(&context_path) {
            None => {
                tracing::debug!(
                    "Context {} already in visit stack, skipping execution.",
                    context_name
                );
                return Ok(collector);
            }
            Some(collector) => {
                let mut collector = collector;
                loop {
                    if let Some(new_collector) =
                        self.execute_step(collector.clone(), &context_path)?
                    {
                        tracing::debug!(
                            "Worker::execute returning collected for context: {}",
                            context_name
                        );
                        return Ok(new_collector);
                    }
                    tracing::debug!("Worker::execute continuing for context: {}", context_name);
                }
            }
        }
    }

    /// Executes a single step of the two-pass strategy.
    ///
    /// 1.  **Pass 2**: Scans the document for anchors that are ready for injection (`NeedInjection`)
    ///     and patches the file on disk. This is a fast, synchronous operation.
    /// 2.  **Pass 1**: Re-reads the document and collects content. If it encounters anchors
    ///     that need processing (`JustCreated` or `NeedProcessing`), it triggers the slow
    ///     tasks (like model calls) and returns `Ok(None)` to signal that the `execute`
    ///     loop needs to run again.
    ///
    /// If `pass_1` completes without starting any new tasks, it returns `Ok(Some(Collector))`,
    /// signaling that the execution has converged and is complete.
    fn execute_step(&self, collector: Collector, context_path: &Path) -> Result<Option<Collector>> {
        tracing::debug!("Worker::execute_step for path: {:?}", context_path);

        // Lock file, read it (could be edited outside), parse it, execute fast things that may modify context and save it
        match self.pass_2(context_path)? {
            true => {
                tracing::debug!(
                    "Worker::execute_step pass_2 needs another pass for path: {:?}",
                    context_path
                );
                return Ok(None);
            }
            false => {
                tracing::debug!(
                    "Worker::execute_step no changes in pass_2 for path: {:?}",
                    context_path
                );
            }
        }

        // Re-read file, parse it, execute slow things that do not modify context, collect data
        match self.pass_1(collector, context_path)? {
            Some(collector) => {
                // Successfully collected everything without needing further processing
                tracing::debug!(
                    "Worker::execute_step collected from pass_1 for path: {:?}",
                    context_path
                );
                return Ok(Some(collector));
            }
            None => {
                // Could not collect everything in pass_1, need to trigger another step
                tracing::debug!(
                    "Worker::execute_step could not collect from pass_1 for path: {:?}",
                    context_path
                );
                return Ok(None);
            }
        };
    }

    fn call_model(&self, collector: &Collector, contents: Vec<ModelContent>) -> Result<String> {
        let query = contents
            .into_iter()
            .flatten()
            .map(|item| item.to_string())
            .collect::<Vec<String>>()
            .join("\n");
        agent::shell::shell_call(&collector.variables.provider, &query)
    }

    /// **Pass 1**: Collects content and triggers slow/asynchronous tasks.
    ///
    /// This pass reads the AST and builds up the `ModelContent` by processing
    /// text and tags. When it encounters an anchor that is in a state that requires
    /// processing (e.g., `JustCreated`), it initiates the required action (like a
    /// model call) and returns `Ok(None)`. This signals to the `execute_step` loop
    /// that a long-running task has started and a new cycle will be needed later.
    ///
    /// If the entire document is parsed without initiating any new tasks, it returns
    /// `Ok(Some(Collector))`, indicating this pass is complete and the state is stable.
    fn pass_1(&self, collector: Collector, context_path: &Path) -> Result<Option<Collector>> {
        tracing::debug!("Worker::pass_1 for path: {:?}", context_path,);
        let context_content = self.file_access.read_file(context_path)?;
        let ast = crate::ast2::parse_document(&context_content)?;

        let mut current_collector = collector;

        for item in &ast.content {
            match item {
                crate::ast2::Content::Text(text) => {
                    current_collector = self.pass_1_text(current_collector, text)?;
                }
                crate::ast2::Content::Tag(tag) => {
                    current_collector = self.pass_1_tag(current_collector, tag)?;
                }
                crate::ast2::Content::Anchor(anchor) => {
                    match self.pass_1_anchor(current_collector, anchor)? {
                        Some(c) => current_collector = c,
                        None => {
                            tracing::debug!(
                                "Worker::pass_1 returning None after processing anchor for path: {:?}",
                                context_path
                            );
                            return Ok(None);
                        }
                    }
                }
            }
        }
        tracing::debug!("Worker::pass_1 finished for path: {:?}", context_path);
        Ok(Some(current_collector))
    }

    fn pass_1_text(&self, mut collector: Collector, text: &Text) -> Result<Collector> {
        collector
            .context
            .push(ModelContentItem::user(&text.content));
        Ok(collector)
    }

    /// Processes tags that do not modify context, such as `@include`.
    fn pass_1_tag(&self, collector: Collector, tag: &Tag) -> Result<Collector> {
        match tag.command {
            CommandKind::Include => self.pass_1_include_tag(collector, tag),
            CommandKind::Set => self.pass1_set_tag(collector, tag),
            _ => Ok(collector),
        }
    }

    /// Handles the `@include` tag by recursively calling the main `collect` method.
    fn pass_1_include_tag(&self, collector: Collector, tag: &Tag) -> Result<Collector> {
        let included_context_name = tag
            .arguments
            .arguments
            .get(0)
            .ok_or_else(|| anyhow::anyhow!("Missing argument for include tag"))?
            .value
            .clone();
        self.collect(collector, &included_context_name)
    }

     /// Handles the `@set` tag by updating collector's variabiles
    fn pass1_set_tag(&self, collector: Collector, tag: &Tag) -> Result<Collector> {        
        Ok(collector.update(&tag.parameters))
    }

    /// Processes anchors and dispatches them to the appropriate handler based on their command.
    /// This is the entry point for triggering the state machines for `@answer`, `@derive`, etc.
    fn pass_1_anchor(&self, collector: Collector, anchor: &Anchor) -> Result<Option<Collector>> {
        tracing::debug!(
            "Worker::pass_1_anchor processing command: {:?}, kind: {:?}",
            anchor.command,
            anchor.kind
        );
        let asm =
            utils::AnchorStateManager::new(self.file_access.clone(), self.path_res.clone(), anchor);
        let result = match (&anchor.command, &anchor.kind) {
            (CommandKind::Answer, AnchorKind::Begin) => {
                tracing::debug!("Worker::pass_1_anchor calling pass_1_answer_begin_anchor");
                self.pass_1_answer_begin_anchor(
                    collector,
                    &asm,
                    anchor,
                )
            }
            (CommandKind::Derive, AnchorKind::Begin) => {
                tracing::debug!("Worker::pass_1_anchor calling pass_1_derive_begin_anchor");
                self.pass_1_derive_begin_anchor(
                    collector,
                    &asm,
                    anchor,
                )
            }
            (CommandKind::Inline, AnchorKind::Begin) => {
                tracing::debug!("Worker::pass_1_anchor calling pass_1_inline_begin_anchor");
                self.pass_1_inline_begin_anchor(
                    collector,
                    &asm,
                    anchor,
                )
            }
            (CommandKind::Repeat, AnchorKind::Begin) => {
                tracing::debug!("Worker::pass_1_anchor calling pass_1_repeat_begin_anchor");
                self.pass_1_repeat_begin_anchor(
                    collector,
                    &asm,
                    anchor,
                )
            }
            _ => {
                tracing::debug!(
                    "Worker::pass_1_anchor not handling command: {:?}, kind: {:?}",
                    anchor.command,
                    anchor.kind
                );
                Ok(Some(collector))
            }
        };
        match result {
            Ok(maybe_collector) => {
                match maybe_collector {
                    Some(collector) => {
                        // Update collector anchor stack
                        let collector = match anchor.kind {
                            AnchorKind::Begin => collector.enter(&anchor),
                            AnchorKind::End => collector.exit()?,
                        };
                        return Ok(Some(collector));
                    }
                    None => {
                        return Ok(None);
                    }
                }
            }
            Err(x) => {
                return Err(x);
            }
        }
    }

    fn pass_1_answer_begin_anchor(
        &self,
        collector: Collector,
        asm: &utils::AnchorStateManager,
        anchor: &Anchor,
    ) -> Result<Option<Collector>> {
        let mut state: AnswerState = asm.load_state()?;
        match state.status {
            AnchorStatus::JustCreated => {
                if !collector.can_execute {
                    return Err(anyhow::anyhow!("Execution not allowed"));
                }
                state.query = collector.context.clone();
                state.status = AnchorStatus::NeedProcessing;
                asm.save_state(&state, None)?;
                Ok(None)
            }
            AnchorStatus::NeedProcessing => {
                if !collector.can_execute {
                    return Err(anyhow::anyhow!("Execution not allowed"));
                }
                // Update variables locally for this answer
                let collector = collector.update(&anchor.parameters);
                state.reply = self.call_model(&collector, vec![state.query.clone()])?;
                state.status = AnchorStatus::NeedInjection;
                asm.save_state(&state, None)?;
                Ok(None)
            }
            _ => Ok(Some(collector)),
        }
    }

    fn pass_1_derive_begin_anchor(
        &self,
        collector: Collector,
        asm: &utils::AnchorStateManager,
        anchor: &Anchor,
    ) -> Result<Option<Collector>> {
        let mut state: DeriveState = asm.load_state()?;
        match state.status {
            AnchorStatus::JustCreated => {
                if !collector.can_execute {
                    return Err(anyhow::anyhow!("Execution not allowed"));
                }
                state.instruction_context_name = anchor.arguments
                    .arguments
                    .get(0)
                    .ok_or_else(|| anyhow::anyhow!("Missing instruction context name"))?
                    .value
                    .clone();
                state.input_context_name = anchor.arguments
                    .arguments
                    .get(1)
                    .ok_or_else(|| anyhow::anyhow!("Missing input context name"))?
                    .value
                    .clone();
                state.status = AnchorStatus::NeedProcessing;
                asm.save_state(&state, None)?;
                Ok(None)
            }
            AnchorStatus::NeedProcessing => {
                if !collector.can_execute {
                    return Err(anyhow::anyhow!("Execution not allowed"));
                }
                state.instruction_context = collect_context(
                    self.file_access.clone(),
                    self.path_res.clone(),
                    &state.instruction_context_name,
                )?;
                state.input_context = collect_context(
                    self.file_access.clone(),
                    self.path_res.clone(),
                    &state.input_context_name,
                )?;
                // Update variables locally for this derive
                let collector = collector.update(&anchor.parameters);
                // call llm to derive
                state.derived = "rispostone!! TODO ".into();
                state.status = AnchorStatus::NeedInjection;
                asm.save_state(&state, None)?;
                Ok(None)
            }
            _ => Ok(Some(collector)),
        }
    }

    fn pass_1_inline_begin_anchor(
        &self,
        collector: Collector,
        asm: &utils::AnchorStateManager,
        anchor: &Anchor,
    ) -> Result<Option<Collector>> {
        let mut state: InlineState = asm.load_state()?;
        match state.status {
            AnchorStatus::JustCreated => {
                if !collector.can_execute {
                    return Err(anyhow::anyhow!("Execution not allowed"));
                }
                state.context_name = anchor.arguments
                    .arguments
                    .get(0)
                    .ok_or_else(|| anyhow::anyhow!("Missing context name"))?
                    .value
                    .clone();
                state.status = AnchorStatus::NeedProcessing;
                asm.save_state(&state, None)?;
                Ok(None)
            }
            AnchorStatus::NeedProcessing => {
                if !collector.can_execute {
                    return Err(anyhow::anyhow!("Execution not allowed"));
                }
                let context_path = self.path_res.resolve_context(&state.context_name)?;
                state.context = self.file_access.read_file(&context_path)?;
                state.status = AnchorStatus::NeedInjection;
                asm.save_state(&state, None)?;
                Ok(None)
            }
            _ => Ok(Some(collector)),
        }
    }

    fn pass_1_repeat_begin_anchor(
        &self,
        collector: Collector,
        asm: &utils::AnchorStateManager,
        anchor: &Anchor,
    ) -> Result<Option<Collector>> {
        let mut state: RepeatState = asm.load_state()?;
        match state.status {
            AnchorStatus::JustCreated => {
                if !collector.can_execute {
                    return Err(anyhow::anyhow!("Execution not allowed"));
                }
                if let Some(x) = collector.anchor_stack.last() {
                    // Mutate wrapper anchor
                    state.wrapper = x.uuid;
                } else {
                    return Err(anyhow::anyhow!("@repeat not wrapped by an existing anchor"));
                } 
                
                state.status = AnchorStatus::NeedInjection;
                asm.save_state(&state, None)?;
                Ok(None)
            }
            _ => Ok(Some(collector)),
        }
    }

    /// **Pass 2**: Injects completed content into files.
    ///
    /// This pass is responsible for the fast, synchronous part of the execution.
    /// It acquires a lock on the context file, scans the AST for anchors whose state
    /// is `NeedInjection`, and applies the generated content from their state objects
    /// as patches to the file. It also handles the initial creation of anchor pairs for
    /// simple tags (e.g., turning `@answer` into a full anchor block).
    ///
    /// # Returns
    ///
    /// Returns `Ok(true)` if any patches were applied, signaling that the document
    /// has changed. Returns `Ok(false)` if no changes were made.
    fn pass_2(&self, context_path: &Path) -> Result<bool> {
        tracing::debug!("Worker::pass_2 for path: {:?}", context_path);
        let _lock = FileLock::new(self.file_access.clone(), context_path)?;

        let context_content = self.file_access.read_file(context_path)?;
        let ast = crate::ast2::parse_document(&context_content)?;
        let mut patches = utils::Patches::new(&context_content);

        let result = self.pass_2_internal_y(&mut patches, &ast)?;

        if !patches.is_empty() {
            let new_context_content = patches.apply_patches()?;
            self.file_access
                .write_file(context_path, &new_context_content, None)?;
        }

        tracing::debug!("Worker::pass_2 finished for path: {:?}", context_path);
        Ok(result)
    }

    /// Scans the document AST to find and apply patches.
    fn pass_2_internal_y(&self, patches: &mut utils::Patches, ast: &Document) -> Result<bool> {
        let anchor_index = utils::AnchorIndex::new(&ast.content);

        for item in &ast.content {
            match item {
                crate::ast2::Content::Tag(tag) => {
                    if self.pass_2_tag(patches, tag)? {
                        return Ok(true);
                    }
                }
                crate::ast2::Content::Anchor(a0) => match a0.kind {
                    AnchorKind::Begin => {
                        let j = anchor_index
                            .get_end(&a0.uuid)
                            .ok_or_else(|| anyhow::anyhow!("Anchor not closed!"))?;
                        let a1 = ast
                            .content
                            .get(j)
                            .ok_or_else(|| anyhow::anyhow!("Bad index!?!?"))?;
                        match a1 {
                            crate::ast2::Content::Anchor(a1) => match a1.kind {
                                AnchorKind::End => {
                                    if self.pass_2_anchors(patches, ast, &anchor_index, a0, a1)? {
                                        return Ok(true);
                                    }
                                }
                                _ => return Err(anyhow::anyhow!("Bad end anchor!")),
                            },
                            _ => return Err(anyhow::anyhow!("Bad end content!")),
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        Ok(false)
    }

    fn pass_2_tag(&self, patches: &mut utils::Patches, tag: &Tag) -> Result<bool> {
        match tag.command {
            CommandKind::Answer => self.pass_2_normal_tag::<AnswerState>(
                patches,
                tag,
            ),
            CommandKind::Derive => self.pass_2_normal_tag::<DeriveState>(
                patches,
                tag,
            ),
            CommandKind::Inline => self.pass_2_normal_tag::<InlineState>(
                patches,
                tag,
            ),
            CommandKind::Repeat => self.pass_2_normal_tag::<RepeatState>(
                patches,
                tag,
            ),
            _ => Ok(false),
        }
    }

    fn pass_2_normal_tag<S: State + 'static + serde::Serialize>(
        &self,
        patches: &mut utils::Patches,
        tag: &Tag,
    ) -> Result<bool> {
        let (a0, a1) = Anchor::new_couple(tag.command, &tag.parameters, &tag.arguments);
        patches.add_patch(&tag.range, &format!("{}\n{}\n", a0.to_string(), a1.to_string()));
        let asm =
            utils::AnchorStateManager::new(self.file_access.clone(), self.path_res.clone(), &a0);
        asm.save_state(&S::new(), None)?;
        Ok(true)
    }

    fn pass_2_anchors(
        &self,
        patches: &mut utils::Patches,
        ast: &Document,
        anchor_index: &utils::AnchorIndex,
        a0: &Anchor,
        a1: &Anchor,
    ) -> Result<bool> {
        tracing::debug!(
            "Worker::pass_2_anchors processing command: {:?}",
            a0.command
        );
        let asm =
            utils::AnchorStateManager::new(self.file_access.clone(), self.path_res.clone(), a0);
        match a0.command {
            CommandKind::Answer => {
                tracing::debug!(
                    "Worker::pass_2_anchors calling pass_2_normal_begin_anchor for Answer"
                );
                self.pass_2_normal_begin_anchor::<AnswerState>(
                    patches,
                    &asm,
                    a0,
                    a1,
                )
            }
            CommandKind::Derive => {
                tracing::debug!(
                    "Worker::pass_2_anchors calling pass_2_normal_begin_anchor for Derive"
                );
                self.pass_2_normal_begin_anchor::<DeriveState>(
                    patches,
                    &asm,
                    a0,
                    a1,
                )
            }
            CommandKind::Inline => {
                tracing::debug!(
                    "Worker::pass_2_anchors calling pass_2_normal_begin_anchor for Inline"
                );
                self.pass_2_normal_begin_anchor::<InlineState>(
                    patches,
                    &asm,
                    a0,
                    a1,
                )
            }           
            CommandKind::Repeat => {
                tracing::debug!(
                    "Worker::pass_2_anchors calling pass_2_normal_begin_anchor for Repeat"
                );
                self.pass_2_repeat_begin_anchor(
                    patches,
                    &asm,
                    ast,
                    anchor_index,
                    a0,
                    a1,
                )
            }
            _ => {
                tracing::debug!(
                    "Worker::pass_2_anchors not handling command: {:?}",
                    a0.command
                );
                Ok(false)
            }
        }
    }

    fn pass_2_normal_begin_anchor<S: State + 'static>(
        &self,
        patches: &mut utils::Patches,
        asm: &utils::AnchorStateManager,
        a0: &Anchor, 
        a1: &Anchor,
    ) -> Result<bool> {
        let mut state: S = asm.load_state()?;
        match state.get_status() {
            AnchorStatus::NeedRepeat => {
                let range = Range {
                    begin: a0.range.end.clone(),
                    end: a1.range.begin.clone(),
                };
                patches.add_patch(&range, "");
                state.set_status(AnchorStatus::JustCreated);
                asm.save_state(&state, None)?;
                Ok(true)
            }
            AnchorStatus::NeedInjection => {
                let range = Range {
                    begin: a0.range.end.clone(),
                    end: a1.range.begin.clone(),
                };
                patches.add_patch(&range, &state.output());
                state.set_status(AnchorStatus::Completed);
                asm.save_state(&state, None)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }   

    fn pass_2_repeat_begin_anchor(
        &self,
        patches: &mut utils::Patches,
        asm: &utils::AnchorStateManager,
        ast: &Document,
        anchor_index: &utils::AnchorIndex,
        a0: &Anchor, 
        a1: &Anchor,
    ) -> Result<bool> {
        let mut state: RepeatState = asm.load_state()?;
        match state.status {
            AnchorStatus::NeedInjection => {
                let wrapper = anchor_index.get_begin(&state.wrapper)
                    .ok_or_else(|| anyhow::anyhow!("@repeat wrapper begin anchor not found"))?;
                let wrapper = ast.content
                    .get(wrapper)
                    .ok_or_else(|| anyhow::anyhow!("@repeat wrapper begin anchor bad index"))?;
                let Content::Anchor(wrapper) = wrapper else {
                    return Err(anyhow::anyhow!("@repeat wrapper begin anchor bad content"));
                };
                let wrapper_mutated = wrapper.update(&a0.parameters);
                patches.add_patch(&wrapper_mutated.range, &format!("{}\n", &wrapper_mutated.to_string()));
                // Update wrapper's status
                match wrapper.command {
                    CommandKind::Answer => {
                        self.pass_2_set_anchor_to_repeat_state::<AnswerState>(&wrapper)?;
                    }
                    CommandKind::Derive => {
                        self.pass_2_set_anchor_to_repeat_state::<DeriveState>(&wrapper)?;
                    }
                    CommandKind::Inline => {
                        self.pass_2_set_anchor_to_repeat_state::<InlineState>(&wrapper)?;
                    }
                    _ => {
                        return Err(anyhow::anyhow!("@repeat is wrapped by non-repeatable anchor"));
                    }
                }
                // Mark repeat as completed
                state.status = AnchorStatus::Completed;
                asm.save_state(&state, None)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    } 
    
    fn pass_2_set_anchor_to_repeat_state<S: State + 'static>(
        &self,
        anchor: &Anchor,
    ) -> Result<()> {
        let asm =
            utils::AnchorStateManager::new(self.file_access.clone(), self.path_res.clone(), anchor);
        let mut state: S = asm.load_state()?;
        state.set_status(AnchorStatus::NeedRepeat);
        asm.save_state(&state, None)?;
        Ok(())
    }
}
