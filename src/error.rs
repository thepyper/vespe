use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Context name is required unless --today is specified.")]
    ContextNameRequired,
    #[error("Failed to read from stdin: {0}")]
    StdinReadError(#[from] std::io::Error),
    #[error("File '{file_name}' not found in any of the following paths: {searched_paths:?}")]
    FileNotFound {
        file_name: String,
        searched_paths: Vec<PathBuf>,
    },
    #[error("Failed to get parent directory for path: '{file_path}'")]
    ParentDirectoryNotFound { file_path: PathBuf },
    #[error("Failed to create directory '{path}': {source}")]
    FailedToCreateDirectory {
        path: PathBuf,
        source: std::io::Error,
    },
}
