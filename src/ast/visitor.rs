use super::types::{Context, Snippet, LineKind};
use std::error::Error;

#[async_trait::async_trait]
pub trait AsyncVisitor {
    async fn pre_visit_context(&mut self, _context: &mut Context) -> Result<(), Box<dyn Error>> { Ok(()) }
    async fn post_visit_context(&mut self, _context: &mut Context) -> Result<(), Box<dyn Error>> { Ok(()) }
    async fn pre_visit_snippet(&mut self, _snippet: &mut Snippet) -> Result<(), Box<dyn Error>> { Ok(()) }
    async fn post_visit_snippet(&mut self, _snippet: &mut Snippet) -> Result<(), Box<dyn Error>> { Ok(()) }
    async fn visit_context_lines(&mut self, _context: &mut Context) -> Result<(), Box<dyn Error>> { Ok(()) }
    async fn visit_snippet_lines(&mut self, _snippet: &mut Snippet) -> Result<(), Box<dyn Error>> { Ok(()) }
}

pub async fn async_walk_context(context: &mut Context, visitor: &mut (dyn AsyncVisitor + Send)) -> Result<(), Box<dyn Error>> {
    visitor.pre_visit_context(context).await?;
    visitor.visit_context_lines(context).await?;
    for line in &mut context.lines {
        match &mut line.kind {
            LineKind::Include { context: included_context, .. } => {
                async_walk_context(included_context, visitor).await?;
            }
            LineKind::Inline { snippet, .. } => {
                async_walk_snippet(snippet, visitor).await?;
            }
            LineKind::Summary { context: summary_context, .. }   => {
                async_walk_context(summary_context, visitor).await?;
            }
            _ => {}
        }
    }
    visitor.post_visit_context(context).await?;
    Ok(())
}

pub async fn async_walk_snippet(snippet: &mut Snippet, visitor: &mut (dyn AsyncVisitor + Send)) -> Result<(), Box<dyn Error>> {
    visitor.pre_visit_snippet(snippet).await?;
    visitor.visit_snippet_lines(snippet).await?;
    for line in &mut snippet.lines {
        match &mut line.kind {
            LineKind::Include { context, .. } => {
                async_walk_context(context, visitor).await?;
            }
            LineKind::Inline { snippet: included_snippet, .. } => {
                async_walk_snippet(included_snippet, visitor).await?;
            }
            LineKind::Summary { context, .. }   => {
                walk_context_async(context, visitor).await?;
            }
            _ => {}
        }
    }
    visitor.post_visit_snippet(snippet).await?;
    Ok(())
}

pub trait Visitor {
    /// Called before visiting a `Context`.
    fn pre_visit_context(&mut self, _context: &mut Context) -> Result<(), Box<dyn Error>> { Ok(()) }

    /// Called after visiting a `Context`.
    fn post_visit_context(&mut self, _context: &mut Context) -> Result<(), Box<dyn Error>> { Ok(()) }

    /// Called before visiting a `Snippet`.
    fn pre_visit_snippet(&mut self, _snippet: &mut Snippet) -> Result<(), Box<dyn Error>> { Ok(()) }

    /// Called after visiting a `Snippet`.
    fn post_visit_snippet(&mut self, _snippet: &mut Snippet) -> Result<(), Box<dyn Error>> { Ok(()) }

    /// Called to process the lines within a `Context`.
    fn visit_context_lines(&mut self, _context: &mut Context) -> Result<(), Box<dyn Error>> { Ok(()) }

    /// Called to process the lines within a `Snippet`.
    fn visit_snippet_lines(&mut self, _snippet: &mut Snippet) -> Result<(), Box<dyn Error>> { Ok(()) }
}

/// Walks a `Context` node and its children, calling the appropriate
/// methods on the provided `Visitor`.
pub fn walk_context(context: &mut Context, visitor: &mut dyn Visitor) -> Result<(), Box<dyn Error>> {
    visitor.pre_visit_context(context)?;
    visitor.visit_context_lines(context)?;
    for line in &mut context.lines {
        match &mut line.kind {
            LineKind::Include { context: included_context, .. } => {
                walk_context(included_context, visitor)?;
            }
            LineKind::Inline { snippet, .. } => {
                walk_snippet(snippet, visitor)?;
            }
            LineKind::Summary { context: summary_context, .. }   => {
                walk_context(summary_context, visitor)?;
            }
            _ => {}
        }
    }
    visitor.post_visit_context(context)?;
    Ok(())
}

/// Walks a `Snippet` node and its children, calling the appropriate
/// methods on the provided `Visitor`.
pub fn walk_snippet(snippet: &mut Snippet, visitor: &mut dyn Visitor) -> Result<(), Box<dyn Error>> {
    visitor.pre_visit_snippet(snippet)?;
    visitor.visit_snippet_lines(snippet)?;
    for line in &mut snippet.lines {
        match &mut line.kind {
            LineKind::Include { context, .. } => {
                walk_context(context, visitor)?;
            }
            LineKind::Inline { snippet: included_snippet, .. } => {
                walk_snippet(included_snippet, visitor)?;
            }
            LineKind::Summary { context, .. }   => {
                walk_context(context, visitor)?;
            }
            _ => {}
        }
    }
    visitor.post_visit_snippet(snippet)?;
    Ok(())
}
