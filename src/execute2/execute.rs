use crate::ast2::{
    Anchor, AnchorKind, Arguments, CommandKind, Document, Parameters, Range, Tag, Text,
};
use crate::path;
use crate::utils;
use crate::{agent, file};
use anyhow::Result;
use std::collections::{self, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::file::FileAccessor;
use crate::path::PathResolver;

use super::*;

use crate::execute2::content::ModelContentItem;
use crate::execute2::state::{AnchorStatus, AnswerState, DeriveState, InlineState};
use crate::execute2::variables::Variables;

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

#[derive(Clone)]
struct Collector {
    visit_stack: Vec<PathBuf>,
    context: ModelContent,
    variables: Variables,
    can_execute: bool,
}

impl Collector {
    fn new(can_execute: bool) -> Self {
        Collector {
            visit_stack: Vec::new(),
            context: ModelContent::new(),
            variables: Variables::new(),
            can_execute,
        }
    }
    fn descent(&self, context_path: &Path) -> Option<Self> {
        if self.visit_stack.contains(&context_path.to_path_buf()) {
            return None;
        }

        let mut visit_stack = self.visit_stack.clone();
        visit_stack.push(context_path.to_path_buf());
        Some(Collector {
            visit_stack,
            context: ModelContent::new(),
            variables: self.variables.clone(),
            can_execute: self.can_execute,
        })
    }
}

struct Worker {
    file_access: Arc<dyn file::FileAccessor>,
    path_res: Arc<dyn path::PathResolver>,
}

impl Worker {
    fn new(
        file_access: Arc<dyn file::FileAccessor>,
        path_res: Arc<dyn path::PathResolver>,
    ) -> Self {
        Worker {
            file_access,
            path_res,
        }
    }

    /// Collect context without executing anchors that modify state, can be executed only on final contexts
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

    /// Execute context, processing anchors that modify state as needed and modifying context accordingly until completion
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

    /// Execute a single step: pass_2 to modify context as needed, then pass_1 to collect data.
    /// If pass_1 can collect everything without requiring further passes, returns Some(Collector),
    /// else returns None to signal need for further processing.
    fn execute_step(&self, collector: Collector, context_path: &Path) -> Result<Option<Collector>> {
        tracing::debug!("Worker::execute_step for path: {:?}", context_path);

        // Lock file, read it (could be edited outside), parse it, execute fast things that may modify context and save it
        self.pass_2(context_path)?;

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
                // Could not collect everything in pass_1, need to do pass_2 to modify context and trigger another pass_1
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

    /// Process tags that do NOT modify context, so tags that do NOT spawn anchors
    fn pass_1_tag(&self, collector: Collector, tag: &Tag) -> Result<Collector> {
        match tag.command {
            CommandKind::Include => self.pass_1_include_tag(collector, tag),
            _ => Ok(collector),
        }
    }

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

    /// Process anchors that can trigger slow tasks that modify state
    fn pass_1_anchor(&self, collector: Collector, anchor: &Anchor) -> Result<Option<Collector>> {
        tracing::debug!(
            "Worker::pass_1_anchor processing command: {:?}, kind: {:?}",
            anchor.command,
            anchor.kind
        );
        let asm =
            utils::AnchorStateManager::new(self.file_access.clone(), self.path_res.clone(), anchor);
        match (&anchor.command, &anchor.kind) {
            (CommandKind::Answer, AnchorKind::Begin) => {
                tracing::debug!("Worker::pass_1_anchor calling pass_1_answer_begin_anchor");
                self.pass_1_answer_begin_anchor(
                    collector,
                    &asm,
                    &anchor.parameters,
                    &anchor.arguments,
                )
            }
            (CommandKind::Derive, AnchorKind::Begin) => {
                tracing::debug!("Worker::pass_1_anchor calling pass_1_derive_begin_anchor");
                self.pass_1_derive_begin_anchor(
                    collector,
                    &asm,
                    &anchor.parameters,
                    &anchor.arguments,
                )
            }
            (CommandKind::Inline, AnchorKind::Begin) => {
                tracing::debug!("Worker::pass_1_anchor calling pass_1_inline_begin_anchor");
                self.pass_1_inline_begin_anchor(
                    collector,
                    &asm,
                    &anchor.parameters,
                    &anchor.arguments,
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
        }
    }

    fn pass_1_answer_begin_anchor(
        &self,
        collector: Collector,
        asm: &utils::AnchorStateManager,
        _parameters: &Parameters,
        _arguments: &Arguments,
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
                // TODO prelude, secondo agent!!
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
        _parameters: &Parameters,
        arguments: &Arguments,
    ) -> Result<Option<Collector>> {
        let mut state: DeriveState = asm.load_state()?;
        match state.status {
            AnchorStatus::JustCreated => {
                if !collector.can_execute {
                    return Err(anyhow::anyhow!("Execution not allowed"));
                }
                state.instruction_context_name = arguments
                    .arguments
                    .get(0)
                    .ok_or_else(|| anyhow::anyhow!("Missing instruction context name"))?
                    .value
                    .clone();
                state.input_context_name = arguments
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
        _parameters: &Parameters,
        arguments: &Arguments,
    ) -> Result<Option<Collector>> {
        let mut state: InlineState = asm.load_state()?;
        match state.status {
            AnchorStatus::JustCreated => {
                if !collector.can_execute {
                    return Err(anyhow::anyhow!("Execution not allowed"));
                }
                state.context_name = arguments
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

    fn pass_2(&self, context_path: &Path) -> Result<()> {
        tracing::debug!("Worker::pass_2 for path: {:?}", context_path);
        let lock_id = self.file_access.lock_file(context_path)?;
        let result = self.pass_2_internal_x(context_path);
        self.file_access.unlock_file(&lock_id)?;
        tracing::debug!("Worker::pass_2 finished for path: {:?}", context_path);
        result
    }

    fn pass_2_internal_x(&self, context_path: &Path) -> Result<()> {
        let context_content = self.file_access.read_file(context_path)?;
        let ast = crate::ast2::parse_document(&context_content)?;
        let mut patches = utils::Patches::new(&context_content);

        self.pass_2_internal_y(&mut patches, &ast)?;

        if !patches.is_empty() {
            let new_context_content = patches.apply_patches()?;
            self.file_access
                .write_file(context_path, &new_context_content, None)?; // TODO comment?
        }

        Ok(())
    }

    fn pass_2_internal_y(&self, patches: &mut utils::Patches, ast: &Document) -> Result<()> {
        let anchor_index = utils::AnchorIndex::new(&ast.content);

        for item in &ast.content {
            match item {
                crate::ast2::Content::Tag(tag) => {
                    if self.pass_2_tag(patches, tag)? {
                        break;
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
                                    if self.pass_2_anchors(patches, a0, a1)? {
                                        break;
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
        Ok(())
    }

    fn pass_2_tag(&self, patches: &mut utils::Patches, tag: &Tag) -> Result<bool> {
        match tag.command {
            CommandKind::Answer => self.pass_2_normal_tag::<AnswerState>(
                tag.command,
                patches,
                &tag.parameters,
                &tag.arguments,
                &tag.range,
            ),
            CommandKind::Derive => self.pass_2_normal_tag::<DeriveState>(
                tag.command,
                patches,
                &tag.parameters,
                &tag.arguments,
                &tag.range,
            ),
            CommandKind::Inline => self.pass_2_normal_tag::<InlineState>(
                tag.command,
                patches,
                &tag.parameters,
                &tag.arguments,
                &tag.range,
            ),
            _ => Ok(false),
        }
    }

    fn pass_2_normal_tag<S: State + 'static + serde::Serialize>(
        &self,
        command_kind: CommandKind,
        patches: &mut utils::Patches,
        parameters: &Parameters,
        arguments: &Arguments,
        range: &Range,
    ) -> Result<bool> {
        let (a0, a1) = Anchor::new_couple(command_kind, parameters, arguments);
        patches.add_patch(range, &format!("{}\n{}\n", a0.to_string(), a1.to_string()));
        let asm =
            utils::AnchorStateManager::new(self.file_access.clone(), self.path_res.clone(), &a0);
        asm.save_state(&S::new(), None)?;
        Ok(true)
    }

    fn pass_2_anchors(
        &self,
        patches: &mut utils::Patches,
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
                    &a0.parameters,
                    &a0.arguments,
                    &a0.range,
                    &a1.range,
                )
            }
            CommandKind::Derive => {
                tracing::debug!(
                    "Worker::pass_2_anchors calling pass_2_normal_begin_anchor for Derive"
                );
                self.pass_2_normal_begin_anchor::<DeriveState>(
                    patches,
                    &asm,
                    &a0.parameters,
                    &a0.arguments,
                    &a0.range,
                    &a1.range,
                )
            }
            CommandKind::Inline => {
                tracing::debug!(
                    "Worker::pass_2_anchors calling pass_2_normal_begin_anchor for Inline"
                );
                self.pass_2_normal_begin_anchor::<InlineState>(
                    patches,
                    &asm,
                    &a0.parameters,
                    &a0.arguments,
                    &a0.range,
                    &a1.range,
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
        _parameters: &Parameters,
        _arguments: &Arguments,
        range_begin: &Range,
        range_end: &Range,
    ) -> Result<bool> {
        let mut state: S = asm.load_state()?;
        match state.get_status() {
            AnchorStatus::NeedInjection => {
                let range = Range {
                    begin: range_begin.end.clone(),
                    end: range_end.begin.clone(),
                };
                patches.add_patch(&range, &state.output());
                state.set_status(AnchorStatus::Completed);
                asm.save_state(&state, None)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
