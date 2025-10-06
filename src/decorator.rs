use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use uuid::Uuid;

use crate::project::Project;
use crate::ast::parser::parse_document;
use crate::ast::types::{Line, LineKind, Anchor, AnchorKind, AnchorTag, TagKind};

pub fn decorate_context(project: &Project, context_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let context_path = project.resolve_context(context_name);
    let original_content = fs::read_to_string(&context_path)?;
    let mut lines = parse_document(&original_content)?;

    let mut modified = false;

    // First Pass: Add missing :begin anchors
    for line in lines.iter_mut() {
        if let LineKind::Tagged { tag, .. } = &line.kind {
            let expected_anchor_kind = match tag {
                TagKind::Inline => Some(AnchorKind::Inline),
                TagKind::Summary => Some(AnchorKind::Summary),
                TagKind::Answer => Some(AnchorKind::Answer),
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

    // Second Pass: Add missing :end anchors
    let mut processed_lines: Vec<Line> = Vec::new();
    let mut active_begin_anchors: HashMap<AnchorKind, Uuid> = HashMap::new(); // Tracks (kind -> uid) of currently open begin anchors

    for line in lines.into_iter() {
        if let Some(anchor) = &line.anchor {
            match anchor.tag {
                AnchorTag::Begin => {
                    // If there's an active begin anchor of the same kind, it means the previous one was not closed.
                    // Insert an end anchor for the previous one *before* adding the current line.
                    if let Some(prev_uid) = active_begin_anchors.get(&anchor.kind) {
                        let end_anchor = Anchor {
                            kind: anchor.kind.clone(),
                            uid: *prev_uid,
                            tag: AnchorTag::End,
                        };
                        processed_lines.push(Line {
                            kind: LineKind::Text(String::new()),
                            anchor: Some(end_anchor),
                        });
                        modified = true;
                        active_begin_anchors.remove(&anchor.kind);
                    }
                    // Now add the current begin anchor to active_begin_anchors
                    active_begin_anchors.insert(anchor.kind.clone(), anchor.uid);
                },
                AnchorTag::End => {
                    // If this end anchor matches an active begin anchor, close it.
                    if let Some(expected_uid) = active_begin_anchors.get(&anchor.kind) {
                        if *expected_uid == anchor.uid {
                            active_begin_anchors.remove(&anchor.kind);
                        }
                    }
                },
                _ => {},
            }
        }
        processed_lines.push(line);
    }

    // After processing all lines, close any remaining active begin anchors
    for (kind, uid) in active_begin_anchors {
        let end_anchor = Anchor {
            kind,
            uid,
            tag: AnchorTag::End,
        };
        processed_lines.push(Line {
            kind: LineKind::Text(String::new()),
            anchor: Some(end_anchor),
        });
        modified = true;
    }

    // Reconstruct content
    let mut new_content = String::new();
    for line in &processed_lines {
        new_content.push_str(&line.to_string());
        new_content.push('
');
    }

    if modified {
        fs::write(&context_path, new_content)?; 
    }

    Ok(())
}