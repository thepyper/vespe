use gix::bstr::ByteSlice;
use gix::objs::tree::EntryMode;
use gix::prelude::*;
use gix::{ObjectId, Repository};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use thiserror::Error as ThisError;
use tracing::debug;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("Failed to open repository: {message}")]
    GitRepository { message: String },
    #[error("No work directory found for the repository")]
    NoWorkdir,
    #[error("Failed to get HEAD commit: {0}")]
    HeadCommit(String),
    #[error("Failed to get tree from commit: {0}")]
    TreeFromCommit(String),
    #[error("Failed to load repository index: {0}")]
    RepositoryIndex(String),
    #[error("Failed to get repository status: {0}")]
    RepositoryStatus(String),
    #[error("Failed to restore index: {0}")]
    RestoreIndex(String),
    #[error("Failed to canonicalize path '{path}': {source}")]
    CanonicalizePath {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("Path '{file_path}' is outside the work directory '{workdir}'")]
    PathOutsideWorkdir { file_path: PathBuf, workdir: PathBuf },
    #[error("Failed to add file '{file_path}' to index: {message}")]
    AddFileToIndex { file_path: PathBuf, message: String },
    #[error("Failed to write index: {0}")]
    WriteIndex(String),
    #[error("Failed to write tree: {0}")]
    WriteTree(String),
    #[error("Failed to find tree '{oid}': {message}")]
    FindTree { oid: ObjectId, message: String },
    #[error("Failed to create signature: {0}")]
    CreateSignature(String),
    #[error("Failed to create commit: {0}")]
    CreateCommit(String),
    #[error("Failed to find commit '{oid}': {message}")]
    FindCommit { oid: ObjectId, message: String },
    #[error("Gix error: {0}")]
    Gix(String),
}

impl From<Box<dyn std::error::Error + Send + Sync>> for Error {
    fn from(e: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Error::Gix(e.to_string())
    }
}

pub fn git_commit_files(
    root_path: &Path,
    files_to_commit: &[PathBuf],
    message: &str,
) -> Result<(), Error> {
    debug!(
        "Running git_commit_files with message {} on files {:?}",
        message, files_to_commit
    );

    let repo = gix::discover(root_path)
        .map_err(|e| Error::GitRepository {
            message: format!("Failed to open repository: {}", e),
        })?;

    let workdir = repo
        .work_dir()
        .ok_or(Error::NoWorkdir)?
        .to_path_buf();

    // Convert files_to_commit to a HashSet for efficient lookup
    let files_to_commit_set: HashSet<PathBuf> = files_to_commit.iter().cloned().collect();

    // 1. Get the current HEAD commit
    let head_ref = repo
        .head()
        .map_err(|e| Error::HeadCommit(e.to_string()))?;
    let head_commit = head_ref
        .peel_to_commit_in_place()
        .map_err(|e| Error::HeadCommit(e.to_string()))?;
    let head_commit_id = head_commit.id;

    // 2. Get the tree associated with the HEAD commit
    let head_tree_id = head_commit.tree_id().map_err(|e| Error::TreeFromCommit(e.to_string()))?;

    // 3. Load the repository index
    let mut index = repo
        .index_or_empty()
        .map_err(|e| Error::RepositoryIndex(e.to_string()))?;

    // 4. Identify files that were staged before our operation
    let mut files_to_re_stage: Vec<PathBuf> = Vec::new();

    // Get status to find staged files
    let mut status_platform = repo
        .status(gix::progress::Discard)
        .map_err(|e| Error::RepositoryStatus(e.to_string()))?;

    for item in status_platform
        .into_iter()
        .map_err(|e| Error::RepositoryStatus(e.to_string()))?
    {
        let item = item.map_err(|e| Error::RepositoryStatus(e.to_string()))?;
        let rela_path = item.rela_path().to_path().map_err(|e| {
            Error::RepositoryStatus(format!("Failed to convert path: {}", e))
        })?;
        
        // Check if file is staged (index changed) and not in our commit list
        if item.index_status().is_some() {
            let path = workdir.join(rela_path);
            if !files_to_commit_set.contains(&path) {
                files_to_re_stage.push(path);
            }
        }
    }

    // 5. Reset the index to HEAD
    let head_tree = repo
        .find_object(head_tree_id)
        .map_err(|e| Error::FindTree {
            oid: head_tree_id,
            message: e.to_string(),
        })?
        .peel_to_tree()
        .map_err(|e| Error::TreeFromCommit(e.to_string()))?;

    index
        .set_state_from_tree(&head_tree_id)
        .map_err(|e| Error::RestoreIndex(e.to_string()))?;

    // 6. Add files_to_commit to the index
    for path in files_to_commit {
        let canonical_path = path
            .canonicalize()
            .map_err(|e| Error::CanonicalizePath {
                path: path.clone(),
                source: e,
            })?;
        let canonical_workdir =
            workdir
                .canonicalize()
                .map_err(|e| Error::CanonicalizePath {
                    path: workdir.clone(),
                    source: e,
                })?;

        let relative_path = canonical_path
            .strip_prefix(&canonical_workdir)
            .map_err(|_| Error::PathOutsideWorkdir {
                file_path: path.clone(),
                workdir: workdir.clone(),
            })?;

        // Read file content and add to index
        let content = std::fs::read(path).map_err(|e| Error::AddFileToIndex {
            file_path: path.clone(),
            message: e.to_string(),
        })?;

        let odb = repo.objects.clone();
        let blob_id = odb
            .write_blob(&content)
            .map_err(|e| Error::AddFileToIndex {
                file_path: path.clone(),
                message: e.to_string(),
            })?;

        // Get file metadata for mode
        let metadata = std::fs::metadata(path).map_err(|e| Error::AddFileToIndex {
            file_path: path.clone(),
            message: e.to_string(),
        })?;

        let mode = if metadata.is_dir() {
            gix::objs::tree::EntryKind::Tree
        } else {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if metadata.permissions().mode() & 0o111 != 0 {
                    gix::objs::tree::EntryKind::BlobExecutable
                } else {
                    gix::objs::tree::EntryKind::Blob
                }
            }
            #[cfg(not(unix))]
            {
                gix::objs::tree::EntryKind::Blob
            }
        };

        let entry = gix::index::Entry::from_stat(
            &gix::index::entry::stat::Stat::from_fs(&metadata)
                .map_err(|e| Error::AddFileToIndex {
                    file_path: path.clone(),
                    message: e.to_string(),
                })?,
            blob_id,
            gix::index::entry::Flags::empty(),
        );

        index
            .set_entry(relative_path, entry)
            .map_err(|e| Error::AddFileToIndex {
                file_path: path.clone(),
                message: e.to_string(),
            })?;
    }

    index
        .write(Default::default())
        .map_err(|e| Error::WriteIndex(e.to_string()))?;

    // 8. Create a new tree from the index
    let tree_id = index
        .write_tree()
        .map_err(|e| Error::WriteTree(e.to_string()))?;

    // 9. Create the commit
    let author = gix::actor::SignatureRef {
        name: "vespe".into(),
        email: "vespe@example.com".into(),
        time: gix::date::Time::now_local_or_utc(),
    };
    let committer = author.clone();

    let new_commit_id = repo
        .commit_as(
            &author,
            &committer,
            "HEAD",
            message,
            tree_id,
            [head_commit_id],
        )
        .map_err(|e| Error::CreateCommit(e.to_string()))?
        .detach();

    // 11. Reset index to the new HEAD
    let new_tree_id = repo
        .find_commit(new_commit_id)
        .map_err(|e| Error::FindCommit {
            oid: new_commit_id,
            message: e.to_string(),
        })?
        .tree_id()
        .map_err(|e| Error::TreeFromCommit(e.to_string()))?;

    index
        .set_state_from_tree(&new_tree_id)
        .map_err(|e| Error::RestoreIndex(e.to_string()))?;

    // 12. Re-add the files_to_re_stage
    for path in files_to_re_stage {
        let canonical_path = path
            .canonicalize()
            .map_err(|e| Error::CanonicalizePath {
                path: path.clone(),
                source: e,
            })?;
        let canonical_workdir =
            workdir
                .canonicalize()
                .map_err(|e| Error::CanonicalizePath {
                    path: workdir.clone(),
                    source: e,
                })?;

        let relative_path = canonical_path
            .strip_prefix(&canonical_workdir)
            .map_err(|_| Error::PathOutsideWorkdir {
                file_path: path.clone(),
                workdir: workdir.clone(),
            })?;

        if path.exists() {
            let content = std::fs::read(&path).map_err(|e| Error::AddFileToIndex {
                file_path: path.clone(),
                message: e.to_string(),
            })?;

            let odb = repo.objects.clone();
            let blob_id = odb
                .write_blob(&content)
                .map_err(|e| Error::AddFileToIndex {
                    file_path: path.clone(),
                    message: e.to_string(),
                })?;

            let metadata = std::fs::metadata(&path).map_err(|e| Error::AddFileToIndex {
                file_path: path.clone(),
                message: e.to_string(),
            })?;

            let mode = if metadata.is_dir() {
                gix::objs::tree::EntryKind::Tree
            } else {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if metadata.permissions().mode() & 0o111 != 0 {
                        gix::objs::tree::EntryKind::BlobExecutable
                    } else {
                        gix::objs::tree::EntryKind::Blob
                    }
                }
                #[cfg(not(unix))]
                {
                    gix::objs::tree::EntryKind::Blob
                }
            };

            let entry = gix::index::Entry::from_stat(
                &gix::index::entry::stat::Stat::from_fs(&metadata)
                    .map_err(|e| Error::AddFileToIndex {
                        file_path: path.clone(),
                        message: e.to_string(),
                    })?,
                blob_id,
                gix::index::entry::Flags::empty(),
            );

            index
                .set_entry(relative_path, entry)
                .map_err(|e| Error::AddFileToIndex {
                    file_path: path.clone(),
                    message: e.to_string(),
                })?;
        }
    }

    // 13. Write the restored index
    index
        .write(Default::default())
        .map_err(|e| Error::WriteIndex(e.to_string()))?;

    debug!("Commit created with id {}", new_commit_id);
    Ok(())
}

pub fn is_in_git_repository(root_path: &Path) -> Result<bool, Error> {
    match gix::discover(root_path) {
        Ok(_) => Ok(true),
        Err(e) => {
            // Check if the error is "not found"
            if e.to_string().contains("not found")
                || e.to_string().contains("Not a git repository")
            {
                Ok(false)
            } else {
                Err(Error::Gix(e.to_string()))
            }
        }
    }
}