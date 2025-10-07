pub mod states;

use crate::agent::ShellAgentCall;
use crate::execute::states::{AnswerState, AnswerStatus, InlineState, SummaryState, SummaryStatus};
use crate::project::{Project, Snippet};
use crate::semantic::{self, Context, Line, Patches};
use crate::utils::AnchorIndex;
use notify::event::ModifyKind;
use serde::{Deserialize, Serialize};
use tracing::debug;
use uuid::timestamp::context;

pub fn execute(
    project: &Project,
    context_name: &str,
    agent: &ShellAgentCall,
) -> anyhow::Result<()> {
    debug!("Executing context: {}", context_name);

    let mut collector = ExecuteCollector::new();
    let mut modified = true;

    while modified {
        debug!("Starting _execute loop for context: {}", context_name);
        modified = _execute(project, context_name, &mut collector, agent)?;
        debug!("_execute returned: {:?}", modified);
    }

    debug!("Context execution finished for: {}", context_name);
    Ok(())
}

struct ExecuteCollector {
    content: Vec<String>,
    content_hash: String,
}

impl ExecuteCollector {
    fn new() -> Self {
        ExecuteCollector {
            content: Vec::new(),
            content_hash: String::new(),
        }
    }
    fn add_line(&mut self, line: &str) {
        self.content.push(line.to_string());
        // TODO update hash
    }
}

fn _execute(
    project: &Project,
    context_name: &str,
    collector: &mut ExecuteCollector,
    agent: &ShellAgentCall,
) -> anyhow::Result<bool> {
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
                    let state = AnswerState::new(collector.content.join("\n"));
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
                    collector.add_line(x);
                }
                Line::IncludeTag { context_name } => {
                    let included_modified = _execute(project, &context_name, collector, agent)?;
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
                Line::InlineBeginAnchor { uuid } => {
                    // Inline state is handled in the first pass, no further action needed here
                }
                Line::AnswerBeginAnchor { uuid } => {
                    let state = project.load_answer_state(uuid)?;
                    let j = anchor_index.get_end(&uuid).ok_or_else(|| {
                        anyhow::anyhow!("End anchor not found for UUID: {}", uuid)
                    })?;
                    match state.status {
                        AnswerStatus::NeedAnswer => {
                            // Do nothing, wait for answer to be provided
                        }
                        AnswerStatus::NeedInjection => {
                            patches.insert(
                                (i + 1, j),
                                state
                                    .reply
                                    .lines()
                                    .map(|s| Line::Text(s.to_string()))
                                    .collect(),
                            );
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
                    let j = anchor_index.get_end(&uuid).ok_or_else(|| {
                        anyhow::anyhow!("End anchor not found for UUID: {}", uuid)
                    })?;
                    match state.status {
                        SummaryStatus::NeedContext => {
                            // Do nothing, wait for context to be provided
                        }
                        SummaryStatus::NeedInjection => {
                            patches.insert(
                                (i + 1, j),
                                state
                                    .summary
                                    .lines()
                                    .map(|s| Line::Text(s.to_string()))
                                    .collect(),
                            );
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

    context.save()?;

    {
        // Third pass: execute long tasks like summaries or answer generation, without further context modification
        for line in &context.lines {
            match line {
                Line::InlineBeginAnchor { uuid } => {
                    // Inline state is handled in the first pass, no further action needed here
                }
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
                            execute(project, &state.context_name, agent)?; // Ensure the context to summarize is fully executed
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
