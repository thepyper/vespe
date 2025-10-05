use std::collections::HashSet;
use std::path::PathBuf;

use super::types::{Context, Line, Snippet};
use super::visitor::Visitor;

pub trait LineTransformer {
    fn transform_context_line(&self, line: &Line, current_context_path: &PathBuf) -> Option<Vec<Line>>;
    fn transform_snippet_line(&self, line: &Line, current_snippet_path: &PathBuf) -> Option<Vec<Line>>;
}

pub struct LineModifyingVisitor<'a> {
    transformer: Box<dyn LineTransformer + 'a>,
    modified_files: HashSet<PathBuf>,
    current_file_path_stack: Vec<PathBuf>,
}

impl<'a> LineModifyingVisitor<'a> {
    pub fn new(transformer: Box<dyn LineTransformer + 'a>) -> Self {
        LineModifyingVisitor {
            transformer,
            modified_files: HashSet::new(),
            current_file_path_stack: Vec::new(),
        }
    }

    pub fn get_modified_files(&self) -> &HashSet<PathBuf> {
        &self.modified_files
    }
}

impl<'a> Visitor for LineModifyingVisitor<'a> {
    fn pre_visit_context(&mut self, context: &mut Context) {
        self.current_file_path_stack.push(context.path.clone());
    }

    fn post_visit_context(&mut self, _context: &mut Context) {
        self.current_file_path_stack.pop();
    }

    fn pre_visit_snippet(&mut self, snippet: &mut Snippet) {
        self.current_file_path_stack.push(snippet.path.clone());
    }

    fn post_visit_snippet(&mut self, _snippet: &mut Snippet) {
        self.current_file_path_stack.pop();
    }

    fn visit_context_lines(&mut self, context: &mut Context) {
        let current_path = self.current_file_path_stack.last().expect("Path stack should not be empty");
        let mut new_lines: Vec<Line> = Vec::new();
        let mut context_modified = false;

        for line in context.lines.drain(..) {
            if let Some(replacement_lines) = self.transformer.transform_context_line(&line, current_path) {
                new_lines.extend(replacement_lines);
                context_modified = true;
            } else {
                new_lines.push(line);
            }
        }
        context.lines = new_lines;
        if context_modified {
            self.modified_files.insert(context.path.clone());
        }
    }

    fn visit_snippet_lines(&mut self, snippet: &mut Snippet) {
        let current_path = self.current_file_path_stack.last().expect("Path stack should not be empty");
        let mut new_lines: Vec<Line> = Vec::new();
        let mut snippet_modified = false;

        for line in snippet.lines.drain(..) {
            if let Some(replacement_lines) = self.transformer.transform_snippet_line(&line, current_path) {
                new_lines.extend(replacement_lines);
                snippet_modified = true;
            } else {
                new_lines.push(line);
            }
        }
        snippet.lines = new_lines;
        if snippet_modified {
            self.modified_files.insert(snippet.path.clone());
        }
    }
}
