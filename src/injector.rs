// src/injector.rs

use uuid::Uuid;

use crate::project::Project;
use crate::ast::parser::parse_document;
use crate::ast::types::{AnchorKind, AnchorTag, Line};

pub fn inject_content(
    project: &Project,
    ctx_name: &str,
    anchor_kind: AnchorKind,
    anchor_uid: Uuid,
    new_content: Vec<Line>,
) -> Result<(), Box<dyn std::error::Error>> {
    let context_path = project.resolve_context(ctx_name);
    let content = std::fs::read_to_string(&context_path)?;
    let mut lines = parse_document(&content)?;

    let mut start_index = None;
    let mut end_index = None;

    // Find the begin and end anchors
    for (i, line) in lines.iter().enumerate() {
        if let Some(anchor) = &line.anchor {
            if anchor.kind == anchor_kind && anchor.uid == anchor_uid {
                match anchor.tag {
                    AnchorTag::Begin => start_index = Some(i),
                    AnchorTag::End => end_index = Some(i),
                    _ => {},
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
    } else {
        return Err(format!(
            "Could not find both begin and end anchors for kind {:?} and uid {}",
            anchor_kind,
            anchor_uid
        )
        .into());
    }

    let updated_content = lines.into_iter().map(|line| line.to_string()).collect::<Vec<String>>().join("\n");
    std::fs::write(&context_path, updated_content)?;

    Ok(())
}