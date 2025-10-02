use anyhow::Result;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::ast::{AstVisitor, ContextAstNode, Line, LineData};
use crate::project::Project;
use crate::agent_call::AgentCall;

pub struct ContextProcessorVisitor<'a> {
    project: &'a Project,
    _project_root: &'a Path,
    agent: &'a dyn AgentCall,
    visited_contexts: HashSet<PathBuf>,
    pub first_answer_found_globally: bool,
    pub processed_lines: Vec<String>,
    pub llm_prompt_content: String,
}

impl<'a> ContextProcessorVisitor<'a> {
    pub fn new(project: &'a Project, _project_root: &'a Path, agent: &'a dyn AgentCall) -> Self {
        Self {
            project,
            _project_root,
            agent,
            visited_contexts: HashSet::new(),
            first_answer_found_globally: false,
            processed_lines: Vec::new(),
            llm_prompt_content: String::new(),
        }
    }
}

impl<'a> AstVisitor for ContextProcessorVisitor<'a> {
    type Error = anyhow::Error;

    fn pre_visit_context_ast_node(&mut self, node: &mut ContextAstNode) -> Result<(), Self::Error> {
        if self.visited_contexts.contains(&node.path) {
            // Handle circular dependency
            return Ok(())
        }
        self.visited_contexts.insert(node.path.clone());
        Ok(())
    }

    fn post_visit_context_ast_node(&mut self, _node: &mut ContextAstNode) -> Result<(), Self::Error> {
        Ok(())
    }

    fn pre_visit_line(&mut self, _line: &mut Line) -> Result<(), Self::Error> {
        Ok(())
    }

    fn post_visit_line(&mut self, line: &mut Line) -> Result<(), Self::Error> {
        match &line.data {
            LineData::Text(text) => {
                self.processed_lines.push(text.clone());
                if !self.first_answer_found_globally {
                    self.llm_prompt_content.push_str(text);
                    self.llm_prompt_content.push('\n');
                }
            }
            LineData::Answer => {
                if !self.first_answer_found_globally {
                    // This is where the LLM call logic will go
                    // For now, we'll just mark it as found and add a placeholder
                    self.first_answer_found_globally = true;
                    self.processed_lines.push("LLM_RESPONSE_PLACEHOLDER".to_string());
                } else {
                    self.processed_lines.push("@answer".to_string());
                }
            }
            // Include and Inline are handled by the recursive traversal of ContextAstNode::accept
            // Their content will be added to processed_lines and llm_prompt_content as individual lines
            LineData::Inline { .. } => { /* Do nothing here, handled by AST traversal */ }
            LineData::Include { .. } => { /* Do nothing here, handled by AST traversal */ }
            LineData::Summary { context_name } => {
                let summary_content = self.project._handle_summary_tag(context_name, self.agent)?;
                self.processed_lines.push(summary_content.clone());
                if !self.first_answer_found_globally {
                    self.llm_prompt_content.push_str(&summary_content);
                    self.llm_prompt_content.push('\n');
                }
            }
        }
        Ok(())
    }

    fn pre_visit_line_data(&mut self, _line_data: &mut LineData) -> Result<(), Self::Error> {
        Ok(())
    }

    fn post_visit_line_data(&mut self, _line_data: &mut LineData) -> Result<(), Self::Error> {
        Ok(())
    }
}
