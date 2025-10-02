use anyhow::{Context, Result};
use std::collections::HashSet;
use std::path::PathBuf;

use crate::agent_call::AgentCall;
use crate::ast::{ContextAstNode, Line, LineData};
use crate::project::Project; // Import Project to access its methods

pub struct ContextComposer<'a> {
    project: &'a Project,
    agent: &'a dyn AgentCall,
    visited_summaries: HashSet<PathBuf>,
}

impl<'a> ContextComposer<'a> {
    pub fn new(project: &'a Project, agent: &'a dyn AgentCall) -> Self {
        Self {
            project,
            agent,
            visited_summaries: HashSet::new(),
        }
    }

    pub fn compose_from_ast(&mut self, ast_node: &ContextAstNode) -> Result<Vec<Line>> {
        let mut composed_lines = Vec::new();

        for line in &ast_node.lines {
            match &line.data {
                LineData::Include { context_name } => {
                    // Find the child AST node corresponding to the included context
                    if let Some(child_node) = ast_node.children.iter().find(|child| {
                        let child_name = crate::ast::to_name(&child.path.file_name().unwrap().to_string_lossy());
                        child_name == *context_name
                    }) {
                        composed_lines.extend(self.compose_from_ast(child_node)?);
                    } else {
                        // This case should ideally not happen if AST is built correctly
                        // but as a fallback, we can try to build a sub-AST for the include
                        // or just log an error. For now, let's assume AST is complete.
                        anyhow::bail!("Included context '{}' not found in AST children of {:?}", context_name, ast_node.path);
                    }
                }
                LineData::Summary { context_name } => {
                    let summary_path = crate::ast::resolve_context_path(&self.project.root_path, context_name)?;
                    if self.visited_summaries.contains(&summary_path) {
                        // Avoid infinite recursion for circular summaries
                        continue;
                    }
                    self.visited_summaries.insert(summary_path.clone());

                    let summarized_text = self.project._handle_summary_tag(context_name, self.agent)?;
                    composed_lines.push(Line {
                        data: LineData::Text(summarized_text),
                        source_file: line.source_file.clone(),
                        source_line_number: line.source_line_number,
                    });
                }
                LineData::Inline { snippet_name } => {
                    let snippet_path = crate::ast::resolve_snippet_path(&self.project.root_path, snippet_name)?;
                    let snippet_content = std::fs::read_to_string(&snippet_path)
                        .with_context(|| format!("Failed to read snippet file: {:?}", snippet_path))?;
                    
                    let parsed_snippet_lines = crate::ast::ContextAstNode::parse(&snippet_content, snippet_path.clone());

                    for snippet_line in parsed_snippet_lines {
                        composed_lines.push(Line {
                            data: snippet_line.data,
                            source_file: line.source_file.clone(), // Override with the source of the @inline directive
                            source_line_number: line.source_line_number, // Override with the source of the @inline directive
                        });
                    }
                }
                _ => {
                    composed_lines.push(line.clone());
                }
            }
        }
        Ok(composed_lines)
    }
}