use anyhow::{Context as AnyhowContext, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use crate::project::{CONTEXT_EXTENSION, SNIPPET_EXTENSION};

#[derive(Debug, PartialEq, Clone)]
pub enum LineData {
    Text(String),
    Include(Context),
    Inline(Snippet),
    Answer,
    Summary(Context),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Line {
    pub line_number: usize,
    pub data: LineData,
    pub source_file: PathBuf,
    pub source_line_number: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Context {
    pub file_path: PathBuf,
    pub lines: Vec<Line>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Snippet {
    pub file_path: PathBuf,
    pub lines: Vec<Line>,
}

pub trait Visitor {
    fn pre_visit_context(&mut self, _context: &Context) {}
    fn post_visit_context(&mut self, _context: &Context) {}
    fn pre_visit_snippet(&mut self, _snippet: &Snippet) {}
    fn post_visit_snippet(&mut self, _snippet: &Snippet) {}
    fn pre_visit_line(&mut self, _line: &Line) {}
    fn post_visit_line(&mut self, _line: &Line) {}
}

pub trait ContextResolver {
    fn resolve_context(&self, name: &str, project_root: &Path, visited: &mut HashSet<PathBuf>) -> Result<Context>;
    fn resolve_snippet(&self, name: &str, project_root: &Path) -> Result<Snippet>;
}

pub fn walk(context: &Context, visitor: &mut impl Visitor) {
    visitor.pre_visit_context(context);
    for line in &context.lines {
        visitor.pre_visit_line(line);
        match &line.data {
            LineData::Include(child_context) => {
                walk(child_context, visitor);
            },
            LineData::Summary(child_context) => {
                walk(child_context, visitor);
            },
            LineData::Inline(child_snippet) => {
                visitor.pre_visit_snippet(child_snippet);
                for snippet_line in &child_snippet.lines {
                    visitor.pre_visit_line(snippet_line);
                    visitor.post_visit_line(snippet_line);
                }
                visitor.post_visit_snippet(child_snippet);
            },
            _ => {} // Text, Answer, etc.
        }
        visitor.post_visit_line(line);
    }
    visitor.post_visit_context(context);
}

pub trait VisitorTransform {
    fn transform_line(&mut self, line: &Line) -> Vec<Line>;
    fn transform_context(&mut self, context: &Context) -> Context {
        let transformed_lines = context.lines.iter().flat_map(|line| {
            self.transform_line(line)
        }).collect();
        Context {
            file_path: context.file_path.clone(),
            lines: transformed_lines,
        }
    }
    fn transform_snippet(&mut self, snippet: &Snippet) -> Snippet {
        let transformed_lines = snippet.lines.iter().flat_map(|line| {
            self.transform_line(line)
        }).collect();
        Snippet {
            file_path: snippet.file_path.clone(),
            lines: transformed_lines,
        }
    }
}

pub fn transform_context<T: VisitorTransform>(context: &Context, transformer: &mut T) -> Context {
    transformer.transform_context(context)
}

pub fn transform_snippet<T: VisitorTransform>(snippet: &Snippet, transformer: &mut T) -> Snippet {
    transformer.transform_snippet(snippet)
}


pub struct AstPrettyPrinter {
    pub output: String,
    indent_level: usize,
}

impl AstPrettyPrinter {
    pub fn new() -> Self {
        AstPrettyPrinter {
            output: String::new(),
            indent_level: 0,
        }
    }

    fn indent(&mut self) {
        for _ in 0..self.indent_level {
            self.output.push_str("  "); // Two spaces per indent level
        }
    }
}

impl Visitor for AstPrettyPrinter {
    fn pre_visit_context(&mut self, context: &Context) {
        self.indent();
        self.output.push_str(&format!("Context: {}
", context.file_path.display()));
        self.indent_level += 1;
    }

    fn post_visit_context(&mut self, _context: &Context) {
        self.indent_level -= 1;
    }

    fn pre_visit_snippet(&mut self, snippet: &Snippet) {
        self.indent();
        self.output.push_str(&format!("Snippet: {}
", snippet.file_path.display()));
        self.indent_level += 1;
    }

    fn post_visit_snippet(&mut self, _snippet: &Snippet) {
        self.indent_level -= 1;
    }

    fn pre_visit_line(&mut self, line: &Line) {
        self.indent();
        self.output.push_str(&format!("Line {}: ", line.line_number));
        match &line.data {
            LineData::Text(text) => self.output.push_str(&format!("Text: \"{}\"\n", text)),
            LineData::Include(context) => self.output.push_str(&format!("Include: {}\n", context.file_path.display())),
            LineData::Inline(snippet) => self.output.push_str(&format!("Inline: {}\n", snippet.file_path.display())),
            LineData::Answer => self.output.push_str("Answer\n"),
            LineData::Summary(context) => self.output.push_str(&format!("Summary: {}\n", context.file_path.display())),
        }
        self.indent_level += 1;
    }

    fn post_visit_line(&mut self, _line: &Line) {
        self.indent_level -= 1;
    }
}

fn parse_lines(
    content: &str,
    file_path: &Path,
    resolver: &impl ContextResolver,
    project_root: &Path,
    visited: &mut HashSet<PathBuf>,
) -> Result<Vec<Line>> {
    content
        .lines()
        .enumerate()
        .map(|(line_number, line_content)| {
            if let Some(context_name) = line_content.strip_prefix("@include ") {
                let child_context = resolver.resolve_context(context_name.trim(), project_root, visited)?;
                Ok(Line {
                    line_number,
                    data: LineData::Include(child_context),
                    source_file: file_path.to_path_buf(),
                    source_line_number: line_number,
                })
            } else if let Some(snippet_name) = line_content.strip_prefix("@inline ") {
                let child_snippet = resolver.resolve_snippet(snippet_name.trim(), project_root)?;
                Ok(Line {
                    line_number,
                    data: LineData::Inline(child_snippet),
                    source_file: file_path.to_path_buf(),
                    source_line_number: line_number,
                })
            } else if line_content.trim() == "@answer" {
                Ok(Line {
                    line_number,
                    data: LineData::Answer,
                    source_file: file_path.to_path_buf(),
                    source_line_number: line_number,
                })
            } else if let Some(context_name) = line_content.strip_prefix("@summary ") {
                let child_context = resolver.resolve_context(context_name.trim(), project_root, visited)?;
                Ok(Line {
                    line_number,
                    data: LineData::Summary(child_context),
                    source_file: file_path.to_path_buf(),
                    source_line_number: line_number,
                })
            } else {
                Ok(Line {
                    line_number,
                    data: LineData::Text(line_content.to_string()),
                    source_file: file_path.to_path_buf(),
                    source_line_number: line_number,
                })
            }
        })
        .collect()
}

pub fn build_context(
    resolver: &impl ContextResolver,
    project_root: &Path,
    current_path: &Path,
    visited: &mut HashSet<PathBuf>,
) -> Result<Context> {
    if visited.contains(current_path) {
        return Ok(Context {
            file_path: current_path.to_path_buf(),
            lines: Vec::new(),
        });
    }
    visited.insert(current_path.to_path_buf());

    let content = std::fs::read_to_string(current_path)
        .with_context(|| format!("Failed to read {:?}", current_path))?;

    let lines = parse_lines(&content, current_path, resolver, project_root, visited)?;

    Ok(Context {
        file_path: current_path.to_path_buf(),
        lines,
    })
}

pub fn build_snippet(
    resolver: &impl ContextResolver,
    project_root: &Path,
    current_path: &Path,
) -> Result<Snippet> {
    let content = std::fs::read_to_string(current_path)
        .with_context(|| format!("Failed to read {:?}", current_path))?;

    let mut dummy_visited = HashSet::new();
    let lines = parse_lines(&content, current_path, resolver, project_root, &mut dummy_visited)?;

    Ok(Snippet {
        file_path: current_path.to_path_buf(),
        lines,
    })
}

pub struct InlineExpander;

impl VisitorTransform for InlineExpander {
    fn transform_line(&mut self, line: &Line) -> Vec<Line> {
        match &line.data {
            LineData::Inline(snippet) => {
                // Recursively transform the snippet's lines in case it contains further inlines
                let transformed_snippet = self.transform_snippet(snippet);
                transformed_snippet.lines
            },
            _ => vec![line.clone()],
        }
    }
}

// Helper function to resolve context paths, moved from Project
pub fn resolve_context_path(project_root: &Path, name: &str) -> Result<PathBuf> {
    let contexts_dir = project_root.join("contexts");
    let path = contexts_dir.join(to_filename(name));
    if !path.is_file() {
        anyhow::bail!("Context '{}' does not exist", name);
    }
    Ok(path)
}

// Helper function to convert name to filename, moved from Project
pub fn to_filename(name: &str) -> String {
    if name.ends_with(format!(".{}", CONTEXT_EXTENSION).as_str()) {
        name.to_string()
    } else {
        format!("{}.{}", name, CONTEXT_EXTENSION)
    }
}

// Helper function to convert filename to name, moved from Project
pub fn to_name(name: &str) -> String {
    name.strip_suffix(format!(".{}", CONTEXT_EXTENSION).as_str()).unwrap_or(name).to_string()
}

// Helper function to resolve snippet paths
pub fn resolve_snippet_path(project_root: &Path, name: &str) -> Result<PathBuf> {
    let snippets_dir = project_root.join("snippets");
    let path = snippets_dir.join(to_snippet_filename(name));
    if !path.is_file() {
        anyhow::bail!("Snippet '{}' does not exist", name);
    }
    Ok(path)
}

// Helper function to convert snippet name to filename
pub fn to_snippet_filename(name: &str) -> String {
    if name.ends_with(format!(".{}", SNIPPET_EXTENSION).as_str()) {
        name.to_string()
    } else {
        format!("{}.{}", name, SNIPPET_EXTENSION)
    }
}
