pub mod states;

use std::collections::HashSet;

use crate::agent::ShellAgentCall;
use crate::execute::states::{
    AnswerState, AnswerStatus, InlineState, InlineStatus, SummaryState, SummaryStatus,
    DeriveState, DeriveStatus
};
use crate::git::Commit;
use crate::project::{Project, Snippet};
use crate::semantic::{self, Context, Line, Patches};
use crate::utils::AnchorIndex;
use anyhow::anyhow;

use sha2::{Digest, Sha256};
use thiserror::Error;
use tracing::debug;

#[derive(Error, Debug)]
pub enum ExecuteError {
    #[error("End anchor not found for UUID: {0}")]
    MissingEndAnchor(uuid::Uuid),
}

pub fn execute(
    project: &Project,
    context_name: &str,
    agent: &ShellAgentCall,
    commit: &mut Commit,
) -> anyhow::Result<()> {
    debug!("Executing context: {}", context_name);

    let mut visited_contexts = HashSet::new();

    let mut worker = ExecuteWorker::new();
    worker.execute_loop(project, context_name, agent, &mut visited_contexts, commit)?;

    debug!("Context execution finished for: {}", context_name);
    Ok(())
}

struct ExecuteWorker {
    content: Vec<String>,
    hasher: Sha256,
}

impl ExecuteWorker {
    fn new() -> Self {
        ExecuteWorker {
            content: Vec::new(),
            hasher: Sha256::new(),
        }
    }
    fn add_line(&mut self, line: &str) {
        self.content.push(line.to_string());
        self.hasher.update(line.as_bytes());
    }

    fn execute_loop(
        &mut self,
        project: &Project,
        context_name: &str,
        agent: &ShellAgentCall,
        initial_visited_contexts: &mut HashSet<String>,
        commit: &mut Commit,
    ) -> anyhow::Result<()> {
        for i in 1..100 {
            debug!(
                "Starting execute_step {} loop for context: {}",
                i, context_name
            );
            let mut visited_contexts = initial_visited_contexts.clone();
            if !self.execute_step(project, context_name, agent, &mut visited_contexts, commit)? {
                break;
            }
        }
        Ok(())
    }

    fn execute_step(
        &mut self,
        project: &Project,
        context_name: &str,
        agent: &ShellAgentCall,
        visited_contexts: &mut HashSet<String>,
        commit: &mut Commit,
    ) -> anyhow::Result<bool> {
        if !visited_contexts.insert(context_name.to_string()) {
            debug!(
                "Context '{}' already visited. Skipping further execution.",
                context_name
            );
            return Ok(false);
        }

        let mut context = Context::load(project, context_name)?;

        // Transform tags into anchors and initialize states
        let mut need_next_step =
            self._handle_tags_and_anchors(project, &mut context, agent, visited_contexts, commit)?;

        // Handle repeat tag
        let mut need_next_step = self._hanled_repeat_tag(project, &mut context, commit)?;

        // Execute document injection and mutate states
        need_next_step |= self._handle_anchor_states(project, &mut context, commit)?;

        // Save document, no more document changes on this step
        if context.modified {
            let uid = project.request_file_modification(&context.path)?;
            context.save()?;
            commit.files.insert(context.path.clone());
            project.notify_file_modified(&context.path, uid)?;
        }

        // Execute long tasks that only mutate state (and not document)
        need_next_step |=
            self._execute_long_tasks(project, &context, agent, visited_contexts, commit)?;

        Ok(need_next_step)
    }

    fn _handle_tags_and_anchors(
        &mut self,
        project: &Project,
        context: &mut Context,
        agent: &ShellAgentCall,
        visited_contexts: &mut HashSet<String>,
        commit: &mut Commit,
    ) -> anyhow::Result<bool> {
        let mut modified = false;
        let mut patches = Patches::new();

        for (i, line) in context.lines.iter().enumerate() {
            match line {
                Line::InlineTag { snippet_name } => {
                    let uid = uuid::Uuid::new_v4();
                    let anchors = semantic::Line::new_inline_anchors(uid);
                    patches.insert(
                        (i, i + 1), // Replace the current line (the tag line) with the anchor
                        anchors,
                    );
                    let state = InlineState::new(&snippet_name);
                    project.save_inline_state(&uid, &state, commit)?;
                    // Exit processing, inline could add any kind of content that need re-execution
                    break;
                }
                Line::AnswerTag => {
                    let uid = uuid::Uuid::new_v4();
                    let anchors = semantic::Line::new_answer_anchors(uid);
                    patches.insert(
                        (i, i + 1), // Replace the current line (the tag line) with the anchor
                        anchors,
                    );
                    let state = AnswerState::new();
                    project.save_answer_state(&uid, &state, commit)?;
                    // Exit processing, answer could add content needed for further actions
                    break;
                }
                Line::SummaryTag { context_name } => {
                    let uid = uuid::Uuid::new_v4();
                    let anchors = semantic::Line::new_summary_anchors(uid);
                    patches.insert(
                        (i, i + 1), // Replace the current line (the tag line) with the anchor
                        anchors,
                    );
                    let state = SummaryState::new(&context_name);
                    project.save_summary_state(&uid, &state, commit)?;
                    // Exit processing, summary could add content needed for further actions
                    break;
                }
                Line::DeriveTag { snippet_name, context_name } => {
                    let uid = uuid::Uuid::new_v4();
                    let anchors = semantic::Line::new_derive_anchors(uid);
                    patches.insert(
                        (i, i + 1), // Replace the current line (the tag line) with the anchor
                        anchors,
                    );
                    let state = DeriveState::new(&snippet_name, &context_name);
                    project.save_derive_state(&uid, &state, commit)?;
                    // Exit processing, derive could add content needed for further actions
                    break;
                }
                Line::IncludeTag { context_name } => {
                    let included_modified =
                        self.execute_step(project, &context_name, agent, visited_contexts, commit)?;
                    if included_modified {
                        // Exit processing, included could add any kind of content that need re-execution
                        break;
                    }
                }
                _ => { /* Do nothing */ }
            }
        }

        if context.apply_patches(patches) {
            debug!("Pass 1 patches applied.");
            modified = true;
        }
        Ok(modified)
    }

    fn _hanled_repeat_tag(
        &mut self,
        project: &Project,
        context: &mut Context,
        commit: &mut Commit,
    ) -> anyhow::Result<bool> {
        let mut modified = false;
        let mut patches = Patches::new();
        let anchor_index = AnchorIndex::new(&context.lines);
        let mut latest_anchor = None;

        for (i, line) in context.lines.iter().enumerate() {
            match line {
                Line::RepeatTag => {
                    if let Some(j) = latest_anchor {
                        // Repeat inside some anchor, manage it
                        let anchor_begin_line: &Line = context.lines.get(j).unwrap(); // TODO gestione errore
                        let uuid = &anchor_begin_line.get_uid();
                        let k = anchor_index
                            .get_end(uuid)
                            .ok_or_else(|| ExecuteError::MissingEndAnchor(*uuid))?;
                        inject_lines(&mut patches, j + 1, k, vec![]);
                        match anchor_begin_line {
                            Line::InlineBeginAnchor { uuid } => {
                                let state = project.load_inline_state(uuid)?;
                                match state.status {
                                    InlineStatus::Completed => {
                                        let mut new_state = state.clone();
                                        new_state.status = InlineStatus::NeedInjection;
                                        project.save_inline_state(uuid, &new_state, commit)?;
                                        modified = true;
                                    }
                                    _ => { /* Do nothing */ }
                                }
                            }
                            Line::AnswerBeginAnchor { uuid } => {
                                let state = project.load_answer_state(uuid)?;
                                match state.status {
                                    AnswerStatus::Completed => {
                                        let mut new_state = state.clone();
                                        new_state.status = AnswerStatus::NeedContext;
                                        project.save_answer_state(uuid, &new_state, commit)?;
                                        modified = true;
                                    }
                                    _ => { /* Do nothing */ }
                                }
                            }
                            Line::SummaryBeginAnchor { uuid } => {
                                let state = project.load_summary_state(uuid)?;
                                match state.status {
                                    SummaryStatus::Completed => {
                                        let mut new_state = state.clone();
                                        new_state.status = SummaryStatus::NeedContext;
                                        project.save_summary_state(uuid, &new_state, commit)?;
                                        modified = true;
                                    }
                                    _ => { /* Do nothing */ }
                                }
                            }
                             Line::DeriveBeginAnchor { uuid } => {
                                let state = project.load_derive_state(uuid)?;
                                match state.status {
                                    DeriveStatus::Completed => {
                                        let mut new_state = state.clone();
                                        new_state.status = DeriveStatus::NeedContext;
                                        project.save_derive_state(uuid, &new_state, commit)?;
                                        modified = true;
                                    }
                                    _ => { /* Do nothing */ }
                                }
                            }
                            _ => { /* Do nothing */ }
                        }
                    } else {
                        // Repeat outside of any anchor, just remove it
                        patches.insert((i, i + 1), vec![]);
                    }
                }
                Line::AnswerBeginAnchor { .. }
                | Line::InlineBeginAnchor { .. }
                | Line::SummaryBeginAnchor { .. }
                | Line::DeriveBeginAnchor { .. } => { // TODO anchors annidate non funzionano bene cosi!!!
                    latest_anchor = Some(i);
                }
                _ => { /* Do nothing */ }
            }
        }

        if context.apply_patches(patches) {
            debug!("Pass 1.5 patches applied.");
            modified = true;
        }
        Ok(modified)
    }

    fn _handle_anchor_states(
        &mut self,
        project: &Project,
        context: &mut Context,
        commit: &mut Commit,
    ) -> anyhow::Result<bool> {
        let mut modified = false;
        let mut patches = Patches::new();
        let anchor_index = AnchorIndex::new(&context.lines);

        for (i, line) in context.lines.iter().enumerate() {
            match line {
                Line::InlineBeginAnchor { uuid } => {
                    let state = project.load_inline_state(uuid)?;
                    let j = anchor_index
                        .get_end(uuid)
                        .ok_or_else(|| ExecuteError::MissingEndAnchor(*uuid))?;
                    match state.status {
                        InlineStatus::NeedInjection => {
                            let snippet = project.load_snippet(&state.snippet_name)?;
                            inject_lines(&mut patches, i + 1, j, snippet.content)?;
                            let mut new_state = state.clone();
                            new_state.status = InlineStatus::Completed;
                            project.save_inline_state(uuid, &new_state, commit)?;
                            modified = true;
                        }
                        InlineStatus::Completed => {
                            // Do nothing, already completed
                        }
                    }
                }
                Line::AnswerBeginAnchor { uuid } => {
                    let state = project.load_answer_state(uuid)?;
                    let j = anchor_index
                        .get_end(uuid)
                        .ok_or_else(|| ExecuteError::MissingEndAnchor(*uuid))?;
                    match state.status {
                        AnswerStatus::NeedContext => {
                            let mut new_state = state.clone();
                            new_state.status = AnswerStatus::NeedAnswer;
                            new_state.query = self.content.join("\n");
                            project.save_answer_state(uuid, &new_state, commit)?;
                            modified = true;
                        }
                        AnswerStatus::NeedAnswer => {
                            // Do nothing, wait for answer to be provided
                        }
                        AnswerStatus::NeedInjection => {
                            inject_content(&mut patches, i + 1, j, &state.reply)?;
                            let mut new_state = state.clone();
                            new_state.status = AnswerStatus::Completed;
                            project.save_answer_state(uuid, &new_state, commit)?;
                            modified = true;
                        }
                        AnswerStatus::Completed => {
                            // Do nothing, already completed
                        }
                    }
                }
                Line::SummaryBeginAnchor { uuid } => {
                    let state = project.load_summary_state(uuid)?;
                    let j = anchor_index
                        .get_end(uuid)
                        .ok_or_else(|| ExecuteError::MissingEndAnchor(*uuid))?;
                    match state.status {
                        SummaryStatus::NeedContext => {
                            // Do nothing, wait for context to be provided
                        }
                        SummaryStatus::NeedInjection => {
                            inject_content(&mut patches, i + 1, j, &state.summary)?;
                            let mut new_state = state.clone();
                            new_state.status = SummaryStatus::Completed;
                            project.save_summary_state(uuid, &new_state, commit)?;
                            modified = true;
                        }
                        SummaryStatus::Completed => {
                            // Do nothing, already completed
                        }
                    }
                }
                Line::DeriveBeginAnchor { uuid } => {
                    let state = project.load_derive_state(uuid)?;
                    let j = anchor_index
                        .get_end(uuid)
                        .ok_or_else(|| ExecuteError::MissingEndAnchor(*uuid))?;
                    match state.status {
                        DeriveStatus::NeedContext => {
                            // Do nothing, wait for context to be provided
                        }
                        DeriveStatus::NeedInjection => {
                            inject_content(&mut patches, i + 1, j, &state.derived)?;
                            let mut new_state = state.clone();
                            new_state.status = DeriveStatus::Completed;
                            project.save_derive_state(uuid, &new_state, commit)?;
                            modified = true;
                        }
                        DeriveStatus::Completed => {
                            // Do nothing, already completed
                        }
                    }
                }
                Line::Text(x) => {
                    self.add_line(x);
                }
                _ => { /* Do nothing */ }
            }
        }

        if context.apply_patches(patches) {
            debug!("Pass 2 patches applied.");
            modified = true;
        }
        Ok(modified)
    }

    fn _execute_long_tasks(
        &mut self,
        project: &Project,
        context: &Context,
        agent: &ShellAgentCall,
        visited_contexts: &mut HashSet<String>,
        commit: &mut Commit,
    ) -> anyhow::Result<bool> {
        let mut need_next_step = false;
        for line in &context.lines {
            match line {
                Line::AnswerBeginAnchor { uuid } => {
                    let state = project.load_answer_state(uuid)?;
                    match state.status {
                        AnswerStatus::NeedAnswer => {
                            let mut new_state = state.clone();
                            new_state.reply = agent.call(&state.query)?;
                            new_state.status = AnswerStatus::NeedInjection;
                            project.save_answer_state(uuid, &new_state, commit)?;
                            need_next_step = true;
                        }
                        _ => { /* Do nothing */ }
                    }
                }
                Line::SummaryBeginAnchor { uuid } => {
                    let state = project.load_summary_state(uuid)?;
                    match state.status {
                        SummaryStatus::NeedContext => {
                            let mut summary_worker = ExecuteWorker::new();
                            summary_worker.execute_loop(
                                project,
                                &state.context_name,
                                agent,
                                visited_contexts,
                                commit,
                            )?;
                            let context = Context::load(project, &state.context_name)?;
                            let mut new_state = state.clone();
                            new_state.context = context
                                .lines
                                .iter()
                                .filter_map(|line| {
                                    if let Line::Text(t) = line {
                                        Some(t.clone())
                                    } else {
                                        None
                                    }
                                })
                                .collect::<Vec<String>>()
                                .join("\n");
                            new_state.summary = agent.call(&format!(
                                "Summarize the following content:\n\n{}",
                                new_state.context
                            ))?;
                            new_state.status = SummaryStatus::NeedInjection;
                            project.save_summary_state(uuid, &new_state, commit)?;
                            need_next_step = true;
                        }
                        _ => { /* Do nothing */ }
                    }
                    Line::DeriveBeginAnchor { uuid } => {
                        let state = project.load_derive_state(uuid)?;
                        match state.status {
                            DeriveStatus::NeedContext => {
                                let mut derive_worker = ExecuteWorker::new();
                                derive_worker.execute_loop(
                                    project,
                                    &state.context_name,
                                    agent,
                                    visited_contexts,
                                    commit,
                                )?;
                                let snippet = project.load_snippet(&state.snippet_name)?;
                                let snippet_text =  snippet.content
                                    .iter()
                                    .filter_map(|line| {
                                        if let Line::Text(t) = line {
                                            Some(t.clone())
                                        } else {
                                            None
                                        }
                                    })
                                    .collect::<Vec<String>>()
                                    .join("\n");
                                let context = Context::load(project, &state.context_name)?;
                                let context_text = context
                                    .lines
                                    .iter()
                                    .filter_map(|line| {
                                        if let Line::Text(t) = line {
                                            Some(t.clone())
                                        } else {
                                            None
                                        }
                                    })
                                    .collect::<Vec<String>>()
                                    .join("\n");
                                let mut new_state = state.clone();
                                new_state.context = context_text;
                                new_state.snippet = snippet_text;
                                new_state.derived = agent.call(&format!(                               
                                    "{}\n{}",
                                    new_state.snippet,
                                    new_state.context
                                ))?;
                                new_state.status = DeriveStatus::NeedInjection;
                                project.save_derive_state(uuid, &new_state, commit)?;
                                need_next_step = true;
                            }
                            _ => { /* Do nothing */ }
                        }
                    }
                }
                _ => { /* Do nothing */ }
            }
        }
        Ok(need_next_step)
    }
}

fn inject_lines(
    patches: &mut Patches,
    start_index: usize,
    end_index: usize,
    lines: Vec<Line>,
) -> anyhow::Result<()> {
    patches.insert((start_index, end_index), lines);
    Ok(())
}

fn inject_content(
    patches: &mut Patches,
    start_index: usize,
    end_index: usize,
    content: &str,
) -> anyhow::Result<()> {
    let lines = content.lines().map(|s| Line::Text(s.to_string())).collect();
    inject_lines(patches, start_index, end_index, lines)
}
