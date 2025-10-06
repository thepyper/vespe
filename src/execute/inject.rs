use std::collections::HashSet;
use crate::project::Project;
use crate::ast::types::{Line};
use crate::injector;
use serde::{Serialize, Deserialize};
use std::fs;
use anyhow::Context as AnyhowContext;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct InlineState {
    pub pasted: bool,
}

pub fn inject_recursive_inline(
    project: &mut Project,
    context_name: &str,
) -> anyhow::Result<()> {
    let mut inlined_set = HashSet::new();
    let mut context_lines = project.load_context_lines(context_name)?;

    _inject_recursive_inline(project, context_name, &mut context_lines, &mut inlined_set)?;

    project.update_context_lines(context_name, context_lines)?;
    Ok(())
}

fn _inject_recursive_inline(
    project: &mut Project,
    context_name: &str,
    context_lines: &mut Vec<Line>,
    inlined_set: &mut HashSet<String>,
) -> anyhow::Result<()> {
    if inlined_set.contains(context_name) {
        return Ok(());
    }
    inlined_set.insert(context_name.to_string());

    let mut lines_to_process = Vec::new();
    for (i, line) in context_lines.iter().enumerate() {
        if let Some((anchor_kind, anchor_uid, snippet_name)) = line.get_inline_tag_info() {
            lines_to_process.push((i, anchor_kind, anchor_uid, snippet_name));
        }
    }

    // Process in reverse order to avoid issues with index changes
    for (_i, anchor_kind, anchor_uid, snippet_name) in lines_to_process.into_iter().rev() {
        let anchor_metadata_dir = project.resolve_metadata(&anchor_kind.to_string(), &anchor_uid)?;
        let state_file_path = anchor_metadata_dir.join("state.json");

        let mut inline_state = InlineState::default();
        if state_file_path.exists() {
            let state_content = fs::read_to_string(&state_file_path)
                .context(format!("Failed to read state file: {}", state_file_path.display()))?;
            inline_state = serde_json::from_str(&state_content)
                .context(format!("Failed to parse state file: {}", state_file_path.display()))?;
        }

        if inline_state.pasted {
            // Skip injection if already pasted
            continue;
        }

        let snippet_lines = project.load_snippet_lines(&snippet_name)?;

        injector::inject_content_in_memory(
            context_lines,
            anchor_kind,
            anchor_uid,
            snippet_lines,
        )?;

        // Update state after successful injection
        inline_state.pasted = true;
        let updated_state_content = serde_json::to_string_pretty(&inline_state)
            .context("Failed to serialize InlineState")?;
        fs::write(&state_file_path, updated_state_content)
            .context(format!("Failed to write state file: {}", state_file_path.display()))?;
    }

    // Recursively inject for included contexts
    let mut includes_to_inject = Vec::new();
    for line in context_lines.iter() {
        if let Some(included_context_name) = line.get_include_path() {
            includes_to_inject.push(included_context_name.to_string());
        }
    }

    for included_context_name in includes_to_inject.into_iter() {
        let mut included_lines = project.load_context_lines(&included_context_name)?;
        _inject_recursive_inline(project, &included_context_name, &mut included_lines, inlined_set)?;
        project.update_context_lines(&included_context_name, included_lines)?;
    }

    Ok(())
}
