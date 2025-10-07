// src/injector.rs

use anyhow::Result;
use uuid::Uuid;

use crate::ast::parser;
use crate::ast::types::{AnchorKind, AnchorTag, Line};
use crate::project::Project;

pub fn inject_content(
    project: &Project,
    ctx_name: &str,
    anchor_kind: AnchorKind,
    anchor_uid: Uuid,
    new_content: Vec<Line>,
) -> Result<()> {
    let context_path = project.resolve_context(ctx_name);
    let content = std::fs::read_to_string(&context_path)?;
    let mut lines = parser::parse_document(&content).map_err(anyhow::Error::msg)?;

    let modified = inject_content_in_memory(&mut lines, anchor_kind, anchor_uid, new_content)?;

    if modified {
        let updated_content = parser::format_document(&lines);
        std::fs::write(&context_path, updated_content)?;
    }

    Ok(())
}

pub fn inject_content_in_memory(
    lines: &mut Vec<Line>,
    anchor_kind: AnchorKind,
    anchor_uid: Uuid,
    new_content: Vec<Line>,
) -> Result<bool> {
    let mut modified = false;
    let mut start_index = None;
    let mut end_index = None;

    // Find the begin and end anchors
    for (i, line) in lines.iter().enumerate() {
        if let Line::Anchor(anchor) = line {
            if anchor.kind == anchor_kind && anchor.uid == anchor_uid {
                match anchor.tag {
                    AnchorTag::Begin => start_index = Some(i),
                    AnchorTag::End => end_index = Some(i),
                    _ => {}
                }
            }
        }
    }

    if let (Some(start), Some(end)) = (start_index, end_index) {
        // Remove existing content between anchors
        lines.drain(start + 1..end);

        // Insert new content
        for (i, line) in new_content.into_iter().enumerate() {
            lines.insert(start + 1 + i, line);
        }
        modified = true;
    } else {
        return Err(anyhow::Error::msg(format!(
            "Could not find both begin and end anchors for kind {:?} and uid {}",
            anchor_kind, anchor_uid
        )));
    }
    Ok(modified)
}