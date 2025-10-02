use anyhow::Result;
use std::collections::HashSet;
use std::path::PathBuf;

use crate::ast::ContextAstNode;

pub struct ContextTreeBuilder;

impl ContextTreeBuilder {
    pub fn build_tree_from_ast(ast_node: &ContextAstNode, visited: &mut HashSet<PathBuf>) -> Result<ContextAstNode> {
        if visited.contains(&ast_node.path) {
            return Ok(ast_node.clone());
        }
        visited.insert(ast_node.path.clone());

        // The children are already part of the ast_node, so we just return the node itself.
        // The visited set is still useful to prevent infinite recursion if we were to rebuild the AST here,
        // but since we're just returning the existing node, it's less critical for this specific function.
        // However, it's good practice to keep it if this function were to be expanded later.
        Ok(ast_node.clone())
    }
}