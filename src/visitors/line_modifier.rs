use std::{collections::HashMap, path::PathBuf};
use std::io::Read;

use crate::ast::{Line, Context, Visitor};

/// A trait for processing individual lines within a context.
pub trait LineProcessor {
    /// Processes a single line.
    ///
    /// If `Some(new_content)` is returned, the original line is replaced by `new_content`.
    /// If `None` is returned, the original line is kept as is.
    /// The `new_content` should include an end-of-line character if it's not empty.
    fn process_line(&self, line: &Line, current_context_path: &PathBuf) -> Option<String>;
}

/// A visitor that processes lines using a `LineProcessor` and collects modified content.
pub struct LineModifyingVisitor<'a, P: LineProcessor> {
    line_processor: &'a P,
    /// Temporary buffer to build the lines for the *current* file being visited.
    current_file_rebuilt_lines: Vec<String>,
    /// Stores the final, rebuilt content for files that have been modified.
    modified_files_content: HashMap<PathBuf, Vec<String>>,
    /// Stack to keep track of the current context's path for nested contexts.
    context_path_stack: Vec<PathBuf>,
}

impl<'a, P: LineProcessor> LineModifyingVisitor<'a, P> {
    pub fn new(line_processor: &'a P) -> Self {
        Self {
            line_processor,
            current_file_rebuilt_lines: Vec::new(),
            modified_files_content: HashMap::new(),
            context_path_stack: Vec::new(),
        }
    }

    /// Helper to read a file's content as a single string.
    fn read_file_content_as_string(path: &PathBuf) -> Result<String, std::io::Error> {
        std::fs::read_to_string(path)
    }

    /// Writes all modified files back to the filesystem.
    pub fn write_modified_files(&self) -> Result<(), std::io::Error> {
        for (path, lines) in &self.modified_files_content {
            let content = lines.join("");
            std::fs::write(path, content)?;
        }
        Ok(())
    }
}

impl<'a, P: LineProcessor> Visitor for LineModifyingVisitor<'a, P> {
    fn pre_visit_context(&mut self, context: &Context) {
        self.context_path_stack.push(context.path.clone());
        self.current_file_rebuilt_lines.clear(); // Start fresh for each new context file
    }

    fn post_visit_context(&mut self, context: &Context) {
        if let Some(current_context_path) = self.context_path_stack.pop() {
            let rebuilt_content_string = self.current_file_rebuilt_lines.join("");

            match Self::read_file_content_as_string(&current_context_path) {
                Ok(original_content_string) => {
                    if rebuilt_content_string != original_content_string {
                        self.modified_files_content.insert(current_context_path, self.current_file_rebuilt_lines.clone());
                    }
                },
                Err(e) => {
                    eprintln!("Error reading original file {}: {}", current_context_path.display(), e);
                    // If we can't read the original, we can't compare. For now, we'll skip writing if original can't be read.
                }
            }
        }
    }

    fn visit_line(&mut self, line: &Line) {
        if let Some(current_context_path) = self.context_path_stack.last() {
            let original_line_with_eol = format!("{}{}", line.text, line.eol);
            let processed_result = self.line_processor.process_line(line, current_context_path);

            match processed_result {
                Some(new_content) => {
                    // Ensure new_content has an EOL if it's not empty
                    if !new_content.is_empty() && !new_content.ends_with('\n') && !new_content.ends_with('\r') {
                        self.current_file_rebuilt_lines.push(format!("{}\n", new_content));
                    } else {
                        self.current_file_rebuilt_lines.push(new_content);
                    }
                },
                None => {
                    // If None, keep the original line
                    self.current_file_rebuilt_lines.push(original_line_with_eol);
                }
            }
        } else {
            eprintln!("Warning: visit_line called without a current context path on stack.");
        }
    }
}
