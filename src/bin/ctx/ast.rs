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

#[derive(Debug, Clone)]
pub struct ContextInfo {
    pub name: String,
    pub path: PathBuf,
}

pub fn get_context_info(path: &Path) -> Result<ContextInfo> {
    let name = to_name(path.file_name().unwrap().to_str().unwrap());
    Ok(ContextInfo {
        name,
        path: path.to_path_buf(),
    })
}