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

pub fn answer_first_question(
    project: &Project,
    context_manager: &mut ContextManager,
    context_name: &str,
    agent: &ShellAgentCall,
) -> anyhow::Result<bool> {
    let mut answered_set = HashSet::new();

    _answer_first_question_recursive(
        project,
        context_manager,
        context_name,
        agent,
        &mut answered_set,
    )
}

fn _answer_first_question_recursive(
    project: &Project,
    context_manager: &mut ContextManager,
    context_name: &str,
    agent: &ShellAgentCall,
    answered_set: &mut HashSet<String>,
) -> anyhow::Result<bool> {
    if !answered_set.insert(context_name.to_string()) {
        return Ok(false); // Already processed this context to prevent circular dependencies
    }

    let mut current_context_for_agent: Vec<Line> = Vec::new();

    let context_lines = context_manager.load_context(project, context_name)?;
    let mut lines_to_process = std::mem::take(context_lines); // Take ownership to modify and re-insert

    let mut i = 0;
    while i < lines_to_process.len() {
        let line = &lines_to_process[i];

        match &line.kind {
            LineKind::Tagged { tag: TagKind::Include, arguments, .. } => {
                let include_path = arguments.first().map(|s| s.as_str()).unwrap_or("");
                // Decorate the included lines
                decorate::decorate_recursive_file(project, context_manager, include_path)?;

                // Inject into the included lines
                inject::inject_recursive_inline(project, context_manager, include_path)?;

                let answered_in_included = _answer_first_question_recursive(
                    project,
                    context_manager,
                    include_path,
                    agent,
                    answered_set,
                )?;
                if answered_in_included {
                    return Ok(true); // A question was answered in an included context, stop and return true
                }
                // Append processed included lines to current_context_for_agent, excluding tags/anchors
                let included_lines_for_agent = context_manager.load_context(project, include_path)?;
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
            LineKind::Tagged { tag: TagKind::Answer, arguments: _, .. } => {
                let answered = _answer_and_mark_context(
                    project,
                    context_manager,
                    agent,
                    context_name,
                    line.clone(),
                    &current_context_for_agent,
                    &mut lines_to_process,
                )?;
                if answered {
                    return Ok(true);
                }
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

    *context_manager.load_context(project, context_name)? = lines_to_process; // Put back the modified lines
    Ok(false) // No question was answered in this context or its includes
}

fn _answer_and_mark_context(
    project: &Project,
    context_manager: &mut ContextManager,
    agent: &ShellAgentCall,
    context_name: &str,
    line: Line,
    current_context_for_agent: &Vec<Line>,
    lines_to_process: &mut Vec<Line>,
) -> anyhow::Result<bool> {
    let (tag_kind, arguments) = match &line.kind {
        LineKind::Tagged { tag, arguments, .. } => (tag, arguments),
        _ => unreachable!(), // Should not happen as we are in a TagKind::Answer block
    };

    let uid_str = line.anchor.as_ref().map(|a| a.uid.to_string()).unwrap_or_else(|| {
        arguments.first().map(|s| s.to_string()).unwrap_or_default()
    });
    let uid_uuid = Uuid::parse_str(&uid_str).map_err(|e| anyhow::anyhow!("Invalid UID for answer tag: {}", e))?;

    let anchor_kind = match tag_kind {
        TagKind::Answer => AnchorKind::Answer,
        _ => unreachable!(), // Should not happen as we are in a TagKind::Answer block
    };
    let metadata_dir = project.resolve_metadata(&anchor_kind.to_string(), &uid_uuid)?;
    let state_file = metadata_dir.join("state.json");

    let mut answer_state: AnswerState = if state_file.exists() {
        let content = std::fs::read_to_string(&state_file)?;
        serde_json::from_str(&content)?
    } else {
        AnswerState::default()
    };

    if !answer_state.answered {
        // The query is the accumulated context up to this point
        let query_lines: Vec<String> = current_context_for_agent
            .iter()
            .map(|line| line.text_content())
            .collect();
        let query = query_lines.join("\n");

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
            lines_to_process,
            AnchorKind::Answer,
            uid_uuid,
            new_content_lines,
        )?;
        if injected_modified {
            context_manager.mark_as_modified(context_name);
        }

        answer_state.answered = true;
        std::fs::write(&state_file, serde_json::to_string_pretty(&answer_state)?)?;
        *context_manager.load_context(project, context_name)? = lines_to_process.clone(); // Put back the modified lines
        return Ok(true); // A question was answered, stop and return true
    }
    Ok(false)
}