use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Context name is required unless --today is specified.")]
    ContextNameRequired,
    #[error("Project already initialized in this directory.")]
    ProjectAlreadyInitialized,
    #[error("No .ctx project found in the current directory or any parent directory.")]
    ProjectNotFound,
    #[error("Context file already exists: '{path}'")]
    ContextFileAlreadyExists { path: PathBuf },
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Failed to acquire mutex lock")]
    MutexLockError,
    #[error("Failed to canonicalize path '{path}': {source}")]
    CanonicalizePath {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Editor interface error: {message} ({source})")]
    EditorInterface {
        message: String,
        #[source]
        source: anyhow::Error,
    },
    #[error("Failed to create directory '{path}': {source}")]
    FailedToCreateDirectory {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("Failed to write file '{path}': {source}")]
    FileWrite {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("Parent directory not found for path: '{file_path}'")]
    ParentDirectoryNotFound { file_path: PathBuf },
    #[error("Failed to read file '{path}': {source}")]
    FileRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error(transparent)]
    Utils(#[from] crate::utils::Error),
}
