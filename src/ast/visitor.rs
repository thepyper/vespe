use super::types::{Context, Snippet, LineKind};

/// A trait for implementing the Visitor pattern on the AST.
///
/// Implementors of this trait can traverse the AST and perform operations
/// on its nodes. All methods provide default empty implementations, so
/// you only need to override the methods relevant to your use case.
pub trait Visitor {
    /// Called before visiting a `Context`.
    fn pre_visit_context(&mut self, _context: &mut Context) {}

    /// Called after visiting a `Context`.
    fn post_visit_context(&mut self, _context: &mut Context) {}

    /// Called before visiting a `Snippet`.
    fn pre_visit_snippet(&mut self, _snippet: &mut Snippet) {}

    /// Called after visiting a `Snippet`.
    fn post_visit_snippet(&mut self, _snippet: &mut Snippet) {}

    /// Called to process the lines within a `Context`.
    fn visit_context_lines(&mut self, _context: &mut Context) {}

    /// Called to process the lines within a `Snippet`.
    fn visit_snippet_lines(&mut self, _snippet: &mut Snippet) {}
}

/// Walks a `Context` node and its children, calling the appropriate
/// methods on the provided `Visitor`.
pub fn walk_context(context: &mut Context, visitor: &mut dyn Visitor) {
    visitor.pre_visit_context(context);
    visitor.visit_context_lines(context);
    for line in &mut context.lines {
        match &mut line.kind {
            LineKind::Include { context: included_context, .. } => {
                walk_context(included_context, visitor);
            }
            LineKind::Inline { snippet, .. } => {
                walk_snippet(snippet, visitor);
            }
            LineKind::Summary { context: summary_context, .. }   => {
                walk_context(summary_context, visitor);
            }
            _ => {}
        }
    }
    visitor.post_visit_context(context);
}

/// Walks a `Snippet` node and its children, calling the appropriate
/// methods on the provided `Visitor`.
pub fn walk_snippet(snippet: &mut Snippet, visitor: &mut dyn Visitor) {
    visitor.pre_visit_snippet(snippet);
    visitor.visit_snippet_lines(snippet);
    for line in &mut snippet.lines {
        match &mut line.kind {
            LineKind::Include { context, .. } => {
                walk_context(context, visitor);
            }
            LineKind::Inline { snippet: included_snippet, .. } => {
                walk_snippet(included_snippet, visitor);
            }
            LineKind::Summary { context, .. }   => {
                walk_context(context, visitor);
            }
            _ => {}
        }
    }
    visitor.post_visit_snippet(snippet);
}
