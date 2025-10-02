use anyhow::{Context as AnyhowContext, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use crate::project::{CONTEXT_EXTENSION, SNIPPET_EXTENSION};

#[derive(Debug, PartialEq, Clone)]
pub enum LineData {
    Text(String),
    Include(String),
    Inline(String),
    Answer,
    Summary(String),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Line {
    pub line_number: usize,
    pub data: LineData,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Context {
    pub file_path: PathBuf,
    pub lines: Vec<AstNode>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Snippet {
    pub file_path: PathBuf,
    pub lines: Vec<AstNode>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AstNode {
    Context(Context),
    Line(Line),
    Snippet(Snippet),
}

pub trait Visitor {
    fn pre_visit_context(&mut self, context: &Context) {}
    fn post_visit_context(&mut self, context: &Context) {}
    fn pre_visit_snippet(&mut self, snippet: &Snippet) {}
    fn post_visit_snippet(&mut self, snippet: &Snippet) {}
    fn pre_visit_line(&mut self, line: &Line) {}
    fn post_visit_line(&mut self, line: &Line) {}
}

pub fn walk(node: &AstNode, visitor: &mut impl Visitor) {
    match node {
        AstNode::Context(context) => {
            visitor.pre_visit_context(context);
            for child_node in &context.lines {
                walk(child_node, visitor);
            }
            visitor.post_visit_context(context);
        },
        AstNode::Snippet(snippet) => {
            visitor.pre_visit_snippet(snippet);
            for child_node in &snippet.lines {
                walk(child_node, visitor);
            }
            visitor.post_visit_snippet(snippet);
        },
        AstNode::Line(line) => {
            visitor.pre_visit_line(line);
            visitor.post_visit_line(line);
        },
    }
}

fn parse_lines(content: &str) -> Vec<Line> {
    content
        .lines()
        .enumerate()
        .map(|(line_number, line)| {
            if let Some(context_name) = line.strip_prefix("@include ") {
                Line {
                    line_number,
                    data: LineData::Include(context_name.trim().to_string()),
                }
            } else if let Some(snippet_name) = line.strip_prefix("@inline ") {
                Line {
                    line_number,
                    data: LineData::Inline(snippet_name.trim().to_string()),
                }
            } else if line.trim() == "@answer" {
                Line {
                    line_number,
                    data: LineData::Answer,
                }
            } else if let Some(context_name) = line.strip_prefix("@summary ") {
                Line {
                    line_number,
                    data: LineData::Summary(context_name.trim().to_string()),
                }
            } else {
                Line {
                    line_number,
                    data: LineData::Text(line.to_string()),
                }
            }
        })
        .collect()
}

pub fn build_context(
    project_root: &Path,
    current_path: &Path,
    visited: &mut HashSet<PathBuf>,
) -> Result<Context> {
    if visited.contains(current_path) {
        // Handle circular dependency by returning a placeholder context
        return Ok(Context {
            file_path: current_path.to_path_buf(),
            lines: Vec::new(),
        });
    }
    visited.insert(current_path.to_path_buf());

    let content = std::fs::read_to_string(current_path)
        .with_context(|| format!("Failed to read {:?}", current_path))?;
    let parsed_lines = parse_lines(&content);

    let mut ast_nodes = Vec::new();
    for line in parsed_lines {
        match line.data {
            LineData::Include(context_name) => {
                let include_path = resolve_context_path(project_root, &context_name)?;
                let child_context = build_context(project_root, &include_path, visited)?;
                ast_nodes.push(AstNode::Context(child_context));
            },
            LineData::Inline(snippet_name) => {
                let snippet_path = resolve_snippet_path(project_root, &snippet_name)?;
                let child_snippet = build_snippet(project_root, &snippet_path)?;
                ast_nodes.push(AstNode::Snippet(child_snippet));
            },
            LineData::Summary(context_name) => {
                let summary_path = resolve_context_path(project_root, &context_name)?;
                let child_context = build_context(project_root, &summary_path, visited)?;
                ast_nodes.push(AstNode::Context(child_context));
            },
            _ => ast_nodes.push(AstNode::Line(line)),
        }
    }

    Ok(Context {
        file_path: current_path.to_path_buf(),
        lines: ast_nodes,
    })
}

pub fn build_snippet(
    project_root: &Path,
    current_path: &Path,
) -> Result<Snippet> {
    let content = std::fs::read_to_string(current_path)
        .with_context(|| format!("Failed to read {:?}", current_path))?;
    let parsed_lines = parse_lines(&content);

    let mut ast_nodes = Vec::new();
    for line in parsed_lines {
        // Snippets do not process includes/inlines/summaries recursively
        // They just contain lines of text or directives that will be processed by the parent context
        ast_nodes.push(AstNode::Line(line));
    }

    Ok(Snippet {
        file_path: current_path.to_path_buf(),
        lines: ast_nodes,
    })
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
