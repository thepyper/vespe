use git2::{Repository, Signature, Index, Oid, Tree, StatusOptions, Status};
use std::path::{Path, PathBuf, StripPrefixError};
use tracing::debug;
use std::collections::HashSet;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Git2 error: {0}")]
    Git2Error(#[from] git2::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Path prefix error: {0}")]
    StripPrefixError(#[from] StripPrefixError),
    #[error("Failed to open repository")]
    RepositoryOpenError,
    #[error("Repository has no workdir")]
    NoWorkdir,
    #[error("Failed to get HEAD commit")]
    HeadCommitError,
    #[error("Failed to get tree from HEAD commit")]
    HeadTreeError,
    #[error("Failed to get repository index")]
    IndexError,
    #[error("Failed to get repository status")]
    StatusError,
    #[error("Failed to read HEAD tree into index")]
    ReadHeadTreeError,
    #[error("Failed to canonicalize path: {0}")]
    CanonicalizePathError(PathBuf),
    #[error("Failed to canonicalize workdir: {0}")]
    CanonicalizeWorkdirError(PathBuf),
    #[error("File {0} is outside the repository workdir at {1}")]
    FileOutsideWorkdir(PathBuf, PathBuf),
    #[error("Failed to add file '{0}' to index")]
    AddFileToIndexError(PathBuf),
    #[error("Failed to write index")]
    IndexWriteError,
    #[error("Failed to write tree from index")]
    WriteTreeError,
    #[error("Failed to find tree")]
    FindTreeError,
    #[error("Failed to create signature")]
    SignatureError,
    #[error("Failed to create commit")]
    CommitCreationError,
    #[error("Failed to find the new commit")]
    FindNewCommitError,
    #[error("Failed to get tree from the new commit")]
    NewCommitTreeError,
    #[error("Failed to restore index to new HEAD state")]
    RestoreIndexError,
    #[error("Failed to re-add file '{0}' to index")]
    ReAddFileToIndexError(PathBuf),
}

pub type Result<T> = std::result::Result<T, Error>;
 
pub fn git_commit_files(files_to_commit: &[PathBuf], message: &str) -> Result<()> {

    debug!("Running git_commit_files wth message {} on files {:?}", message, files_to_commit);
    
    let repo = Repository::open(".")
        .map_err(|_| Error::RepositoryOpenError)?;
    let workdir = repo.workdir().ok_or(Error::NoWorkdir)?;

    let files_to_commit_set: HashSet<PathBuf> = files_to_commit.iter().cloned().collect();

    let head_commit = repo.head()
        .and_then(|head| head.peel_to_commit())
        .map_err(|_| Error::HeadCommitError)?;

    let head_tree = head_commit.tree()
        .map_err(|_| Error::HeadTreeError)?;

    let mut index = repo.index()
        .map_err(|_| Error::IndexError)?;

    let mut files_to_re_stage: Vec<PathBuf> = Vec::new();
    let mut status_options = StatusOptions::new();
    status_options.include_ignored(false)
                  .include_untracked(false)
                  .recurse_untracked_dirs(false)
                  .exclude_submodules(true);

    let statuses = repo.statuses(Some(&mut status_options))
        .map_err(|_| Error::StatusError)?;

    for entry in statuses.iter() {
        if let Some(path_str) = entry.path() {
            let path = PathBuf::from(path_str);
            let status = entry.status();
            if (status.is_index_new() || status.is_index_modified() || status.is_index_deleted() || status.is_index_renamed() || status.is_index_typechange())
                && !files_to_commit_set.contains(&path) {
                files_to_re_stage.push(path);
            }
        }
    }

    index.read_tree(&head_tree)
        .map_err(|_| Error::ReadHeadTreeError)?;

    for path in files_to_commit {
        let canonical_path = path.canonicalize().map_err(|_| Error::CanonicalizePathError(path.clone()))?;
        let canonical_workdir = workdir.canonicalize().map_err(|_| Error::CanonicalizeWorkdirError(workdir.to_path_buf()))?;

        let relative_path = canonical_path.strip_prefix(&canonical_workdir).map_err(|_| {
            Error::FileOutsideWorkdir(
                path.clone(),
                workdir.to_path_buf(),
            )
        })?;
        index.add_path(relative_path)
            .map_err(|_| Error::AddFileToIndexError(path.clone()))?;
    }

    index.write()
        .map_err(|_| Error::IndexWriteError)?;

    let tree_oid = index.write_tree()
        .map_err(|_| Error::WriteTreeError)?;
    let tree = repo.find_tree(tree_oid)
        .map_err(|_| Error::FindTreeError)?;

    let signature = Signature::now("vespe", "vespe@example.com")
        .map_err(|_| Error::SignatureError)?;

    let new_commit_oid = repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        message,
        &tree,
        &[&head_commit],
    )
    .map_err(|_| Error::CommitCreationError)?;

    let new_head_commit = repo.find_commit(new_commit_oid)
        .map_err(|_| Error::FindNewCommitError)?;
    let new_head_tree = new_head_commit.tree()
        .map_err(|_| Error::NewCommitTreeError)?;

    index.read_tree(&new_head_tree)
        .map_err(|_| Error::RestoreIndexError)?;

    for path in files_to_re_stage {
        let canonical_path = path.canonicalize().map_err(|_| Error::CanonicalizePathError(path.clone()))?;
        let canonical_workdir = workdir.canonicalize().map_err(|_| Error::CanonicalizeWorkdirError(workdir.to_path_buf()))?;

        let relative_path = canonical_path.strip_prefix(&canonical_workdir).map_err(|_| {
            Error::FileOutsideWorkdir(
                path.clone(),
                workdir.to_path_buf(),
            )
        })?;
        index.add_path(relative_path)
            .map_err(|_| Error::ReAddFileToIndexError(path.clone()))?;
    }

    index.write()
        .map_err(|_| Error::IndexWriteError)?;

    debug!("Commit created with id {}", new_commit_oid);
    Ok(())
}

pub struct Commit {
    pub files: HashSet<PathBuf>,
}

impl Commit {
    pub fn new() -> Self {
        Commit { files: HashSet::new() }
    }
    pub fn commit(&self, message: &str) -> Result<()> {
        if self.files.is_empty() {
            return Ok(())
        }
        let files = self.files.iter().cloned().collect::<Vec<PathBuf>>();
        git_commit_files(&files, message)
    }
}

