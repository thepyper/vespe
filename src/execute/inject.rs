use crate::ast::types::{AnchorKind, Line, TagKind, AnchorTag, Anchor};
use crate::injector;
use crate::project::{ContextManager, Project};
use anyhow::Context as AnyhowContext;
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, BTreeMap};
use std::fs;
use uuid::Uuid;
use crate::execute::apply_patches;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InlineState {
    pub pasted: bool,
}

impl Default for InlineState {
    fn default() -> Self {
        InlineState {
            pasted: false,
        }
    }
}

pub fn inject_recursive_inline(
    project: &Project,
    context_manager: &mut ContextManager,
    context_name: &str,
) -> anyhow::Result<()> {
    let mut inlined_set = HashSet::new();
    _inject_recursive_inline(project, context_manager, context_name, &mut inlined_set)
}

fn _inject_recursive_inline(
    project: &Project,
    context_manager: &mut ContextManager,
    context_name: &str,
    inlined_set: &mut HashSet<String>,
) -> anyhow::Result<()> {
    if inlined_set.contains(context_name) {
        return Ok(());
    }
    inlined_set.insert(context_name.to_string());

    let context_lines = context_manager.load_context(project, context_name)?;
    let mut lines_to_process = std::mem::take(context_lines); // Take ownership to modify and re-insert

    let mut modified_current_context = false;
    let mut patches = BTreeMap::<(usize, usize), Vec<Line>>::new();

    // Collect inline tags and their positions
    let mut inline_tags_info = Vec::new();
    for i in 0..lines_to_process.len() {
        if let Line::Tagged { tag: TagKind::Inline, arguments, .. } = &lines_to_process[i] {
            // Assuming the next line is the begin anchor
            if i + 1 < lines_to_process.len() {
                if let Line::Anchor(begin_anchor) = &lines_to_process[i + 1] {
                    if begin_anchor.tag == AnchorTag::Begin && begin_anchor.kind == AnchorKind::Inline {
                        inline_tags_info.push((i, begin_anchor.kind.clone(), begin_anchor.uid, arguments.first().unwrap().to_string()));
                    }
                }
            }
        }
    }

    // Process inline tags in reverse order to avoid index invalidation
    for (i, anchor_kind, anchor_uid, snippet_name) in inline_tags_info.into_iter().rev() {
        let anchor_metadata_dir =
            project.resolve_metadata(&anchor_kind.to_string(), &anchor_uid)?;
        let state_file_path = anchor_metadata_dir.join("state.json");

        let mut inline_state = InlineState::default();
        if state_file_path.exists() {
            let state_content = fs::read_to_string(&state_file_path)?;
            inline_state = serde_json::from_str(&state_content).context(format!(
                "Failed to deserialize InlineState from {}",
                state_file_path.display()
            ))?;
        }

        if inline_state.pasted {
            continue;
        }

        let snippet_lines = project.load_snippet_lines(&snippet_name)?;
        // Replace the inline tag and its begin anchor with the snippet content
        patches.insert((i, 2), snippet_lines); // Replace the tagged line and its begin anchor
        modified_current_context = true;

        inline_state.pasted = true;
        let updated_state_content = serde_json::to_string_pretty(&inline_state)
            .context("Failed to serialize InlineState")?;
        fs::write(&state_file_path, updated_state_content)?;
    }

    if !patches.is_empty() {
        apply_patches(&mut lines_to_process, patches)?;
    }

    // Recursively process included contexts
    for line in lines_to_process.iter() {
        if let Line::Tagged { tag: TagKind::Include, arguments, .. } = line {
            let included_context_name = arguments.first().unwrap().as_str();
            _inject_recursive_inline(
                project,
                context_manager,
                included_context_name,
                inlined_set,
            )?;
        }
    }

    *context_manager.load_context(project, context_name)? = lines_to_process; // Put back the modified lines
    if modified_current_context {
        context_manager.mark_as_modified(context_name);
    }

    Ok(())
}
