use anyhow::Result;
use std::fs;
use uuid::Uuid;

use crate::ast::parser;
use crate::ast::types::{Anchor, AnchorKind, AnchorTag, Line, LineKind, TagKind};
use crate::project::Project;
pub fn decorate_context(project: &Project, context_name: &str) -> Result<()> {
    let context_path = project.resolve_context(context_name);
    let original_content = fs::read_to_string(&context_path)?;
    let mut lines = parser::parse_document(&original_content).map_err(anyhow::Error::msg)?;

    let modified = decorate_context_in_memory(&mut lines)?;

    if modified {
        let new_content = parser::format_document(&lines);
        fs::write(&context_path, new_content)?;
    }

    Ok(())
}

pub fn decorate_context_in_memory(lines: &mut Vec<Line>) -> Result<bool> {
    let mut modified = false;

    // First Pass: Add missing :begin anchors
    for line in lines.iter_mut() {
        if let LineKind::Tagged { tag, .. } = &line.kind {
            let expected_anchor_kind = match tag {
                TagKind::Inline => Some(AnchorKind::Inline),
                TagKind::Answer => Some(AnchorKind::Answer),
                TagKind::Summary => Some(AnchorKind::Summary),
                _ => None,
            };

            if let Some(expected_kind) = expected_anchor_kind {
                let has_begin_anchor = line.anchor.as_ref().map_or(false, |a| {
                    a.kind == expected_kind && a.tag == AnchorTag::Begin
                });

                if !has_begin_anchor {
                    let new_uid = Uuid::new_v4();
                    line.anchor = Some(Anchor {
                        kind: expected_kind,
                        uid: new_uid,
                        tag: AnchorTag::Begin,
                    });
                    modified = true;
                }
            }
        }
    }

    // Second Pass: Add missing :end anchors immediately after their :begin counterparts
    let mut final_lines: Vec<Line> = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].clone();
        final_lines.push(line.clone());

        if let Some(anchor) = &line.anchor {
            if anchor.tag == AnchorTag::Begin {
                let mut has_matching_end = false;
                // Check if a corresponding :end anchor exists anywhere after this :begin anchor
                for j in (i + 1)..lines.len() {
                    if let Some(other_anchor) = &lines[j].anchor {
                        if other_anchor.kind == anchor.kind
                            && other_anchor.uid == anchor.uid
                            && other_anchor.tag == AnchorTag::End
                        {
                            has_matching_end = true;
                            break;
                        }
                    }
                }

                if !has_matching_end {
                    // Insert the missing :end anchor immediately after the current line
                    let end_anchor = Anchor {
                        kind: anchor.kind.clone(),
                        uid: anchor.uid,
                        tag: AnchorTag::End,
                    };
                    final_lines.push(Line {
                        kind: LineKind::Text(String::new()), // Empty line for the anchor
                        anchor: Some(end_anchor),
                    });
                    modified = true;
                }
            }
        }
        i += 1;
    }
    *lines = final_lines;
    Ok(modified)
}
