use anyhow::{Context as AnyhowContext, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Clone)]
pub enum LineData {
    Include { context_name: String },
    Answer,
    Summary { context_name: String },
    Text(String),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Line {
    pub data: LineData,
    pub source_file: PathBuf,
    pub source_line_number: usize,
}



#[derive(Debug, Clone)]
pub struct ContextAstNode {
    pub path: PathBuf,
    pub lines: Vec<Line>,
    pub children: Vec<ContextAstNode>,
}

impl ContextAstNode {
    pub fn parse(content: &str, file_path: PathBuf) -> Vec<Line> {
        content
            .lines()
            .enumerate()
            .map(|(line_number, line)| {
                if let Some(context_name) = line.strip_prefix("@include ") {
                    Line {
                        data: LineData::Include {
                            context_name: context_name.trim().to_string(),
                        },
                        source_file: file_path.clone(),
                        source_line_number: line_number,
                    }
                } else if let Some(context_name) = line.strip_prefix("@summary ") {
                    Line {
                        data: LineData::Summary {
                            context_name: context_name.trim().to_string(),
                        },
                        source_file: file_path.clone(),
                        source_line_number: line_number,
                    }
                } else if line.trim() == "@answer" {
                    Line {
                        data: LineData::Answer,
                        source_file: file_path.clone(),
                        source_line_number: line_number,
                    }
                } else {
                    Line {
                        data: LineData::Text(line.to_string()),
                        source_file: file_path.clone(),
                        source_line_number: line_number,
                    }
                }
            })
            .collect()
    }

    pub fn build_context_ast(
        project_root: &Path,
        current_path: &Path,
        visited: &mut HashSet<PathBuf>,
    ) -> Result<ContextAstNode> {
        if visited.contains(current_path) {
            // Handle circular dependency by returning a node with no children
            return Ok(ContextAstNode {
                path: current_path.to_path_buf(),
                lines: Vec::new(), // No lines for circular dependency to avoid infinite loop
                children: Vec::new(),
            });
        }
        visited.insert(current_path.to_path_buf());

        let content = std::fs::read_to_string(current_path)
            .with_context(|| format!("Failed to read {:?}", current_path))?;
        let lines = Self::parse(&content, current_path.to_path_buf());

        let mut children = Vec::new();
        for line in &lines {
            if let LineData::Include { context_name } = &line.data {
                let include_path = resolve_context_path(project_root, context_name)?;
                children.push(Self::build_context_ast(project_root, &include_path, visited)?);
            }
        }

        Ok(ContextAstNode {
            path: current_path.to_path_buf(),
            lines,
            children,
        })
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
    if name.ends_with(".md") {
        name.to_string()
    } else {
        format!("{}.md", name)
    }
}

// Helper function to convert filename to name, moved from Project
pub fn to_name(name: &str) -> String {
    name.strip_suffix(".md").unwrap_or(name).to_string()
}
