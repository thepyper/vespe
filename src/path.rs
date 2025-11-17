use crate::constants::{CONTEXTS_DIR_NAME, CTX_DIR_NAME, METADATA_DIR_NAME};
use anyhow::{Context, Result};
use std::path::PathBuf;
use uuid::Uuid;

pub trait PathResolver {
    /// Resolve a file name to a path
    fn resolve_input_file(&self, file_name: &str) -> Result<PathBuf>;
    /// Resolve a file name to a path
    fn resolve_output_file(&self, file_name: &str) -> Result<PathBuf>;
    /// Resolve a meta kind / uuid to a path, create directory if doesn't exist
    fn resolve_metadata(&self, meta_kind: &str, meta_uuid: &Uuid) -> Result<PathBuf>;
}

pub struct ProjectPathResolver {
    root_path: PathBuf,
    aux_paths: Vec<PathBuf>,
}

impl ProjectPathResolver {
    pub fn new(root_path: PathBuf, aux_paths: Vec<PathBuf>) -> Self {
        ProjectPathResolver { root_path, aux_paths }
    }

    pub fn with_additional_aux_paths(&self, additional_aux_paths: Vec<PathBuf>) -> Self {
        let mut new_aux_paths = self.aux_paths.clone();
        new_aux_paths.extend(additional_aux_paths);
        ProjectPathResolver {
            root_path: self.root_path.clone(),
            aux_paths: new_aux_paths,
        }
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
    /// Resolve a file name to a path, create directory if doesn't exist
    fn resolve_input_file(&self, file_name: &str) -> Result<PathBuf> {
        let file_path = self.contexts_root().join(file_name);
        if file_path.exists() {
            return Ok(file_path);
        }

        for aux_path in &self.aux_paths {
            let aux_file_path = aux_path.join(file_name);
            if aux_file_path.exists() {
                return Ok(aux_file_path);
            }
        }

        Err(anyhow::anyhow!(
            "File '{}' not found in root_path or any auxiliary paths.",
            file_name
        ))
    }
    /// Resolve a file name to a path, create directory if doesn't exist
    fn resolve_output_file(&self, file_name: &str) -> Result<PathBuf> {
        let file_path = self.contexts_root().join(format!("{}", file_name));
        let parent_dir = file_path
            .parent()
            .context("Failed to get parent directory")?;
        std::fs::create_dir_all(parent_dir)
            .context("Failed to create parent directories for context file")?;
        Ok(file_path)
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
