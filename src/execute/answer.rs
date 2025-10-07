use sha2::{Digest, Sha256};
use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    agent::ShellAgentCall,
    ast::types::{AnchorKind, AnchorTag, Line, LineKind, TagKind},
    execute::{decorate, inject},
    injector,
    project::{ContextManager, Project},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnswerState {
    pub content_hash: String,
    pub reply_hash: String,
    pub reply: String,
    pub injected_hash: String,
}

impl Default for AnswerState {
    fn default() -> Self {
        AnswerState {
            content_hash: String::new(),
            reply_hash: String::new(),
            reply: String::new(),
            injected_hash: String::new(),
        }
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
        hasher.update(line.get_text_content().as_bytes());
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
        context_name,
        agent,
        context_manager,
        &mut answered_set,
    )
}

fn _answer_first_question_recursive(
    project: &Project,
    context_name: &str,
    agent: &ShellAgentCall,
    context_manager: &mut ContextManager,
    answered_questions: &mut HashSet<String>,
) -> anyhow::Result<bool> {
    if answered_questions.contains(context_name) {
        return Ok(false);
    }
    answered_questions.insert(context_name.to_string());

    let mut current_context_for_agent: Vec<Line> = Vec::new();
    let context_lines = context_manager.load_context(project, context_name)?;
    let mut lines_to_process = std::mem::take(context_lines); // Take ownership to modify and re-insert

    let mut i = 0;
    let mut modified_current_context = false;
    while i < lines_to_process.len() {
        let line = &lines_to_process[i];

        match line {
            Line::Tagged {
                tag: TagKind::Include,
                arguments,
                .. 
            } => {
                let included_context_name = arguments.first().unwrap().as_str();
                // Decorate the included lines
                decorate::decorate_recursive_file(project, context_manager, included_context_name)?;
                // Inject into the included lines
                inject::inject_recursive_inline(project, context_manager, included_context_name)?;
                // Answer questions in the included lines
                let modified_included = _answer_first_question_recursive(
                    project,
                    included_context_name,
                    agent,
                    context_manager,
                    answered_questions,
                )?;
                if modified_included {
                    modified_current_context = true;
                }

                // Append processed included lines to current_context_for_agent, excluding tags/anchors
                let included_lines_for_agent = context_manager
                    .load_context(project, included_context_name)?;
                for included_line in included_lines_for_agent {
                    if !matches!(included_line, Line::Tagged { .. })
                        && !matches!(included_line, Line::Anchor(_))
                    {
                        current_context_for_agent.push(included_line.clone());
                    }
                }
            }
            Line::Tagged {
                tag: TagKind::Summary,
                arguments,
                .. 
            } => {
                let summary_target_name = arguments.first().unwrap().as_str();
                let summary_uid_str = if let Some(Line::Anchor(anchor)) = lines_to_process.get(i + 1) {
                    anchor.uid.to_string()
                } else {
                    Uuid::new_v4().to_string()
                };
                let summary_anchor_kind = AnchorKind::Summary;
                let summary_metadata_dir =
                    project.resolve_metadata(&summary_anchor_kind.to_string(), &Uuid::parse_str(&summary_uid_str)?)?;
                let state_file_path = summary_metadata_dir.join("state.json");

                let mut summary_state = SummaryState::default();
                if state_file_path.exists() {
                    let content = std::fs::read_to_string(&state_file_path)?;
                    summary_state = serde_json::from_str(&content)?;
                }

                // Extract lines between TagKind::Summary and its AnchorTag::End
                let mut summary_block_lines: Vec<Line> = Vec::new();
                let mut j = i + 1;
                while j < lines_to_process.len() {
                    let current_line = &lines_to_process[j];
                    if let Line::Anchor(anchor) = current_line {
                        if anchor.uid.to_string() == summary_uid_str
                            && anchor.tag == AnchorTag::End
                        {
                            break;
                        }
                    }
                    summary_block_lines.push(current_line.clone());
                    j += 1;
                }

                let current_hash = hash_lines(&summary_block_lines);

                if summary_state.content_hash != current_hash || summary_state.summary_text.is_empty() {
                    // Create a temporary context for summarization
                    let temp_context_name = format!("_summary_{}", summary_uid_str);
                    context_manager.insert_context(temp_context_name.clone(), summary_block_lines.clone());

                    // Recursively process the temporary context
                    _answer_first_question_recursive(
                        project,
                        &temp_context_name,
                        agent,
                        context_manager,
                        answered_questions,
                    )?;

                    // Retrieve processed lines from the temporary context
                    let processed_summary_lines = context_manager
                        .remove_context(&temp_context_name)
                        .ok_or_else(|| {
                            anyhow::anyhow!(
                                "Failed to retrieve processed summary lines for {}",
                                temp_context_name
                            )
                        })?;

                    let query_lines: Vec<String> = processed_summary_lines
                        .iter()
                        .filter(|l| !matches!(l, Line::Tagged { .. }) && !matches!(l, Line::Anchor(_)))
                        .map(|line| {
                            if let Line::Text(s) = line {
                                s.clone()
                            } else {
                                "".to_string()
                            }
                        })
                        .collect();
                    let query = query_lines.join("\n");

                    let summary_text = agent.call(&query)?;

                    summary_state.content_hash = current_hash;
                    summary_state.summary_text = summary_text;
                    let updated_state_content = serde_json::to_string_pretty(&summary_state)?;
                    std::fs::write(&state_file_path, updated_state_content)?;
                    modified_current_context = true;
                }

                let new_content_lines: Vec<Line> = summary_state
                    .summary_text
                    .lines()
                    .map(|s| Line::Text(s.to_string()))
                    .collect();

                // Replace the summary block with the generated summary
                lines_to_process.splice(i + 1..j, new_content_lines);
                i += new_content_lines.len(); // Adjust index for inserted lines
            }
            Line::Tagged {
                tag: TagKind::Answer,
                arguments,
                .. 
            } => {
                let q_uid = if let Some(Line::Anchor(anchor)) = lines_to_process.get(i + 1) {
                    anchor.uid.to_string()
                } else {
                    arguments.first().unwrap().to_string() // Fallback to argument if no anchor
                };
                let q_uid_uuid = Uuid::parse_str(&q_uid)?;

                let answer_anchor_kind = AnchorKind::Answer;
                let answer_metadata_dir =
                    project.resolve_metadata(&answer_anchor_kind.to_string(), &q_uid_uuid)?;
                let state_file_path = answer_metadata_dir.join("state.json");

                let mut answer_state = AnswerState::default();
                if state_file_path.exists() {
                    let content = std::fs::read_to_string(&state_file_path)?;
                    answer_state = serde_json::from_str(&content)?;
                }

                if answer_state.content_hash != hash_lines(&current_context_for_agent)
                    || answer_state.reply.is_empty()
                {
                    let query_lines: Vec<String> = current_context_for_agent
                        .iter()
                        .filter(|l| !matches!(l, Line::Tagged { .. }) && !matches!(l, Line::Anchor(_)))
                        .map(|line| {
                            if let Line::Text(s) = line {
                                s.clone()
                            } else {
                                "".to_string()
                            }
                        })
                        .collect();
                    let query = query_lines.join("\n");

                    let answer_text = agent.call(&query)?;

                    answer_state.content_hash = hash_lines(&current_context_for_agent);
                    answer_state.reply = answer_text;
                    let updated_state_content = serde_json::to_string_pretty(&answer_state)?;
                    std::fs::write(&state_file_path, updated_state_content)?;
                    modified_current_context = true;
                }

                let new_content_lines: Vec<Line> = answer_state
                    .reply
                    .lines()
                    .map(|s| Line::Text(s.to_string()))
                    .collect();

                // Replace the answer tag with the generated answer
                lines_to_process.splice(i..i + 1, new_content_lines);
                i += new_content_lines.len(); // Adjust index for inserted lines
            }
            Line::Text(_) => {
                // Append text lines to current_context_for_agent
                current_context_for_agent.push(line.clone());
                i += 1;
            }
            Line::Anchor(_) => {
                // Skip anchors for current_context_for_agent
                i += 1;
            }
        }
    }

    *context_manager.load_context(project, context_name)? = lines_to_process; // Put back the modified lines
    if modified_current_context {
        context_manager.mark_as_modified(context_name);
    }

    Ok(modified_current_context)
}

/*
// Commented out _answer_and_mark_context for now as it's not called and causes errors.
fn _answer_and_mark_context(
    project: &Project,
    context_manager: &mut ContextManager,
    agent: &ShellAgentCall,
    context_name: &str,
    line: Line,
    current_context_for_agent: &Vec<Line>,
    lines_to_process: &mut Vec<Line>,
) -> anyhow::Result<bool> {
    let (tag_kind, arguments) = match line {
        Line::Tagged { tag, arguments, .. } => (tag, arguments),
        _ => unreachable!(), // Should not happen as we are in a TagKind::Answer block
    };

    let uid_str = if let Line::Anchor(anchor) = line {
        anchor.uid.to_string()
    } else {
        arguments.first().map(|s| s.to_string()).unwrap_or_default()
    };
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
            .map(|line| line.get_text_content())
            .collect();
        let query = query_lines.join("\n");

        // Call the agent
        let agent_response = agent.call(&query)?;
        // Inject the agent's response
        let new_content_lines: Vec<Line> = agent_response
            .lines()
            .map(|s| Line::Text(s.to_string()))
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
*/