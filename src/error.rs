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
    #[error("Git repository error: {message} ({source})")]
    GitRepositoryError {
        message: String,
        #[source]
        source: git2::Error,
    },
    #[error("Git repository has no workdir")]
    NoWorkdirError,
    #[error("Failed to get HEAD commit")]
    HeadCommitError(#[source] git2::Error),
    #[error("Failed to get tree from HEAD commit: {0}")]
    TreeFromCommitError(#[source] git2::Error),
    #[error("Failed to get repository index: {0}")]
    RepositoryIndexError(#[source] git2::Error),
    #[error("Failed to get repository status: {0}")]
    RepositoryStatusError(#[source] git2::Error),
    #[error("Failed to canonicalize path '{path}': {source}")]
    CanonicalizePathError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("File '{file_path}' is outside the repository workdir at '{workdir}'")]
    PathOutsideWorkdirError {
        file_path: PathBuf,
        workdir: PathBuf,
    },
    #[error("Failed to add file '{file_path}' to index: {0}")]
    AddFileToIndexError {
        file_path: PathBuf,
        #[source]
        source: git2::Error,
    },
    #[error("Failed to write index: {0}")]
    WriteIndexError(#[source] git2::Error),
    #[error("Failed to write tree from index: {0}")]
    WriteTreeError(#[source] git2::Error),
    #[error("Failed to find tree with OID '{oid}': {0}")]
    FindTreeError {
        oid: git2::Oid,
        #[source]
        source: git2::Error,
    },
    #[error("Failed to create git signature: {0}")]
    CreateSignatureError(#[source] git2::Error),
    #[error("Failed to create commit: {0}")]
    CreateCommitError(#[source] git2::Error),
    #[error("Failed to find commit with OID '{oid}': {0}")]
    FindCommitError {
        oid: git2::Oid,
        #[source]
        source: git2::Error,
    },
    #[error("Failed to restore index to new HEAD state: {0}")]
    RestoreIndexError(#[source] git2::Error),
}
