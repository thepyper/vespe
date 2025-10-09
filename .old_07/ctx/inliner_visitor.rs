use std::fs;
use std::path::PathBuf;

use crate::ast::{Context, Line, LineData, Snippet, Visitor};

pub struct InlinerVisitor {}

impl Visitor for InlinerVisitor {
    fn pre_visit_line(&mut self, line: &Line) {
        if let LineData::Inline(snippet) = &line.data {
            let source_file = line.source_file.clone();
            let source_line_number = line.source_line_number;

            let original_content = match fs::read_to_string(&source_file) {
                Ok(content) => content,
                Err(e) => {
                    eprintln!("Error reading file {:?}: {}", source_file, e);
                    return;
                }
            };

            let mut lines: Vec<String> = original_content.lines().map(String::from).collect();

            let snippet_content: Vec<String> = snippet.lines.iter()
                .map(|s_line| s_line.text.clone())
                .collect();

            if source_line_number < lines.len() {
                // Replace the @inline line with the snippet content
                lines.splice(source_line_number..=source_line_number, snippet_content);
            } else {
                eprintln!("Line number {} out of bounds for file {:?}", source_line_number, source_file);
                return;
            }

            let new_file_content = lines.join("\n");
            if let Err(e) = fs::write(&source_file, new_file_content) {
                eprintln!("Error writing to file {:?}: {}", source_file, e);
            }
        }
    }
}
