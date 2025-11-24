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
    #[error("Failed to open repository: {0}")]
    OpenRepository(#[from] gix::open::Error),
    #[error("No work directory found for the repository")]
    NoWorkdir,
    #[error("Failed to get HEAD reference: {0}")]
    HeadReference(#[from] gix::reference::find::existing::Error),
    #[error("Failed to get HEAD commit: {0}")]
    HeadCommit(#[from] gix::head::peel::to_commit::Error),
    #[error("Failed to get tree from commit: {0}")]
    TreeFromCommit(#[from] gix::object::peel::to_kind::Error),
    #[error("Failed to load repository index: {0}")]
    RepositoryIndex(#[from] gix::worktree::open_index::Error),
    #[error("Failed to get repository status: {0}")]
    RepositoryStatus(#[from] gix::status::Error),
    #[error("Failed to restore index: {0}")]
    RestoreIndex(#[from] gix::index::State::Error), // This might still be ambiguous
    #[error("Failed to canonicalize path '{path}': {source}")]
    CanonicalizePath {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("Path '{file_path}' is outside the work directory '{workdir}'")]
    PathOutsideWorkdir {
        file_path: PathBuf,
        workdir: PathBuf,
    },
    #[error("Failed to add file '{file_path}' to index: {source}")]
    AddFileToIndex {
        file_path: PathBuf,
        #[source]
        source: std::io::Error, // For fs::read
    },
    #[error("Failed to write blob: {0}")]
    WriteBlob(#[from] gix::object::write::Error),
    #[error("Failed to get file metadata for '{file_path}': {source}")]
    FileMetadata {
        file_path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("Failed to create index entry from stat: {0}")]
    IndexEntryFromStat(#[from] gix::index::Error), // This might still be ambiguous
    #[error("Failed to set index entry for '{file_path}': {source}")]
    SetIndexEntry {
        file_path: PathBuf,
        #[source]
        source: gix::index::Error, // This might still be ambiguous
    },
    #[error("Failed to write index: {0}")]
    WriteIndex(#[from] gix::index::file::write::Error),
    #[error("Failed to write tree: {0}")]
    WriteTree(#[from] gix::index::write_tree::Error),
    #[error("Failed to create commit: {0}")]
    CreateCommit(#[from] gix::commit::Error),
    #[error("Failed to find commit '{oid}': {oid}")]
    FindCommit {
        oid: ObjectId,
        #[source]
        source: gix::object::find::Error,
    },
    #[error("Gix discover error: {0}")]
    GixDiscover(#[from] gix::discover::Error),
    #[error("Gix object decode error: {0}")]
    GixObjectDecode(#[from] gix::object::decode::Error),
    #[error("Gix commit error: {0}")]
    GixCommit(#[from] gix::commit::Error),
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

    let repo = gix::discover(root_path)?;

    let workdir = repo
        .work_dir()
        .ok_or(Error::NoWorkdir)?
        .to_path_buf();

    let files_to_commit_set: HashSet<PathBuf> = files_to_commit.iter().cloned().collect();

    // 1. Get the current HEAD commit
    let head_ref = repo.head()?;
    let head_commit = head_ref.peel_to_commit_in_place()?;
    let head_commit_id = head_commit.id;

    // 2. Get the tree associated with the HEAD commit
    let head_tree_id = head_commit.tree_id()?;

    // 3. Load the repository index
    let mut index = repo.index_or_empty()?.into_mutable();

    // 4. Identify files that were staged before our operation
    let mut files_to_re_stage: Vec<PathBuf> = Vec::new();

    let status_platform = repo.status(gix::progress::Discard)?;

    for item in status_platform.iter() {
        let item = item?;
        let rela_path = item.rela_path().to_path()?;
        
        if item.index_status().is_some() {
            let path = workdir.join(rela_path);
            if !files_to_commit_set.contains(&path) {
                files_to_re_stage.push(path);
            }
        }
    }

    // 5. Reset the index to HEAD
    index.set_state_from_tree(&head_tree_id)?;

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

        let content = std::fs::read(path).map_err(|e| Error::AddFileToIndex {
            file_path: path.clone(),
            source: e,
        })?;

        let odb = repo.objects.clone();
        let blob_id = odb.write(gix::object::Blob::from_bytes(&content))?;

        let metadata = std::fs::metadata(path).map_err(|e| Error::FileMetadata {
            file_path: path.clone(),
            source: e,
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

        let stat = gix::index::entry::Stat::from_metadata(gix::index::fs::Metadata::from(metadata))?;
        let entry = gix::index::Entry {
            stat,
            mode: mode.into(),
            id: blob_id,
            path: relative_path.as_ref().into(),
            flags: gix::index::entry::Flags::empty(),
        };

        index
            .set_entry(relative_path.as_ref(), entry)
            .map_err(|e| Error::SetIndexEntry {
                file_path: path.clone(),
                source: e,
            })?;
    }

    index.write(Default::default())?;

    // 8. Create a new tree from the index
    let tree_id = index.write_tree()?;

    // 9. Create the commit
    let author = gix::actor::SignatureRef {
        name: "vespe".into(),
        email: "vespe@example.com".into(),
        time: gix::date::Time::now_local_or_utc(),
    };
    let committer = author.clone();

    let new_commit_id = repo
        .commit_as(
            author,
            committer,
            "HEAD",
            message,
            tree_id,
            [head_commit_id],
        )?
        .detach();

    // 11. Reset index to the new HEAD
    let new_tree_id = repo
        .find_object(new_commit_id)?
        .peel_to_commit()? // This line was changed in the file
        .tree_id()?; // This line was changed in the file

    index.set_state_from_tree(&new_tree_id)?;

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
                source: e,
            })?;

            let odb = repo.objects.clone();
            let blob_id = odb.write(&content[..])?;

            let metadata = std::fs::metadata(&path).map_err(|e| Error::FileMetadata {
                file_path: path.clone(),
                source: e,
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

                            let stat = gix::index::entry::Stat::from_metadata(gix::index::fs::Metadata::from(metadata))?;
                            let entry = gix::index::Entry {
                                stat,
                                mode: mode.into(),
                                id: blob_id,
                                path: relative_path.as_ref().into(),
                                flags: gix::index::entry::Flags::empty(),
                            };
            index
                .set_entry(relative_path.to_str().unwrap().as_bytes().into(), entry)
                .map_err(|e| Error::SetIndexEntry {
                    file_path: path.clone(),
                    source: e,
                })?;
        }
    }

    // 13. Write the restored index
    index.write(Default::default())?;

    debug!("Commit created with id {}", new_commit_id);
    Ok(())
}

pub fn is_in_git_repository(root_path: &Path) -> Result<bool, Error> {
    match gix::discover(root_path) {
        Ok(_) => Ok(true),
        Err(gix::open::Error::NotARepository) => Ok(false),
        Err(e) => Err(e.into()),
    }
}