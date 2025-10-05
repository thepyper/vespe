use std::path::PathBuf;

/// Trait for resolving context and snippet names to file paths.
pub trait Resolver {
    /// Resolves a context name to its corresponding file path.
    fn resolve_context(&self, ctx_name: &str) -> PathBuf;
    /// Resolves a snippet name to its corresponding file path.
    fn resolve_snippet(&self, snippet_name: &str) -> PathBuf;
}
