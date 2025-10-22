
trait PathResolver {
    /// Resolve a context name to a path
    fn resolve_context(context_name: &str) -> Result<PathBuf>;
    /// Resolve a meta kind / uuid to a path, create directory if doesn't exist
    fn resolve_meta(meta_kind: &str, meta_uuid: &Uuid) -> Result<PathBuf>;
}

