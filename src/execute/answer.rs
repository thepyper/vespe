use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    agent::ShellAgentCall,
    ast::types::{AnchorKind, Line, LineKind, TagKind},
    injector,
    project::{Project, ContextManager},
    execute::{decorate, inject},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct AnswerState {
    pub answered: bool,
}

impl Default for AnswerState {
    fn default() -> Self {
        Self { answered: false }
    }
}

pub fn answer_questions(
    project: &Project,
    context_manager: &mut ContextManager,
    context_name: &str,
    agent: &ShellAgentCall,
) -> anyhow::Result<bool> {
    let mut answered_set = HashSet::new();

    let modified = _answer_questions_recursive(
        project,
        context_manager,
        context_name,
        agent,
        &mut answered_set,
    )?;

    if modified {
        context_manager.mark_as_modified(context_name);
    }

    Ok(modified)
}

fn _answer_questions_recursive(
    project: &Project,
    context_manager: &mut ContextManager,
    context_name: &str,
    agent: &ShellAgentCall,
    answered_set: &mut HashSet<String>,
) -> anyhow::Result<bool> {
    if !answered_set.insert(context_name.to_string()) {
        return Ok(false); // Already processed this context to prevent circular dependencies
    }

    let mut modified = false;
    let mut current_context_for_agent: Vec<Line> = Vec::new();

    let context_lines = context_manager.get_context(context_name)?;
    let mut lines_to_process = std::mem::take(context_lines); // Take ownership to modify and re-insert

    let mut i = 0;
    while i < lines_to_process.len() {
        let line = &lines_to_process[i];

        match &line.kind {
            LineKind::Tagged { tag: TagKind::Include, arguments, .. } => {
                let include_path = arguments.first().map(|s| s.as_str()).unwrap_or("");
                // Decorate the included lines
                let decorated_modified = decorate::decorate_recursive_file(project, context_manager, include_path)?;
                if decorated_modified {
                    context_manager.mark_as_modified(include_path);
                    modified = true;
                }

                // Inject into the included lines
                let injected_modified = inject::inject_recursive_inline(project, context_manager, include_path)?;
                if injected_modified {
                    context_manager.mark_as_modified(include_path);
                    modified = true;
                }

                let included_modified_by_answer = _answer_questions_recursive(
                    project,
                    context_manager,
                    include_path,
                    agent,
                    answered_set,
                )?;
                if included_modified_by_answer {
                    context_manager.mark_as_modified(include_path);
                    modified = true;
                }
                // Append processed included lines to current_context_for_agent, excluding tags/anchors
                let included_lines_for_agent = context_manager.get_context(include_path)?;
                for included_line in included_lines_for_agent {
                    if !matches!(included_line.kind, LineKind::Tagged { .. }) && included_line.anchor.is_none() {
                        current_context_for_agent.push(included_line.clone());
                    }
                }
                i += 1; // Move past the include tag
            }
            LineKind::Tagged { tag: TagKind::Summary, arguments, .. } => {
                let summary_content = arguments.first().map(|s| s.as_str()).unwrap_or("");
                current_context_for_agent.push(Line {
                    kind: LineKind::Text(summary_content.to_string()),
                    anchor: None,
                });
                i += 1; // Move past the summary tag
            }
            LineKind::Tagged { tag: TagKind::Answer, arguments, .. } => {
                let uid = arguments.first().map(|s| s.as_str()).unwrap_or("");
                let anchor_kind = "answer";
                let uid_uuid = Uuid::parse_str(uid).map_err(|e| anyhow::anyhow!("Invalid UID for answer tag: {}", e))?;
                let metadata_dir = project.resolve_metadata(anchor_kind, &uid_uuid)?;
                let state_file = metadata_dir.join("state.json");

                let mut answer_state: AnswerState = if state_file.exists() {
                    let content = std::fs::read_to_string(&state_file)?;
                    serde_json::from_str(&content)?
                } else {
                    AnswerState::default()
                };

                if !answer_state.answered {
                    let uid_uuid = Uuid::parse_str(uid).map_err(|e| anyhow::anyhow!("Invalid UID for answer tag: {}", e))?;

                    // Extract the question from the answer tag line itself
                    let question_line = lines_to_process[i].clone();
                    let question_text = if let LineKind::Tagged { tag: TagKind::Answer, arguments: q_uid_args, .. } = &question_line.kind {
                        let q_uid = q_uid_args.first().map(|s| s.as_str()).unwrap_or("");
                        format!("Question ({}): {}", q_uid, question_line.text_content())
                    } else {
                        question_line.text_content()
                    };

                    let query_lines: Vec<String> = current_context_for_agent
                        .iter()
                        .map(|line| line.text_content())
                        .chain(std::iter::once(question_text))
                        .collect();
                    let query = query_lines.join("\n"); // Join with escaped newline for agent

                    // Call the agent
                    let agent_response = agent.call(&query)?;

                    // Inject the agent's response
                    let new_content_lines: Vec<Line> = agent_response
                        .lines()
                        .map(|s| Line {
                            kind: LineKind::Text(s.to_string()),
                            anchor: None,
                        })
                        .collect();

                    let injected_modified = injector::inject_content_in_memory(
                        &mut lines_to_process,
                        AnchorKind::Answer,
                        uid_uuid,
                        new_content_lines,
                    )?;
                    if injected_modified {
                        modified = true;
                    }

                    answer_state.answered = true;
                    std::fs::write(&state_file, serde_json::to_string_pretty(&answer_state)?)?;
                }
                // Append the answer content to current_context_for_agent
                // This assumes the injected content is now part of lines_to_process
                // and will be processed in subsequent iterations or appended if it's after the current 'i'
                i += 1; // Move past the answer tag
            }
            LineKind::Text(_) => {
                // Append text lines to current_context_for_agent
                current_context_for_agent.push(line.clone());
                i += 1;
            }
            _ => {
                // Skip other tagged lines and anchors for current_context_for_agent
                i += 1;
            }
        }
    }

    *context_manager.get_context(context_name)? = lines_to_process; // Put back the modified lines
    Ok(modified)
}
