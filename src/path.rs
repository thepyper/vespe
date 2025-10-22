
use anyhow::Result;
use std::path::PathBuf;
use uuid::Uuid;

trait PathResolver {
    /// Resolve a context name to a path
    pub fn resolve_context(&self, context_name: &str) -> Result<PathBuf>;
    /// Resolve a meta kind / uuid to a path, create directory if doesn't exist
    pub fn resolve_metadata(&self, meta_kind: &str, meta_uuid: &Uuid) -> Result<PathBuf>;
}

