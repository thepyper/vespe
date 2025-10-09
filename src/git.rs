use git2::{Repository, Signature, Index, Oid, Tree, StatusOptions, Status}; // Added StatusOptions, Status
use std::path::{Path, PathBuf};
use tracing::debug;
use std::collections::HashSet; // Added HashSet
use thiserror::Error;
use crate::error::Result;
use crate::error::Error as GeneralError;

#[derive(Error, Debug)]
pub enum GitError {
    #[error("Failed to open repository: {0}")]
    RepositoryOpenFailed(#[from] git2::Error),
    #[error("Repository has no workdir")]
    NoWorkdir,
    #[error("Failed to get HEAD commit: {0}")]
    HeadCommitFailed(git2::Error),
    #[error("Failed to get tree from HEAD commit: {0}")]
    HeadTreeFailed(git2::Error),
    #[error("Failed to get repository index: {0}")]
    IndexFailed(git2::Error),
    #[error("Failed to get repository status: {0}")]
    StatusFailed(git2::Error),
    #[error("Failed to canonicalize path '{path}': {source}")]
    CanonicalizePathFailed { path: String, source: std::io::Error },
    #[error("File '{path}' is outside the repository workdir at '{workdir}'")]
    FileOutsideWorkdir { path: String, workdir: String },
    #[error("Failed to add file '{path}' to index: {source}")]
    AddPathFailed { path: String, source: git2::Error },
    #[error("Failed to read HEAD tree into index: {0}")]
    ReadHeadTreeFailed(git2::Error),
    #[error("Failed to write index: {0}")]
    IndexWriteFailed(git2::Error),
    #[error("Failed to write tree from index: {0}")]
    WriteTreeFailed(git2::Error),
    #[error("Failed to find tree: {0}")]
    FindTreeFailed(git2::Error),
    #[error("Failed to create signature: {0}")]
    SignatureFailed(git2::Error),
    #[error("Failed to create commit: {0}")]
    CommitFailed(git2::Error),
    #[error("Failed to find the new commit: {0}")]
    FindNewCommitFailed(git2::Error),
    #[error("Failed to get tree from the new commit: {0}")]
    NewCommitTreeFailed(git2::Error),
    #[error("Failed to restore index to new HEAD state: {0}")]
    RestoreIndexFailed(git2::Error),
    #[error("Failed to write restored index: {0}")]
    WriteRestoredIndexFailed(git2::Error),
}

pub fn git_commit_files(files_to_commit: &[PathBuf], message: &str) -> Result<()> {

    debug!("Running git_commit_files wth message {} on files {:?}", message, files_to_commit);
    
    let repo = Repository::open(".")
        .map_err(GitError::RepositoryOpenFailed)?;
    let workdir = repo.workdir().ok_or(GitError::NoWorkdir)?;

    // Convert files_to_commit to a HashSet for efficient lookup
    let files_to_commit_set: HashSet<PathBuf> = files_to_commit.iter().cloned().collect();

    // 1. Get the current HEAD commit. This will be the parent of the new commit
    // and the base for "cleaning" the staging area.
    let head_commit = repo.head()
        .and_then(|head| head.peel_to_commit())
        .map_err(GitError::HeadCommitFailed)?;

    // 2. Get the tree associated with the HEAD commit.
    // This represents the state of the repository at the last commit.
    let head_tree = head_commit.tree()
        .map_err(GitError::HeadTreeFailed)?;

    // 3. Load the repository index.
    let mut index = repo.index()
        .map_err(GitError::IndexFailed)?;

    // 4. Identify files that were staged *before* our operation,
    //    excluding those that will be committed.
    let mut files_to_re_stage: Vec<PathBuf> = Vec::new();
    let mut status_options = StatusOptions::new();
    status_options.include_ignored(false)
                  .include_untracked(false)
                  .recurse_untracked_dirs(false)
                  .exclude_submodules(true);

    let statuses = repo.statuses(Some(&mut status_options))
        .map_err(GitError::StatusFailed)?;

    for entry in statuses.iter() {
        if let Some(path_str) = entry.path() {
            let path = PathBuf::from(path_str);
            let status = entry.status();
            // Check if the file is staged (IndexNew, IndexModified, etc.)
            // and *not* part of the files we are about to commit.
            if (status.is_index_new() || status.is_index_modified() || status.is_index_deleted() || status.is_index_renamed() || status.is_index_typechange())
                && !files_to_commit_set.contains(&path) {
                files_to_re_stage.push(path);
            }
        }
    }

    // 5. Reset the index to the HEAD. This "unstages" everything.
    index.read_tree(&head_tree)
        .map_err(GitError::ReadHeadTreeFailed)?;

    // 6. Selectively add files_to_commit to the staging area (index).
    for path in files_to_commit {
        let canonical_path = path.canonicalize().map_err(|e| GitError::CanonicalizePathFailed { path: path.display().to_string(), source: e })?;
        let canonical_workdir = workdir.canonicalize().map_err(|e| GitError::CanonicalizePathFailed { path: workdir.display().to_string(), source: e })?;

        let relative_path = canonical_path.strip_prefix(&canonical_workdir).map_err(|_| {
            GitError::FileOutsideWorkdir {
                path: path.display().to_string(),
                workdir: workdir.display().to_string(),
            }
        })?;
        index.add_path(relative_path)
            .map_err(|e| GitError::AddPathFailed { path: path.display().to_string(), source: e })?;
    }

    // 7. Write the changes to the index to disk.
    index.write()
        .map_err(GitError::IndexWriteFailed)?;

    // 8. Create a new tree based on the current state of the index.
    let tree_oid = index.write_tree()
        .map_err(GitError::WriteTreeFailed)?;
    let tree = repo.find_tree(tree_oid)
        .map_err(GitError::FindTreeFailed)?;

    // 9. Create the author and committer signature.
    let signature = Signature::now("vespe", "vespe@example.com") // Using "vespe@example.com" as a placeholder
        .map_err(GitError::SignatureFailed)?;

    // 10. Create the new commit.
    let new_commit_oid = repo.commit(
        Some("HEAD"), // Update HEAD to point to the new commit
        &signature,
        &signature,
        message,
        &tree,
        &[&head_commit], // The new commit has the current HEAD as parent
    )
    .map_err(GitError::CommitFailed)?;

    // 11. Reset the index to the *new* HEAD. This is crucial to avoid the "ghost" effect
    //     for the newly committed files.
    let new_head_commit = repo.find_commit(new_commit_oid)
        .map_err(GitError::FindNewCommitFailed)?;
    let new_head_tree = new_head_commit.tree()
        .map_err(GitError::NewCommitTreeFailed)?;

    index.read_tree(&new_head_tree)
        .map_err(GitError::RestoreIndexFailed)?;

    // 12. Re-add the `files_to_re_stage` to the index.
    for path in files_to_re_stage {
        let canonical_path = path.canonicalize().map_err(|e| GitError::CanonicalizePathFailed { path: path.display().to_string(), source: e })?;
        let canonical_workdir = workdir.canonicalize().map_err(|e| GitError::CanonicalizePathFailed { path: workdir.display().to_string(), source: e })?;

        let relative_path = canonical_path.strip_prefix(&canonical_workdir).map_err(|_| {
            GitError::FileOutsideWorkdir {
                path: path.display().to_string(),
                workdir: workdir.display().to_string(),
            }
        })?;
        index.add_path(relative_path)
            .map_err(|e| GitError::AddPathFailed { path: path.display().to_string(), source: e })?;
    }

    // 13. Write the restored index to disk.
    index.write()
        .map_err(GitError::WriteRestoredIndexFailed)?;

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
        git_commit_files(&files, message).map_err(GeneralError::GitError)
    }
}

