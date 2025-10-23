use crate::ast2::{
    Anchor, AnchorKind, Arguments, CommandKind, Document, Parameters, Range, Tag, Text,
};
use crate::{agent, file};
use crate::path;
use crate::utils;
use anyhow::Result;
use std::collections::HashSet;
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
    let visit_stack = Vec::new();

    let mut exe = Worker::new(file_access, path_res);
    exe.execute(&visit_stack, context_name)?;

    let content = {
        let mut content = ModelContent::new();
        content.extend(exe.prelude.clone());
        content.extend(exe.context.clone());
        content
    };
    tracing::debug!("Finished executing context: {}", context_name);
    Ok(content)
}

pub fn collect_context(
    file_access: Arc<dyn file::FileAccessor>,
    path_res: Arc<dyn path::PathResolver>,
    context_name: &str,
) -> Result<ModelContent> {
    tracing::debug!("Collecting context: {}", context_name);
    let visit_stack = Vec::new();

    let mut exe = Worker::new(file_access, path_res);
    exe.collect(&visit_stack, context_name)?;

    if !exe.prelude.is_empty() {
        tracing::debug!("Prelude not allowed in collect_context for: {}", context_name);
        return Err(anyhow::anyhow!("Prelude not allowed there"));
    }

    let content = {
        let mut content = ModelContent::new();
        content.extend(exe.context.clone());
        content
    };
    tracing::debug!("Finished collecting context: {}", context_name);
    Ok(content)
}

struct Worker {
    file_access: Arc<dyn file::FileAccessor>,
    path_res: Arc<dyn path::PathResolver>,
    prelude: ModelContent,
    context: ModelContent,
    variables: Variables,
}

impl Worker {
    fn new(
        file_access: Arc<dyn file::FileAccessor>,
        path_res: Arc<dyn path::PathResolver>,
    ) -> Self {
        Worker {
            file_access,
            path_res,
            prelude: Vec::new(),
            context: Vec::new(),
            variables: Variables::new(),
        }
    }

    fn collect(&mut self, visit_stack: &Vec<PathBuf>, context_name: &str) -> Result<()> {
        tracing::debug!("Worker::collect for context: {}", context_name);
        let context_path = self.path_res.resolve_context(context_name)?;

        if visit_stack.contains(&context_path) {
            tracing::debug!("Context {} already in visit stack, skipping collection.", context_name);
            return Ok(());
        }

        let mut visit_stack = visit_stack.clone();
        visit_stack.push(context_path.clone());

        // Read file, parse it, collect without allowing execution
        let want_next_step_1 = self.pass_1(&visit_stack, &context_path, false)?;

        if want_next_step_1 {
            tracing::debug!("Next step not allowed in Worker::collect for context: {}", context_name);
            return Err(anyhow::anyhow!("Next step not allowed"));
        }
        tracing::debug!("Worker::collect finished for context: {}", context_name);
        Ok(())
    }

    fn execute(&mut self, visit_stack: &Vec<PathBuf>, context_name: &str) -> Result<()> {
        tracing::debug!("Worker::execute for context: {}", context_name);
        let context_path = self.path_res.resolve_context(context_name)?;

        if visit_stack.contains(&context_path) {
            tracing::debug!("Context {} already in visit stack, skipping execution.", context_name);
            return Ok(());
        }

        let mut visit_stack = visit_stack.clone();
        visit_stack.push(context_path.clone());

        while self.execute_step(&visit_stack, &context_path)? {}
        tracing::debug!("Worker::execute finished for context: {}", context_name);
        Ok(())
    }

    fn execute_step(&mut self, visit_stack: &Vec<PathBuf>, context_path: &Path) -> Result<bool> {
        tracing::debug!("Worker::execute_step for path: {:?}", context_path);
        // Read file, parse it, execute slow things that do not modify context
        let want_next_step_1 = self.pass_1(&visit_stack, context_path, true)?;

        // Lock file, re-read it (could be edited outside), parse it, execute fast things that may modify context and save it
        let want_next_step_2 = self.pass_2(context_path)?;
        tracing::debug!("Worker::execute_step finished for path: {:?}, want_next_step_1: {}, want_next_step_2: {}", context_path, want_next_step_1, want_next_step_2);
        Ok(want_next_step_1 | want_next_step_2)
    }

    fn call_model(&self, contents: Vec<ModelContent>) -> Result<String> {
        let query = contents
            .into_iter()
            .flatten()
            .map(|item| item.to_string())
            .collect::<Vec<String>>()
            .join("\n---\n");
        agent::shell::shell_call(&self.variables.provider, &query)
    }

    fn pass_1(
        &mut self,
        visit_stack: &Vec<PathBuf>,
        context_path: &Path,
        can_execute: bool,
    ) -> Result<bool> {
        tracing::debug!("Worker::pass_1 for path: {:?}, can_execute: {}", context_path, can_execute);
        let context_content = self.file_access.read_file(context_path)?;
        let ast = crate::ast2::parse_document(&context_content)?;

        for item in &ast.content {
            match item {
                crate::ast2::Content::Text(text) => self.pass_1_text(text)?,
                crate::ast2::Content::Tag(tag) => {
                    if self.pass_1_tag(visit_stack, tag)? {
                        tracing::debug!("Worker::pass_1 returning true after processing tag for path: {:?}", context_path);
                        return Ok(true);
                    }
                }
                crate::ast2::Content::Anchor(anchor) => {
                    if !can_execute {
                        tracing::debug!("Execution not allowed in Worker::pass_1 for anchor in path: {:?}", context_path);
                        return Err(anyhow::anyhow!("Execution not allowed"));
                    }
                    if self.pass_1_anchor(anchor)? {
                        tracing::debug!("Worker::pass_1 returning true after processing anchor for path: {:?}", context_path);
                        return Ok(true);
                    }
                }
            }
        }
        tracing::debug!("Worker::pass_1 finished for path: {:?}", context_path);
        Ok(false)
    }

    fn pass_1_text(&mut self, text: &Text) -> Result<()> {
        self.context.push(ModelContentItem::user(&text.content));
        Ok(())
    }

    /// Process tags that do NOT modify context, so tags that do NOT spawn anchors
    fn pass_1_tag(&mut self, visit_stack: &Vec<PathBuf>, tag: &Tag) -> Result<bool> {
        match tag.command {
            CommandKind::Include => self.pass_1_include_tag(visit_stack, tag),
            _ => Ok(false),
        }
    }

    fn pass_1_include_tag(&mut self, visit_stack: &Vec<PathBuf>, tag: &Tag) -> Result<bool> {
        let included_context_name = tag
            .arguments
            .arguments
            .get(0)
            .ok_or_else(|| anyhow::anyhow!("Missing argument for include tag"))?
            .value
            .clone();
        self.collect(visit_stack, &included_context_name)?;
        Ok(true)
    }

    /// Process anchors that can trigger slow tasks that modify state
    fn pass_1_anchor(&mut self, anchor: &Anchor) -> Result<bool> {
        tracing::debug!("Worker::pass_1_anchor processing command: {:?}, kind: {:?}", anchor.command, anchor.kind);
        let asm =
            utils::AnchorStateManager::new(self.file_access.clone(), self.path_res.clone(), anchor);
        match (&anchor.command, &anchor.kind) {
            (CommandKind::Answer, AnchorKind::Begin) => {
                tracing::debug!("Worker::pass_1_anchor calling pass_1_answer_begin_anchor");
                self.pass_1_answer_begin_anchor(&asm, &anchor.parameters, &anchor.arguments)
            }
            (CommandKind::Derive, AnchorKind::Begin) => {
                tracing::debug!("Worker::pass_1_anchor calling pass_1_derive_begin_anchor");
                self.pass_1_derive_begin_anchor(&asm, &anchor.parameters, &anchor.arguments)
            }
            (CommandKind::Inline, AnchorKind::Begin) => {
                tracing::debug!("Worker::pass_1_anchor calling pass_1_inline_begin_anchor");
                self.pass_1_inline_begin_anchor(&asm, &anchor.parameters, &anchor.arguments)
            }
            _ => {
                tracing::debug!("Worker::pass_1_anchor not handling command: {:?}, kind: {:?}", anchor.command, anchor.kind);
                Ok(false)
            }
        }
    }

    fn pass_1_answer_begin_anchor(
        &mut self,
        asm: &utils::AnchorStateManager,
        parameters: &Parameters,
        arguments: &Arguments,
    ) -> Result<bool> {
        let mut state: AnswerState = asm.load_state()?;
        match state.status {
            AnchorStatus::JustCreated => {
                state.query = self.context.clone();
                state.status = AnchorStatus::NeedProcessing;
                asm.save_state(&state, None)?;
                Ok(true)
            }
            AnchorStatus::NeedProcessing => {
                // TODO prelude, secondo agent!!
                state.reply = self.call_model(vec![state.query.clone()])?;
                state.status = AnchorStatus::NeedInjection;
                asm.save_state(&state, None)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn pass_1_derive_begin_anchor(
        &mut self,
        asm: &utils::AnchorStateManager,
        parameters: &Parameters,
        arguments: &Arguments,
    ) -> Result<bool> {
        let mut state: DeriveState = asm.load_state()?;
        match state.status {
            AnchorStatus::JustCreated => {
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
                Ok(true)
            }
            AnchorStatus::NeedProcessing => {
                state.instruction_context = execute_context(
                    self.file_access.clone(),
                    self.path_res.clone(),
                    &state.instruction_context_name,
                )?;
                state.input_context = execute_context(
                    self.file_access.clone(),
                    self.path_res.clone(),
                    &state.input_context_name,
                )?;
                // call llm to derive
                state.derived = "rispostone!! TODO ".into();
                state.status = AnchorStatus::NeedInjection;
                asm.save_state(&state, None)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn pass_1_inline_begin_anchor(
        &mut self,
        asm: &utils::AnchorStateManager,
        parameters: &Parameters,
        arguments: &Arguments,
    ) -> Result<bool> {
        let mut state: InlineState = asm.load_state()?;
        match state.status {
            AnchorStatus::JustCreated => {
                state.context_name = arguments
                    .arguments
                    .get(0)
                    .ok_or_else(|| anyhow::anyhow!("Missing context name"))?
                    .value
                    .clone();
                state.status = AnchorStatus::NeedProcessing;
                asm.save_state(&state, None)?;
                Ok(true)
            }
            AnchorStatus::NeedProcessing => {
                let context_path = self.path_res.resolve_context(&state.context_name)?;
                state.context = self.file_access.read_file(&context_path)?;
                state.status = AnchorStatus::NeedInjection;
                asm.save_state(&state, None)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn pass_2(&mut self, context_path: &Path) -> Result<bool> {
        tracing::debug!("Worker::pass_2 for path: {:?}", context_path);
        let lock_id = self.file_access.lock_file(context_path)?;
        let result = self.pass_2_internal_x(context_path);
        self.file_access.unlock_file(&lock_id)?;
        tracing::debug!("Worker::pass_2 finished for path: {:?}", context_path);
        result
    }

    fn pass_2_internal_x(&mut self, context_path: &Path) -> Result<bool> {
        let context_content = self.file_access.read_file(context_path)?;
        let ast = crate::ast2::parse_document(&context_content)?;
        let mut patches = utils::Patches::new(&context_content);

        let want_next_step = self.pass_2_internal_y(&mut patches, &ast)?;

        if !patches.is_empty() {
            let new_context_content = patches.apply_patches()?;
            self.file_access
                .write_file(context_path, &new_context_content, None)?; // TODO comment?
        }

        Ok(want_next_step)
    }

    fn pass_2_internal_y(&mut self, patches: &mut utils::Patches, ast: &Document) -> Result<bool> {
        let anchor_index = utils::AnchorIndex::new(&ast.content);

        for item in &ast.content {
            match item {
                crate::ast2::Content::Tag(tag) => {
                    if self.pass_2_tag(patches, tag)? {
                        return Ok(true);
                    }
                }
                crate::ast2::Content::Anchor(a0) => {
                    return match a0.kind {
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
                                    AnchorKind::End => self.pass_2_anchors(patches, a0, a1),
                                    _ => return Err(anyhow::anyhow!("Bad end anchor!")),
                                },
                                _ => return Err(anyhow::anyhow!("Bad end content!")),
                            }
                        }
                        _ => Ok(false),
                    }
                }
                _ => {}
            }
        }

        Ok(false)
    }

    fn pass_2_tag(&mut self, patches: &mut utils::Patches, tag: &Tag) -> Result<bool> {
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
        &mut self,
        command_kind: CommandKind,
        patches: &mut utils::Patches,
        parameters: &Parameters,
        arguments: &Arguments,
        range: &Range,
    ) -> Result<bool> {
        let (a0, a1) = Anchor::new_couple(command_kind, parameters, arguments);
        patches.add_patch(range, &format!("{}\n{}", a0.to_string(), a1.to_string()));
        let asm =
            utils::AnchorStateManager::new(self.file_access.clone(), self.path_res.clone(), &a0);
        asm.save_state(&S::new(), None)?;
        Ok(true)
    }

    fn pass_2_anchors(
        &mut self,
        patches: &mut utils::Patches,
        a0: &Anchor,
        a1: &Anchor,
    ) -> Result<bool> {
        tracing::debug!("Worker::pass_2_anchors processing command: {:?}", a0.command);
        let asm =
            utils::AnchorStateManager::new(self.file_access.clone(), self.path_res.clone(), a0);
        match a0.command {
            CommandKind::Answer => {
                tracing::debug!("Worker::pass_2_anchors calling pass_2_normal_begin_anchor for Answer");
                self.pass_2_normal_begin_anchor(
                    patches,
                    &asm,
                    &a0.parameters,
                    &a0.arguments,
                    &a0.range,
                    &a1.range,
                )
            }
            CommandKind::Derive => {
                tracing::debug!("Worker::pass_2_anchors calling pass_2_normal_begin_anchor for Derive");
                self.pass_2_normal_begin_anchor(
                    patches,
                    &asm,
                    &a0.parameters,
                    &a0.arguments,
                    &a0.range,
                    &a1.range,
                )
            }
            CommandKind::Inline => {
                tracing::debug!("Worker::pass_2_anchors calling pass_2_normal_begin_anchor for Inline");
                self.pass_2_normal_begin_anchor(
                    patches,
                    &asm,
                    &a0.parameters,
                    &a0.arguments,
                    &a0.range,
                    &a1.range,
                )
            }
            _ => {
                tracing::debug!("Worker::pass_2_anchors not handling command: {:?}", a0.command);
                Ok(false)
            }
        }
    }

    fn pass_2_normal_begin_anchor(
        &mut self,
        patches: &mut utils::Patches,
        asm: &utils::AnchorStateManager,
        parameters: &Parameters,
        arguments: &Arguments,
        range_begin: &Range,
        range_end: &Range,
    ) -> Result<bool> {
        let mut state: AnswerState = asm.load_state()?;
        match state.status {
            AnchorStatus::NeedInjection => {
                let range = Range {
                    begin: range_begin.end.clone(),
                    end: range_end.begin.clone(),
                };
                patches.add_patch(&range, &state.output());
                state.status = AnchorStatus::Completed;
                asm.save_state(&state, None)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
