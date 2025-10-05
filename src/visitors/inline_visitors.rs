use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::ast::line_processor::LineTransformer;
use crate::ast::types::{AnchorData, AnchorDataValue, AnchorKind, Context, Line, LineKind, Parameters, Snippet};

/// A trait for loading snippet content.
/// This allows the InlineCompleter to be independent of how snippets are stored or retrieved.
pub trait SnippetLoader {
    fn load_snippet(&self, path: &PathBuf) -> Option<Vec<Line>>;
}

/// InlineDecorator: Adds UUID anchors to @inline directives that don't have them.
pub struct InlineDecorator;

impl LineTransformer for InlineDecorator {
    fn transform_context_line(&self, line: &Line, _current_context_path: &PathBuf) -> Option<Vec<Line>> {
        self.transform_line(line)
    }

    fn transform_snippet_line(&self, line: &Line, _current_snippet_path: &PathBuf) -> Option<Vec<Line>> {
        self.transform_line(line)
    }
}

impl InlineDecorator {
    fn transform_line(&self, line: &Line) -> Option<Vec<Line>> {
        if let LineKind::Inline { snippet, parameters: _ } = &line.kind {
            // Check if an InlineBegin anchor already exists
            let has_begin_anchor = line.anchor.as_ref().map_or(false, |anchor| {
                anchor.kind == AnchorKind::Inline && anchor.data.as_ref().map_or(false, |data| *data == AnchorDataValue::Begin)
            });

            if !has_begin_anchor {
                let uuid = Uuid::new_v4();
                let snippet_path_str = snippet.path.to_string_lossy().into_owned();

                let begin_anchor_data = AnchorData {
                    kind: AnchorKind::Inline,
                    uid: uuid,
                    data: Some(AnchorDataValue::Custom(format!("begin:{}", snippet_path_str))),
                };
                let begin_anchor_line = Line {
                    kind: LineKind::Text, // Anchors are treated as text lines
                    text: begin_anchor_data.to_string(), // Use the Display impl
                    anchor: Some(begin_anchor_data),
                };

                let end_anchor_data = AnchorData {
                    kind: AnchorKind::Inline,
                    uid: uuid,
                    data: Some(AnchorDataValue::End),
                };
                let end_anchor_line = Line {
                    kind: LineKind::Text, // Anchors are treated as text lines
                    text: end_anchor_data.to_string(), // Use the Display impl
                    anchor: Some(end_anchor_data),
                };

                return Some(vec![begin_anchor_line, line.clone(), end_anchor_line]);
            }
        }
        None
    }
}

/// InlineCompleter: Replaces content between inline anchors with the actual snippet content.
pub struct InlineCompleter<'a> {
    snippet_loader: &'a dyn SnippetLoader,
    current_inline_block: Option<(Uuid, PathBuf)>, // (uuid, snippet_path)
}

impl<'a> InlineCompleter<'a> {
    pub fn new(snippet_loader: &'a dyn SnippetLoader) -> Self {
        Self {
            snippet_loader,
            current_inline_block: None,
        }
    }
}

impl<'a> LineTransformer for InlineCompleter<'a> {
    fn transform_context_line(&self, line: &Line, _current_context_path: &PathBuf) -> Option<Vec<Line>> {
        self.transform_line(line)
    }

    fn transform_snippet_line(&self, line: &Line, _current_snippet_path: &PathBuf) -> Option<Vec<Line>> {
        self.transform_line(line)
    }
}

impl<'a> InlineCompleter<'a> {
    fn transform_line(&mut self, line: &Line) -> Option<Vec<Line>> {
        if let Some((expected_uuid, expected_path)) = &self.current_inline_block {
            // We are inside an inline block
            if let Some(anchor) = &line.anchor {
                if anchor.kind == AnchorKind::Inline && anchor.uid == *expected_uuid {
                    if let Some(AnchorDataValue::End) = &anchor.data {
                        // Found the matching end anchor
                        let mut result_lines = Vec::new();
                        if let Some(snippet_lines) = self.snippet_loader.load_snippet(expected_path) {
                            result_lines.extend(snippet_lines);
                        } else {
                            // TODO: Log a warning if snippet not found
                            eprintln!("Warning: Snippet not found for path: {:?}", expected_path);
                        }
                        result_lines.push(line.clone()); // Keep the end anchor
                        self.current_inline_block = None; // Exit inline block mode
                        return Some(result_lines);
                    }
                }
            }
            // If we are inside an inline block and it's not the matching end anchor, delete the line
            return Some(vec![]);
        } else {
            // We are not inside an inline block, check for a begin anchor
            if let Some(anchor) = &line.anchor {
                if anchor.kind == AnchorKind::Inline {
                    if let Some(AnchorDataValue::Custom(data_str)) = &anchor.data {
                        if data_str.starts_with("begin:") {
                            let parts: Vec<&str> = data_str.splitn(2, ':').collect();
                            if parts.len() == 2 {
                                let snippet_path = PathBuf::from(parts[1]);
                                self.current_inline_block = Some((anchor.uid, snippet_path));
                                return Some(vec![line.clone()]); // Keep the begin anchor
                            }
                        }
                    }
                }
            }
        }
        None // No change
    }
}
