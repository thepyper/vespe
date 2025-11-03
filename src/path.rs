use crate::constants::{CONTEXTS_DIR_NAME, CTX_DIR_NAME, METADATA_DIR_NAME};
use anyhow::{Context, Result};
use std::path::PathBuf;
use uuid::Uuid;

pub trait PathResolver {
    /// Resolve a context name to a path
    fn resolve_context(&self, context_name: &str) -> Result<PathBuf>;
    /// Resolve a meta kind / uuid to a path, create directory if doesn't exist
    fn resolve_metadata(&self, meta_kind: &str, meta_uuid: &Uuid) -> Result<PathBuf>;
}

pub struct ProjectPathResolver {
    root_path: PathBuf,
}

impl ProjectPathResolver {
    pub fn new(root_path: PathBuf) -> Self {
        ProjectPathResolver { root_path }
    }
    pub fn project_home(&self) -> PathBuf {
        self.root_path.join(CTX_DIR_NAME)
    }
    pub fn metadata_home(&self) -> PathBuf {
        self.project_home().join(METADATA_DIR_NAME)
    }
    pub fn contexts_root(&self) -> PathBuf {
        self.project_home().join(CONTEXTS_DIR_NAME)
    }
}

impl PathResolver for ProjectPathResolver {
    /// Resolve a context name to a path, create directory if doesn't exist
    fn resolve_context(&self, context_name: &str) -> Result<PathBuf> {
        let context_path = self.contexts_root().join(format!("{}.md", context_name));
        let parent_dir = context_path
            .parent()
            .context("Failed to get parent directory")?;
        std::fs::create_dir_all(parent_dir)
            .context("Failed to create parent directories for context file")?;
        Ok(context_path)
    }
    /// Resolve a meta kind / uuid to a path, create directory if doesn't exist
    fn resolve_metadata(&self, meta_kind: &str, meta_uuid: &Uuid) -> Result<PathBuf> {
        let metadata_dir =
            self.metadata_home()
                .join(format!("{}-{}", meta_kind, meta_uuid.to_string()));
        std::fs::create_dir_all(&metadata_dir).with_context(|| {
            format!(
                "Failed to create metadata directory for {}-{}: {}",
                meta_kind,
                meta_uuid,
                metadata_dir.display()
            )
        })?;
        Ok(metadata_dir)
    }
}
