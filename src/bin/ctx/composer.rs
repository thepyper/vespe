use anyhow::Result;
use std::collections::HashSet;
use std::path::PathBuf;

use crate::agent_call::AgentCall;
use crate::ast::{Context, Line, LineData, Snippet, Visitor, walk};
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
            composed_lines: Vec::new(),
        }
    }



    pub fn get_composed_lines(self) -> Vec<Line> {
        self.composed_lines
    }
}

impl<'a> Visitor for ComposerVisitor<'a> {
    fn pre_visit_context(&mut self, context: &crate::ast::Context) {
        // No specific action needed before visiting context children
    }
    fn post_visit_context(&mut self, context: &crate::ast::Context) {
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
                self.composed_lines.push(Line {
                    line_number: line.line_number,
                    data: line.data.clone(),
                    source_file: line.source_file.clone(),
                    source_line_number: line.source_line_number,
                });
            },
            LineData::Include(included_context) => {
                // The ast::walk function will handle the traversal of included_context.lines
                // No explicit action needed here, as the visitor methods will be called.
            },
            LineData::Inline(included_snippet) => {
                // The ast::walk function will handle the traversal of included_snippet.lines
                // No explicit action needed here, as the visitor methods will be called.
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
                    source_file: line.source_file.clone(),
                    source_line_number: line.source_line_number,
                });
            },
            _ => {},
        }
    }
    fn post_visit_line(&mut self, line: &Line) {
        // No specific action needed after visiting a line
    }
}