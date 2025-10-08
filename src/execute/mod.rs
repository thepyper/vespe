pub mod states;

use std::collections::HashSet;

use crate::agent::ShellAgentCall;
use crate::execute::states::{AnswerState, AnswerStatus, InlineState, SummaryState, SummaryStatus};
use crate::project::Project;
use crate::semantic::{self, Context, Line, Patches};
use crate::utils::AnchorIndex;
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
) -> anyhow::Result<()> {
    debug!("Executing context: {}", context_name);

    let mut visited_contexts = HashSet::new();
    let mut worker = ExecuteWorker::new();
    worker.execute_loop(project, context_name, agent, &mut visited_contexts)?;

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

    fn get_hash(&self) -> String {
        format!("{:x}", self.hasher.clone().finalize())
    }

    fn execute_loop(
        &mut self,
        project: &Project,
        context_name: &str,
        agent: &ShellAgentCall,
        visited_contexts: &mut HashSet<String>,
    ) -> anyhow::Result<()> {
        for i = 1..100 {
            debug!("Starting execute_step {} loop for context: {}", i, context_name);
            if !worker.execute_step(project, context_name, agent, visited_contexts)? {
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
    ) -> anyhow::Result<bool> {
        if !visited_contexts.insert(context_name.to_string()) {
            debug!("Context '{}' already visited. Skipping further execution.", context_name);
            return Ok(false);
        }

        let mut modified = false;
        let mut context = Context::load(project, context_name)?;
        // analyze orphan anchors TODO

        {
            // First pass: handle tags and transform them into anchors
            let mut patches = Patches::new();

            for (i, line) in context.lines.iter().enumerate() {
                match line {
                    Line::InlineTag { snippet_name } => {
                        let uid = uuid::Uuid::new_v4();
                        let anchors = semantic::Line::new_inline_anchors(uid);
                        let snippet = project.load_snippet(&snippet_name)?;
                        let mut patch = Vec::new();
                        patch.push(anchors[0].clone());
                        patch.extend(snippet.content);
                        patch.push(anchors[1].clone());
                        patches.insert(
                            (i, i + 1), // Replace the current line (the tag line) with the anchor
                            patch,
                        );
                        let state = InlineState::new(&snippet_name);
                        project.save_inline_state(&uid, &state)?;
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
                        let state = AnswerState::new(self.content.join("\n"));
                        project.save_answer_state(&uid, &state)?;
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
                        project.save_summary_state(&uid, &state)?;
                        // Exit processing, summary could add content needed for further actions
                        break;
                    }
                    Line::Text(x) => {
                        self.add_line(x);
                    }
                    Line::IncludeTag { context_name } => {
                        let included_modified = self.execute_step(project, &context_name, agent, visited_contexts)?;
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
                debug!(
                    "Modified context is now\n***\n{}\n***\n",
                    semantic::format_document(&context.lines)
                );
                modified = true;
            }
        }

        {
            // Second pass: handle anchors and their states
            let mut patches = Patches::new();
            let anchor_index = AnchorIndex::new(&context.lines);

            for (i, line) in context.lines.iter().enumerate() {
                match line {
                    // Inline state is handled in the first pass, no further action needed here
                    Line::AnswerBeginAnchor { uuid } => {
                        let state = project.load_answer_state(uuid)?;
                        let j = anchor_index.get_end(uuid).ok_or_else(|| {
                            ExecuteError::MissingEndAnchor(*uuid)
                        })?;
                        match state.status {
                            AnswerStatus::NeedAnswer => {
                                // Do nothing, wait for answer to be provided
                            }
                            AnswerStatus::NeedInjection => {
                                inject_content(&mut patches, i + 1, j, &state.reply)?;
                                let mut new_state = state.clone();
                                new_state.status = AnswerStatus::Completed;
                                project.save_answer_state(uuid, &new_state)?;
                                modified = true;
                            }
                            AnswerStatus::Completed => {
                                // Do nothing, already completed
                            }
                        }
                    }
                    Line::SummaryBeginAnchor { uuid } => {
                        let state = project.load_summary_state(uuid)?;
                        let j = anchor_index.get_end(uuid).ok_or_else(|| {
                            ExecuteError::MissingEndAnchor(*uuid)
                        })?;
                        match state.status {
                            SummaryStatus::NeedContext => {
                                // Do nothing, wait for context to be provided
                            }
                            SummaryStatus::NeedInjection => {
                                inject_content(&mut patches, i + 1, j, &state.summary)?;
                                let mut new_state = state.clone();
                                new_state.status = SummaryStatus::Completed;
                                project.save_summary_state(uuid, &new_state)?;
                                modified = true;
                            }
                            SummaryStatus::Completed => {
                                // Do nothing, already completed
                            }
                        }
                    }
                    _ => { /* Do nothing */ }
                }
            }

            if context.apply_patches(patches) {
                debug!("Pass 2 patches applied.");
                debug!(
                    "Modified context is now\n***\n{}\n***\n",
                    semantic::format_document(&context.lines)
                );
                modified = true;
            }
        }

        let uid = project.request_file_modification(&context.path)?;
        context.save()?;
        project.notify_file_modified(&context.path, uid)?;

        {
            // Third pass: execute long tasks like summaries or answer generation, without further context modification
            for line in &context.lines {
                match line {
                    Line::AnswerBeginAnchor { uuid } => {
                        let state = project.load_answer_state(uuid)?;
                        match state.status {
                            AnswerStatus::NeedAnswer => {
                                let mut new_state = state.clone();
                                new_state.reply = agent.call(&state.query)?;
                                new_state.status = AnswerStatus::NeedInjection;
                                project.save_answer_state(uuid, &new_state)?;
                            }
                            _ => { /* Do nothing */ }
                        }
                    }
                    Line::SummaryBeginAnchor { uuid } => {
                        let state = project.load_summary_state(uuid)?;
                        match state.status {
                            SummaryStatus::NeedContext => {
                                let mut summary_worker = ExecuteWorker::new();
                                summary_worker.execute_loop(project, &state.context_name, agent, visited_contexts)?;  
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
                                project.save_summary_state(uuid, &new_state)?;
                            }
                            _ => { /* Do nothing */ }
                        }
                    }
                    _ => { /* Do nothing */ }
                }
            }
        }

        Ok(modified)
    }
}



fn inject_content(
    patches: &mut Patches,
    start_index: usize,
    end_index: usize,
    content: &str,
) -> anyhow::Result<()> {
    let lines = content
        .lines()
        .map(|s| Line::Text(s.to_string()))
        .collect();
    patches.insert((start_index, end_index), lines);
    Ok(())
}
