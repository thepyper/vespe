use super::types::{Context, Snippet, Line};
use async_trait::async_trait;
use anyhow::Result;

/// A trait for implementing the Visitor pattern on the AST.
///
/// Implementors of this trait can traverse the AST and perform operations
/// on its nodes. All methods provide default empty implementations, so
/// you only need to override the methods relevant to your use case.
pub trait Visitor {
    /// Called before visiting the lines of a `Context`.
    fn pre_visit_context(&mut self, _context: &mut Context) -> Result<()> { Ok(()) }

    /// Called after visiting the lines of a `Context`.
    fn post_visit_context(&mut self, _context: &mut Context) -> Result<()> { Ok(()) }

    /// Called before visiting the lines of a `Snippet`.
    fn pre_visit_snippet(&mut self, _snippet: &mut Snippet) -> Result<()> { Ok(()) }

    /// Called after visiting the lines of a `Snippet`.
    fn post_visit_snippet(&mut self, _snippet: &mut Snippet) -> Result<()> { Ok(()) }

    /// Called for each `Line` in the AST.
    fn visit_line(&mut self, _line: &mut Line) -> Result<()> { Ok(()) }
}

/// Walks a `Context` node and its children, calling the appropriate
/// methods on the provided `Visitor`.
pub fn walk_context_sync(context: &mut Context, visitor: &mut dyn Visitor) -> Result<()> {
    visitor.pre_visit_context(context)?;
    for line in &mut context.lines {
        walk_line(line, visitor)?;
    }
    visitor.post_visit_context(context)?;
    Ok(())
}

/// Walks a `Snippet` node and its children, calling the appropriate
/// methods on the provided `Visitor`.
pub fn walk_snippet(snippet: &mut Snippet, visitor: &mut dyn Visitor) -> Result<()> {
    visitor.pre_visit_snippet(snippet)?;
    for line in &mut snippet.lines {
        walk_line(line, visitor)?;
    }
    visitor.post_visit_snippet(snippet)?;
    Ok(())
}

/// Walks a `Line` node, calling the appropriate method on the provided `Visitor`.
pub fn walk_line(line: &mut Line, visitor: &mut dyn Visitor) -> Result<()> {
    visitor.visit_line(line)?;
    Ok(())
}
