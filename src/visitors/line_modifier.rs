use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::ast::{Context, Line, Visitor};

pub trait LineProcessor {
    fn process_line(&self, line: &Line, current_context_path: &PathBuf) -> Option<String>;
}

pub struct LineModifyingVisitor<'a, T: LineProcessor> {
    line_processor: &'a T,
    modified_files_content: HashMap<PathBuf, Vec<String>>,
    current_file_path_stack: Vec<PathBuf>,
    modified_files_set: HashSet<PathBuf>,
}

impl<'a, T: LineProcessor> LineModifyingVisitor<'a, T> {
    pub fn new(line_processor: &'a T) -> Self {
        Self {
            line_processor,
            modified_files_content: HashMap::new(),
            current_file_path_stack: Vec::new(),
            modified_files_set: HashSet::new(),
        }
    }

    pub fn write_modified_files(&self) -> Result<(), std::io::Error> {
        for path in &self.modified_files_set {
            if let Some(lines) = self.modified_files_content.get(path) {
                let content = lines.join(""); // Lines already have EOL
                std::fs::write(path, content)?;
            }
        }
        Ok(())
    }
}

impl<'a, T: LineProcessor> Visitor for LineModifyingVisitor<'a, T> {
    fn pre_visit_context(&mut self, context: &Context) {
        self.current_file_path_stack.push(context.path.clone());
        self.modified_files_content
            .entry(context.path.clone())
            .or_insert_with(Vec::new);
    }

    fn post_visit_context(&mut self, _context: &Context) {
        self.current_file_path_stack.pop();
    }

    fn pre_visit_line(&mut self, line: &Line) {
        let current_file_path = self
            .current_file_path_stack
            .last()
            .expect("current_file_path_stack should not be empty when visiting a line");

        let processed_content = self.line_processor.process_line(line, current_file_path);

        let entry = self
            .modified_files_content
            .get_mut(current_file_path)
            .expect("File content vector should exist");

        match processed_content {
            Some(mut new_content) => {
                if new_content.is_empty() {
                    // Skip adding empty lines
                } else if !new_content.ends_with("\n") {
                    new_content.push_str("\n");
                }
                entry.push(new_content);
                self.modified_files_set.insert(current_file_path.clone());
            }
            None => {
                entry.push(line.text.clone());
            }
        }
    }
}