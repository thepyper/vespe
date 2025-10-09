pub mod states;

use std::collections::HashSet;

use crate::agent::ShellAgentCall;
use crate::execute::states::{AnswerState, AnswerStatus, InlineState, InlineStatus, SummaryState, SummaryStatus};
use crate::project::Project;
use crate::semantic::{self, Context, Line, Patches};
use crate::utils::AnchorIndex;
use crate::git::Commit;
use crate::error::{Result, Error as GeneralError};

use sha2::{Digest, Sha256};
use thiserror::Error;
use tracing::debug;

#[derive(Error, Debug)]
pub enum ExecuteError {
    #[error("End anchor not found for UUID: {0}")]
    MissingEndAnchor(uuid::Uuid),
    #[error("Project error: {0}")]
    Project(#[from] crate::project::ProjectError),
    #[error("Agent error: {0}")]
    Agent(#[from] crate::agent::AgentError),
    #[error("Semantic error: {0}")]
    Semantic(#[from] crate::semantic::SemanticError),
    #[error("Git error: {0}")]
    Git(#[from] crate::git::GitError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serde JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("UUID error: {0}")]
    Uuid(#[from] uuid::Error),
    #[error("Failed to load context '{0}': {1}")]
    ContextLoadFailed(String, String),
    #[error("Failed to save context '{0}': {1}")]
    ContextSaveFailed(String, String),
    #[error("Failed to request file modification: {0}")]
    FileModificationRequestFailed(String),
    #[error("Failed to notify file modified: {0}")]
    FileModifiedNotificationFailed(String),
    #[error("Failed to load snippet '{0}': {1}")]
    SnippetLoadFailed(String, String),
    #[error("Failed to call agent: {0}")]
    AgentCallFailed(String),
    #[error("Failed to summarize content: {0}")]
    SummaryFailed(String),
    #[error("Failed to get stdin: {0}")]
    StdinError(String),
    #[error("Failed to get stdout: {0}")]
    StdoutError(String),
    #[error("Failed to get stderr: {0}")]
    StderrError(String),
    #[error("Failed to spawn command: {0}")]
    CommandSpawnFailed(String),
    #[error("Command failed: {0}")]
    CommandFailed(String),
    #[error("Failed to convert command output to UTF-8: {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
    #[error("Unknown error")]
    Unknown,
}

pub fn execute(
    project: &Project,
    context_name: &str,
    agent: &ShellAgentCall,
    commit: &mut Commit,
) -> Result<()> {
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
    ) -> Result<()> {
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
    ) -> Result<bool> {
        if !visited_contexts.insert(context_name.to_string()) {
            debug!(
                "Context '{}' already visited. Skipping further execution.",
                context_name
            );
            return Ok(false);
        }

        let mut context = Context::load(project, context_name).map_err(|e| ExecuteError::ContextLoadFailed(context_name.to_string(), e.to_string()))?;
        
        // Transform tags into anchors and initialize states
        let mut need_next_step = self._handle_tags_and_anchors(project, &mut context, agent, visited_contexts, commit)?;

        // Handle repeat tag
        need_next_step |= self._handle_repeat_tag(project, &mut context, commit)?;
        
        // Execute document injection and mutate states
        need_next_step |= self._handle_anchor_states(project, &mut context, commit)?;

        // Save document, no more document changes on this step
        if context.modified {
            let uid = project.request_file_modification(&context.path).map_err(|e| ExecuteError::FileModificationRequestFailed(e.to_string()))?;
            context.save().map_err(|e| ExecuteError::ContextSaveFailed(context_name.to_string(), e.to_string()))?;
            commit.files.insert(context.path.clone());
            project.notify_file_modified(&context.path, uid).map_err(|e| ExecuteError::FileModifiedNotificationFailed(e.to_string()))?;
        }

        // Execute long tasks that only mutate state (and not document)
        need_next_step |= self._execute_long_tasks(project, &context, agent, visited_contexts, commit)?;

        Ok(need_next_step)
    }

    fn _handle_tags_and_anchors(
        &mut self,
        project: &Project,
        context: &mut Context,
        agent: &ShellAgentCall,
        visited_contexts: &mut HashSet<String>,
        commit: &mut Commit,
    ) -> Result<bool> {
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
                    project.save_inline_state(&uid, &state, commit).map_err(ExecuteError::ProjectError)?;                    
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
                    project.save_answer_state(&uid, &state, commit).map_err(ExecuteError::ProjectError)?;
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
                    project.save_summary_state(&uid, &state, commit).map_err(ExecuteError::ProjectError)?;
                    // Exit processing, summary could add content needed for further actions
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

       

    fn _handle_repeat_tag(
        &mut self,
        project: &Project,
        context: &mut Context,
        commit: &mut Commit,
    ) -> anyhow::Result<bool> {
        let mut modified = false;
        let mut patches = Patches::new();
        let mut latest_anchor = None;

        for (i, line) in context.lines.iter().enumerate() {
            match line {
                Line::RepeatTag => {
                    if let Some(j) = latest_anchor {
                        // Repeat inside some anchor, manage it
                        patches.insert((i, i+1), vec![]);
                        let anchor_begin_line = context.lines.get(j).unwrap(); // TODO gestione errore
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
                             _ => { /* Do nothing */}
                        }
                    } else {
                        // Repeat outside of any anchor, just remove it
                        patches.insert((i, i+1), vec![]);
                    }
                }               
                Line::AnswerBeginAnchor {..} | Line::InlineBeginAnchor { .. } | Line::SummaryBeginAnchor { .. } => {
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
    ) -> Result<bool> {
        let mut need_next_step = false;
        let mut lines_to_add: Vec<Line> = Vec::new();

        for line_idx in 0..context.lines.len() {
            let line = &context.lines[line_idx];
            if let Line::Anchor(anchor) = line {
                let uid = anchor.uid;
                let anchor_kind = anchor.kind.clone();
                let anchor_begin_line_number = anchor.begin_line;
                let anchor_end_line_number = anchor.end_line;

                let anchor_content = context.get_anchor_content(
                    anchor_begin_line_number,
                    anchor_end_line_number,
                );

                let new_state = AnchorState {
                    anchor_kind: anchor_kind.clone(),
                    content: anchor_content,
                    status: AnchorStatus::NeedContext,
                    context_name: context.name.clone(),
                    anchor_begin_line_number,
                    anchor_end_line_number,
                };

                match anchor_kind {
                    AnchorKind::Inline => {
                        let state = project.load_inline_state(&uid).map_err(ExecuteError::ProjectError)?;
                        if state.content != new_state.content {
                            project.save_inline_state(&uid, &new_state, commit).map_err(ExecuteError::ProjectError)?;
                            need_next_step = true;
                        }
                    }
                    AnchorKind::Answer => {
                        let state = project.load_answer_state(&uid).map_err(ExecuteError::ProjectError)?;
                        if state.content != new_state.content {
                            project.save_answer_state(&uid, &new_state, commit).map_err(ExecuteError::ProjectError)?;
                            need_next_step = true;
                        }
                    }
                    AnchorKind::Summary => {
                        let state = project.load_summary_state(&uid).map_err(ExecuteError::ProjectError)?;
                        if state.content != new_state.content {
                            project.save_summary_state(&uid, &new_state, commit).map_err(ExecuteError::ProjectError)?;
                            need_next_step = true;
                        }
                    }
                }
            }
        }
        Ok(need_next_step)
    }

    fn _execute_long_tasks(
        &mut self,
        project: &Project,
        context: &Context,
        agent: &ShellAgentCall,
        visited_contexts: &mut HashSet<String>,
        commit: &mut Commit,
    ) -> Result<bool> {
        let mut need_next_step = false;

        for line in &context.lines {
            if let Line::Anchor(anchor) = line {
                let uid = anchor.uid;
                let anchor_kind = anchor.kind.clone();

                match anchor_kind {
                    AnchorKind::Inline => {
                        let state = project.load_inline_state(&uid).map_err(ExecuteError::ProjectError)?;
                        if state.status == AnchorStatus::NeedContext {
                            let response = agent.call(&state.content).map_err(ExecuteError::AgentError)?;
                            let new_state = AnchorState {
                                status: AnchorStatus::Done,
                                content: response,
                                ..state
                            };
                            project.save_inline_state(&uid, &new_state, commit).map_err(ExecuteError::ProjectError)?;
                            need_next_step = true;
                        }
                    }
                    AnchorKind::Answer => {
                        let state = project.load_answer_state(&uid).map_err(ExecuteError::ProjectError)?;
                        if state.status == AnchorStatus::NeedContext {
                            let response = agent.call(&state.content).map_err(ExecuteError::AgentError)?;
                            let new_state = AnchorState {
                                status: AnchorStatus::Done,
                                content: response,
                                ..state
                            };
                            project.save_answer_state(&uid, &new_state, commit).map_err(ExecuteError::ProjectError)?;
                            need_next_step = true;
                        }
                    }
                    AnchorKind::Summary => {
                        let state = project.load_summary_state(&uid).map_err(ExecuteError::ProjectError)?;
                        if state.status == AnchorStatus::NeedContext {
                            let response = agent.call(&state.content).map_err(ExecuteError::AgentError)?;
                            let new_state = AnchorState {
                                status: AnchorStatus::Done,
                                content: response,
                                ..state
                            };
                            project.save_summary_state(&uid, &new_state, commit).map_err(ExecuteError::ProjectError)?;
                            need_next_step = true;
                        }
                    }
                }
            } else if let Line::RepeatTag(repeat_tag) = line {
                // Handle @repeat tag
                // For now, just mark as need_next_step to continue processing
                need_next_step = true;
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
