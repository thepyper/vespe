use sha2::{Digest, Sha256};
use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    agent::ShellAgentCall,
    ast::types::{AnchorKind, Line, LineKind, TagKind},
    execute::{decorate, inject},
    injector,
    project::{ContextManager, Project},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnswerState {
    pub answered: bool,
}

impl Default for AnswerState {
    fn default() -> Self {
        Self { answered: false }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SummaryState {
    pub content_hash: String,
    pub summary_text: String,
}

impl Default for SummaryState {
    fn default() -> Self {
        Self {
            content_hash: String::new(),
            summary_text: String::new(),
        }
    }
}

fn hash_lines(lines: &[Line]) -> String {
    let mut hasher = Sha256::new();
    for line in lines {
        hasher.update(line.text_content().as_bytes());
    }
    format!("{:x}", hasher.finalize())
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
            LineKind::Tagged {
                tag: TagKind::Include,
                arguments,
                ..
            } => {
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
                let included_lines_for_agent =
                    context_manager.load_context(project, include_path)?;
                for included_line in included_lines_for_agent {
                    if !matches!(included_line.kind, LineKind::Tagged { .. })
                        && included_line.anchor.is_none()
                    {
                        current_context_for_agent.push(included_line.clone());
                    }
                }
                i += 1; // Move past the include tag
            }
            LineKind::Tagged {
                tag: TagKind::Summary,
                arguments,
                ..
            } => {
                let summary_uid_str = line
                    .anchor
                    .as_ref()
                    .map(|a| a.uid.to_string())
                    .unwrap_or_else(|| {
                        arguments.first().map(|s| s.to_string()).unwrap_or_default()
                    });
                let summary_uid_uuid = Uuid::parse_str(&summary_uid_str)
                    .map_err(|e| anyhow::anyhow!("Invalid UID for summary tag: {}", e))?;

                let anchor_kind = AnchorKind::Summary;
                let metadata_dir =
                    project.resolve_metadata(&anchor_kind.to_string(), &summary_uid_uuid)?;
                let state_file = metadata_dir.join("state.json");

                let mut summary_state: SummaryState = if state_file.exists() {
                    let content = std::fs::read_to_string(&state_file)?;
                    serde_json::from_str(&content)?
                } else {
                    SummaryState::default()
                };

                // Extract lines between TagKind::Summary and its AnchorTag::End
                let mut summary_block_lines: Vec<Line> = Vec::new();
                let mut j = i + 1;
                let mut end_anchor_found = false;
                while j < lines_to_process.len() {
                    let current_line = &lines_to_process[j];
                    if let Some(anchor) = &current_line.anchor {
                        if anchor.kind == AnchorKind::Summary
                            && anchor.uid == summary_uid_uuid
                            && anchor.tag == crate::ast::types::AnchorTag::End
                        {
                            end_anchor_found = true;
                            break;
                        }
                    }
                    summary_block_lines.push(current_line.clone());
                    j += 1;
                }

                if !end_anchor_found {
                    return Err(anyhow::anyhow!(
                        "Summary tag without matching end anchor: {}",
                        summary_uid_str
                    ));
                }

                // Generate a unique temporary context name for recursive processing
                let temp_summary_context_name = format!("summary_temp_{}", Uuid::new_v4());
                context_manager.insert_context(
                    temp_summary_context_name.clone(),
                    summary_block_lines.clone(),
                );

                // Recursively process the summary content
                let answered_in_summary = _answer_first_question_recursive(
                    project,
                    context_manager,
                    &temp_summary_context_name,
                    agent,
                    answered_set,
                )?;

                if answered_in_summary {
                    // A question was answered in the summary, so we return true
                    // and let the outer loop continue processing from where it left off.
                    // The temporary context will be removed later.
                    return Ok(true);
                }

                // Retrieve processed lines from the temporary context
                let processed_summary_lines = context_manager
                    .get_context_mut(&temp_summary_context_name)
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "Failed to retrieve processed summary lines for {}",
                            temp_summary_context_name
                        )
                    })?
                    .clone();
                context_manager.remove_context(&temp_summary_context_name); // Remove temporary entry

                let current_hash = hash_lines(&processed_summary_lines);

                if current_hash != summary_state.content_hash
                    || summary_state.summary_text.is_empty()
                {
                    let query_lines: Vec<String> = processed_summary_lines
                        .iter()
                        .map(|line| line.text_content())
                        .collect();
                    let query = query_lines.join("\n");

                    let agent_response = agent.call(&query)?;
                    summary_state.summary_text = agent_response;
                    summary_state.content_hash = current_hash;
                    std::fs::write(&state_file, serde_json::to_string_pretty(&summary_state)?)?;
                    context_manager.mark_as_modified(context_name);
                }

                // Inject the summary text
                let new_content_lines: Vec<Line> = summary_state
                    .summary_text
                    .lines()
                    .map(|s| Line {
                        kind: LineKind::Text(s.to_string()),
                        anchor: None,
                    })
                    .collect();

                let injected_modified = injector::inject_content_in_memory(
                    &mut lines_to_process,
                    AnchorKind::Summary,
                    summary_uid_uuid,
                    new_content_lines,
                )?;
                if injected_modified {
                    context_manager.mark_as_modified(context_name);
                }

                i = j + 1; // Move past the summary block and its end anchor
            }
            LineKind::Tagged {
                tag: TagKind::Answer,
                arguments: _,
                ..
            } => {
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

    let uid_str = line
        .anchor
        .as_ref()
        .map(|a| a.uid.to_string())
        .unwrap_or_else(|| arguments.first().map(|s| s.to_string()).unwrap_or_default());
    let uid_uuid = Uuid::parse_str(&uid_str)
        .map_err(|e| anyhow::anyhow!("Invalid UID for answer tag: {}", e))?;

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
