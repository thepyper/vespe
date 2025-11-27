use super::Result;
use crate::constants::{CONTEXTS_DIR_NAME, CTX_DIR_NAME, METADATA_DIR_NAME};
use std::path::PathBuf;
use thiserror::Error as ThisError;
use uuid::Uuid;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("File not found: '{file_name}'. Searched paths: {searched_paths:?}")]
    FileNotFound {
        file_name: String,
        searched_paths: Vec<PathBuf>,
    },
    #[error("Parent directory not found for path: '{file_path}'")]
    ParentDirectoryNotFound { file_path: PathBuf },
    #[error("Failed to create directory '{path}': {source}")]
    FailedToCreateDirectory {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

use std::fmt::Debug;

pub trait PathResolver: Send + Sync + Debug {
    /// Resolve a file name to a path
    fn resolve_input_file(&self, file_name: &str) -> Result<PathBuf>;
    /// Resolve a file name to a path
    fn resolve_output_file(&self, file_name: &str) -> Result<PathBuf>;
    /// Resolve a meta kind / uuid to a path, create directory if doesn't exist
    fn resolve_metadata(&self, meta_kind: &str, meta_uuid: &Uuid) -> Result<PathBuf>;
}

#[derive(Debug)]
pub struct ProjectPathResolver {
    root_path: PathBuf,
    aux_paths: Vec<PathBuf>,
    output_path: Option<PathBuf>,
}

impl ProjectPathResolver {
    pub fn new(root_path: PathBuf, aux_paths: Vec<PathBuf>, output_path: Option<PathBuf>) -> Self {
        ProjectPathResolver {
            root_path,
            aux_paths,
            output_path,
        }
    }
    pub fn with_additional_aux_paths(&self, additional_aux_paths: Vec<PathBuf>) -> Self {
        let mut new_aux_paths = self.aux_paths.clone();
        new_aux_paths.extend(additional_aux_paths);
        ProjectPathResolver {
            root_path: self.root_path.clone(),
            aux_paths: new_aux_paths,
            output_path: self.output_path.clone(),
        }
    }
    pub fn with_alternative_output_path(&self, alternative_output_path: PathBuf) -> Self {
        ProjectPathResolver {
            root_path: self.root_path.clone(),
            aux_paths: self.aux_paths.clone(),
            output_path: Some(alternative_output_path),
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
        let mut searched_paths = vec![self.contexts_root()];
        if let Ok(file_path) = std::path::absolute(self.contexts_root().join(file_name)) {
            if file_path.exists() {
                return Ok(file_path);
            }
        }

        for aux_path in &self.aux_paths {
            if let Ok(aux_file_path) = std::path::absolute(aux_path.join(file_name)) {
                if aux_file_path.exists() {
                    return Ok(aux_file_path);
                }
            }
            searched_paths.push(aux_path.clone());
        }

        Err(super::Error::Path(Error::FileNotFound {
            file_name: file_name.to_string(),
            searched_paths,
        }))
    }
    /// Resolve a file name to a path, create directory if doesn't exist
    fn resolve_output_file(&self, file_name: &str) -> Result<PathBuf> {
        let base_path = std::path::absolute(if let Some(ref path) = self.output_path {
            path.clone()
        } else {
            self.contexts_root()
        })
        .map_err(Error::Io)?;
        let file_path = base_path.join(format!("{}", file_name));
        let parent_dir = file_path
            .parent()
            .ok_or_else(|| Error::ParentDirectoryNotFound {
                file_path: file_path.clone(),
            })?;
        std::fs::create_dir_all(parent_dir).map_err(|e| Error::FailedToCreateDirectory {
            path: parent_dir.to_path_buf(),
            source: e,
        })?;
        Ok(file_path)
    }
    /// Resolve a meta kind / uuid to a path, create directory if doesn't exist
    fn resolve_metadata(&self, meta_kind: &str, meta_uuid: &Uuid) -> Result<PathBuf> {
        let metadata_dir =
            self.metadata_home()
                .join(format!("{}-{}", meta_kind, meta_uuid.to_string()));
        std::fs::create_dir_all(&metadata_dir).map_err(|e| Error::FailedToCreateDirectory {
            path: metadata_dir.clone(),
            source: e,
        })?;
        Ok(metadata_dir)
    }
}
