use anyhow::{Context, Result};
use std::collections::HashSet;
use std::path::PathBuf;

use crate::agent_call::AgentCall;
use crate::ast::{AstNode, Context, Line, LineData, Snippet, Visitor, walk};
use crate::project::Project; // Import Project to access its methods

pub struct ComposerVisitor<'a> {
    project: &'a Project,
    agent: &'a dyn AgentCall,
    visited_summaries: HashSet<PathBuf>,
    composed_lines: Vec<Line>,
}

impl<'a> ComposerVisitor<'a> {
    pub fn new(project: &'a Project, agent: &'a dyn AgentCall) -> Self {
        Self {
            project,
            agent,
            visited_summaries: HashSet::new(),
        }
    }

    pub fn compose_from_ast(&mut self, ast_node: &AstNode) -> Result<Vec<Line>> {
        let mut visitor = ComposerVisitor::new(self.project, self.agent);
        walk(ast_node, &mut visitor);
        Ok(visitor.get_composed_lines())
    }

    pub fn get_composed_lines(self) -> Vec<Line> {
        self.composed_lines
    }
}

impl<'a> Visitor for ComposerVisitor<'a> {
    fn pre_visit_context(&mut self, context: &Context) {
        // No specific action needed before visiting context children
    }
    fn post_visit_context(&mut self, context: &Context) {
        // No specific action needed after visiting context children
    }
    fn pre_visit_snippet(&mut self, snippet: &Snippet) {
        // No specific action needed before visiting snippet children
    }
    fn post_visit_snippet(&mut self, snippet: &Snippet) {
        // No specific action needed after visiting snippet children
    }
    fn pre_visit_line(&mut self, line: &Line) {
        match &line.data {
            LineData::Text(_) | LineData::Answer => {
                self.composed_lines.push(line.clone());
            },
            LineData::Include(included_context) => {
                // Recursively walk the included context
                for node in &included_context.lines {
                    walk(node, self);
                }
            },
            LineData::Inline(included_snippet) => {
                // Add lines from the inline snippet directly
                for node in &included_snippet.lines {
                    if let AstNode::Line(snippet_line) = node {
                        self.composed_lines.push(snippet_line.clone());
                    }
                }
            },
            LineData::Summary(summary_context) => {
                let summary_path = summary_context.file_path.clone();
                if self.visited_summaries.contains(&summary_path) {
                    // Avoid infinite recursion for circular summaries
                    return;
                }
                self.visited_summaries.insert(summary_path.clone());

                let summarized_text = self.project._handle_summary_tag(summary_context, self.agent).unwrap();
                self.composed_lines.push(Line {
                    data: LineData::Text(summarized_text),
                    line_number: line.line_number,
                });
            },
            _ => {},
        }
    }
    fn post_visit_line(&mut self, line: &Line) {
        // No specific action needed after visiting a line
    }
}