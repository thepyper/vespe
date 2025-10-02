use anyhow::{Context as AnyhowContext, Result};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::ast::{Context, Line, LineData, Snippet, Visitor};

pub struct InlinerVisitor {
    // Stores changes to be applied: file_path -> (line_number_to_replace, new_content)
    changes: HashMap<PathBuf, Vec<(usize, String)>>,
    // Stores original file contents to reconstruct files after inlining
    original_contents: HashMap<PathBuf, String>,
}

impl InlinerVisitor {
    pub fn new() -> Self {
        InlinerVisitor {
            changes: HashMap::new(),
            original_contents: HashMap::new(),
        }
    }

    pub fn apply_inlines(&self) -> Result<()> {
        for (file_path, line_changes) in &self.changes {
            let original_content = self.original_contents.get(file_path)
                .with_context(|| format!("Original content for {:?} not found", file_path))?;

            let mut lines: Vec<String> = original_content.lines().map(String::from).collect();

            // Apply changes in reverse order of line number to avoid issues with shifting line numbers
            let mut sorted_line_changes = line_changes.clone();
            sorted_line_changes.sort_by(|a, b| b.0.cmp(&a.0));

            for (line_number, new_content) in sorted_line_changes {
                if line_number < lines.len() {
                    lines[line_number] = new_content;
                } else {
                    anyhow::bail!("Line number {} out of bounds for file {:?}", line_number, file_path);
                }
            }

            let new_file_content = lines.join("\n");
            fs::write(file_path, new_file_content)
                .with_context(|| format!("Failed to write to file {:?}", file_path))?;
        }
        Ok(())
    }
}

impl Visitor for InlinerVisitor {
    fn pre_visit_line(&mut self, line: &Line) {
        if let LineData::Inline(snippet) = &line.data {
            let source_file = line.source_file.clone();
            let source_line_number = line.source_line_number;

            // Store original content if not already stored
            if !self.original_contents.contains_key(&source_file) {
                if let Ok(content) = fs::read_to_string(&source_file) {
                    self.original_contents.insert(source_file.clone(), content);
                }
            }

            let new_content = snippet.lines.iter()
                .map(|s_line| {
                    if let LineData::Text(text) = &s_line.data {
                        text.clone()
                    } else {
                        // If a snippet contains non-text data, we'll just represent it as an empty string for now.
                        // This might need more sophisticated handling depending on requirements.
                        String::new()
                    }
                })
                .collect::<Vec<String>>()
                .join("\n");

            self.changes.entry(source_file)
                .or_default()
                .push((source_line_number, new_content));
        }
    }
}
