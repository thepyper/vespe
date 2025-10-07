use anyhow::Result;
use std::fs;
use uuid::Uuid;
use std::collections::BTreeMap;

use crate::ast::parser;
use crate::ast::types::{Anchor, AnchorKind, AnchorTag, Line, TagKind};
use crate::project::Project;
use crate::execute::apply_patches;

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
    let mut patches = BTreeMap::<(usize, usize), Vec<Line>>::new();

    // Check for missing tag anchors
    for i in 0..lines.len() {
        if let Line::Tagged { tag, .. } = &lines[i] {
            let expected_begin_anchor_kind = match tag {
                TagKind::Inline => Some(AnchorKind::Inline),
                TagKind::Answer => Some(AnchorKind::Answer),
                TagKind::Summary => Some(AnchorKind::Summary),
                _ => None,
            };
            if let Some(expected_begin_anchor_kind) = expected_begin_anchor_kind {
                let mut is_anchor_ok = false;
                if i + 1 < lines.len() {
                    if let Line::Anchor(anchor) = &lines[i + 1] {
                        if anchor.kind == expected_begin_anchor_kind && anchor.tag == AnchorTag::Begin {
                            is_anchor_ok = true;
                        }
                    }
                }

                if !is_anchor_ok {
                    patches.insert(
                        (i + 1, 0), // Insert after the current line
                        vec![Line::Anchor(Anchor {
                            kind: expected_begin_anchor_kind,
                            uid: Uuid::new_v4(),
                            tag: AnchorTag::Begin,
                        })],
                    );
                    modified = true;
                }
            }
        }
    }
    if !patches.is_empty() {
        apply_patches(lines, patches)?;
        patches = BTreeMap::new(); // Clear patches for next stage
    }

    // Check for orphan end anchors
    let mut final_lines: Vec<Line> = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].clone();
        final_lines.push(line.clone());

        if let Line::Anchor(anchor) = &line {
            if let AnchorTag::Begin = anchor.tag {
                let mut found_end = false;
                for j in (i + 1)..lines.len() {
                    if let Line::Anchor(other_anchor) = &lines[j] {
                        if other_anchor.uid == anchor.uid && other_anchor.tag == AnchorTag::End {
                            found_end = true;
                            break;
                        }
                    }
                }
                if !found_end {
                    // Insert the missing :end anchor immediately after the current line
                    final_lines.push(Line::Anchor(Anchor {
                        kind: anchor.kind.clone(),
                        uid: anchor.uid,
                        tag: AnchorTag::End,
                    }));
                    modified = true;
                }
            }
        }
        i += 1;
    }
    *lines = final_lines;

    Ok(modified)
}