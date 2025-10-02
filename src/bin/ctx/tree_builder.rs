use anyhow::Result;
use std::collections::HashSet;
use std::path::PathBuf;

use crate::ast::{ContextAstNode, ContextTreeItem};

pub struct ContextTreeBuilder;

impl ContextTreeBuilder {
    pub fn build_tree_from_ast(ast_node: &ContextAstNode, visited: &mut HashSet<PathBuf>) -> Result<ContextTreeItem> {
        if visited.contains(&ast_node.path) {
            return Ok(ContextTreeItem::Leaf { name: crate::ast::to_name(&ast_node.path.file_name().unwrap().to_string_lossy()) });
        }
        visited.insert(ast_node.path.clone());

        let mut children = Vec::new();
        for child_node in &ast_node.children {
            children.push(Self::build_tree_from_ast(child_node, visited)?);
        }

        let current_name = crate::ast::to_name(&ast_node.path.file_name().unwrap().to_string_lossy());

        if children.is_empty() {
            Ok(ContextTreeItem::Leaf { name: current_name })
        } else {
            Ok(ContextTreeItem::Node { name: current_name, children })
        }
    }
}