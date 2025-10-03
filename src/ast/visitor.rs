use super::types::{Context, Snippet, Line};

/// A trait for implementing the Visitor pattern on the AST.
///
/// Implementors of this trait can traverse the AST and perform operations
/// on its nodes. All methods provide default empty implementations, so
/// you only need to override the methods relevant to your use case.
pub trait Visitor {
    /// Called before visiting the lines of a `Context`.
    fn pre_visit_context(&mut self, _context: &Context) {}

    /// Called after visiting the lines of a `Context`.
    fn post_visit_context(&mut self, _context: &Context) {}

    /// Called before visiting the lines of a `Snippet`.
    fn pre_visit_snippet(&mut self, _snippet: &Snippet) {}

    /// Called after visiting the lines of a `Snippet`.
    fn post_visit_snippet(&mut self, _snippet: &Snippet) {}

    /// Called for each `Line` in the AST.
    fn visit_line(&mut self, _line: &Line) {}
}

/// Walks a `Context` node and its children, calling the appropriate
/// methods on the provided `Visitor`.
pub fn walk_context(context: &Context, visitor: &mut dyn Visitor) {
    visitor.pre_visit_context(context);
    for line in &context.lines {
        walk_line(line, visitor);
    }
    visitor.post_visit_context(context);
}

/// Walks a `Snippet` node and its children, calling the appropriate
/// methods on the provided `Visitor`.
pub fn walk_snippet(snippet: &Snippet, visitor: &mut dyn Visitor) {
    visitor.pre_visit_snippet(snippet);
    for line in &snippet.lines {
        walk_line(line, visitor);
    }
    visitor.post_visit_snippet(snippet);
}

/// Walks a `Line` node, calling the appropriate method on the provided `Visitor`.
pub fn walk_line(line: &Line, visitor: &mut dyn Visitor) {
    visitor.visit_line(line);
}
