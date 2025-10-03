use super::types::{Context, Snippet, Line};
use async_trait::async_trait;

/// A trait for implementing the Visitor pattern on the AST.
///
/// Implementors of this trait can traverse the AST and perform operations
/// on its nodes. All methods provide default empty implementations, so
/// you only need to override the methods relevant to your use case.
pub trait Visitor {
    /// Called before visiting the lines of a `Context`.
    fn pre_visit_context(&mut self, _context: &mut Context) {}

    /// Called after visiting the lines of a `Context`.
    fn post_visit_context(&mut self, _context: &mut Context) {}

    /// Called before visiting the lines of a `Snippet`.
    fn pre_visit_snippet(&mut self, _snippet: &mut Snippet) {}

    /// Called after visiting the lines of a `Snippet`.
    fn post_visit_snippet(&mut self, _snippet: &mut Snippet) {}

    /// Called for each `Line` in the AST.
    fn visit_line(&mut self, _line: &mut Line) {}
}

/// Walks a `Context` node and its children, calling the appropriate
/// methods on the provided `Visitor`.
pub fn walk_context(context: &mut Context, visitor: &mut dyn Visitor) {
    visitor.pre_visit_context(context);
    for line in &mut context.lines {
        walk_line(line, visitor);
    }
    visitor.post_visit_context(context);
}

/// Walks a `Snippet` node and its children, calling the appropriate
/// methods on the provided `Visitor`.
pub fn walk_snippet(snippet: &mut Snippet, visitor: &mut dyn Visitor) {
    visitor.pre_visit_snippet(snippet);
    for line in &mut snippet.lines {
        walk_line(line, visitor);
    }
    visitor.post_visit_snippet(snippet);
}

/// Walks a `Line` node, calling the appropriate method on the provided `Visitor`.
pub fn walk_line(line: &mut Line, visitor: &mut dyn Visitor) {
    visitor.visit_line(line);
}

#[async_trait]
pub trait AsyncVisitor {
    /// Called before visiting the lines of a `Context` asynchronously.
    async fn pre_visit_context_async(&mut self, _context: &mut Context) -> Result<(), Box<dyn std::error::Error>> { Ok(()) }

    /// Called after visiting the lines of a `Context` asynchronously.
    async fn post_visit_context_async(&mut self, _context: &mut Context) -> Result<(), Box<dyn std::error::Error>> { Ok(()) }

    /// Called before visiting the lines of a `Snippet` asynchronously.
    async fn pre_visit_snippet_async(&mut self, _snippet: &mut Snippet) -> Result<(), Box<dyn std::error::Error>> { Ok(()) }

    /// Called after visiting the lines of a `Snippet` asynchronously.
    async fn post_visit_snippet_async(&mut self, _snippet: &mut Snippet) -> Result<(), Box<dyn std::error::Error>> { Ok(()) }

    /// Called for each `Line` in the AST asynchronously.
    async fn visit_line_async(&mut self, _line: &mut Line) -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
}

/// Walks a `Context` node and its children asynchronously, calling the appropriate
/// methods on the provided `AsyncVisitor`.
pub async fn walk_context_async(context: &mut Context, visitor: &mut dyn AsyncVisitor) -> Result<(), Box<dyn std::error::Error>> {
    visitor.pre_visit_context_async(context).await?;
    for line in &mut context.lines {
        walk_line_async(line, visitor).await?;
    }
    visitor.post_visit_context_async(context).await?;
    Ok(())
}

/// Walks a `Snippet` node and its children asynchronously, calling the appropriate
/// methods on the provided `AsyncVisitor`.
pub async fn walk_snippet_async(snippet: &mut Snippet, visitor: &mut dyn AsyncVisitor) -> Result<(), Box<dyn std::error::Error>> {
    visitor.pre_visit_snippet_async(snippet).await?;
    for line in &mut snippet.lines {
        walk_line_async(line, visitor).await?;
    }
    visitor.post_visit_snippet_async(snippet).await?;
    Ok(())
}

/// Walks a `Line` node asynchronously, calling the appropriate method on the provided `AsyncVisitor`.
pub async fn walk_line_async(line: &mut Line, visitor: &mut dyn AsyncVisitor) -> Result<(), Box<dyn std::error::Error>> {
    visitor.visit_line_async(line).await?;
    Ok(())
}
