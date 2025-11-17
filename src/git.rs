use crate::error::{Error, Result};
use git2::{Repository, Signature, StatusOptions};
use std::collections::HashSet;
#[allow(unused_imports)]
use std::path::Path;
use std::path::PathBuf;
use tracing::debug; // Added HashSet

pub fn git_commit_files(
    root_path: &Path,
    files_to_commit: &[PathBuf],
    message: &str,
) -> Result<()> {
    debug!(
        "Running git_commit_files wth message {} on files {:?}",
        message, files_to_commit
    );

    let repo = Repository::discover(root_path).map_err(|e| Error::GitRepositoryError {
        message: "Failed to open repository".to_string(),
        source: e,
    })?;
    let workdir = repo.workdir().ok_or(Error::NoWorkdirError)?;

    // Convert files_to_commit to a HashSet for efficient lookup
    let files_to_commit_set: HashSet<PathBuf> = files_to_commit.iter().cloned().collect();

    // 1. Get the current HEAD commit. This will be the parent of the new commit
    // and the base for "cleaning" the staging area.
    let head_commit = repo
        .head()
        .and_then(|head| head.peel_to_commit())
        .map_err(Error::HeadCommitError)?;

    // 2. Get the tree associated with the HEAD commit.
    // This represents the state of the repository at the last commit.
    let head_tree = head_commit.tree().map_err(Error::TreeFromCommitError)?;

    // 3. Load the repository index.
    let mut index = repo.index().map_err(Error::RepositoryIndexError)?;

    // 4. Identify files that were staged *before* our operation,
    //    excluding those that will be committed.
    let mut files_to_re_stage: Vec<PathBuf> = Vec::new();
    let mut status_options = StatusOptions::new();
    status_options
        .include_ignored(false)
        .include_untracked(false)
        .recurse_untracked_dirs(false)
        .exclude_submodules(true);

    let statuses = repo
        .statuses(Some(&mut status_options))
        .map_err(Error::RepositoryStatusError)?;

    for entry in statuses.iter() {
        if let Some(path_str) = entry.path() {
            let path = PathBuf::from(path_str);
            let status = entry.status();
            // Check if the file is staged (IndexNew, IndexModified, etc.)
            // and *not* part of the files we are about to commit.
            if (status.is_index_new()
                || status.is_index_modified()
                || status.is_index_deleted()
                || status.is_index_renamed()
                || status.is_index_typechange())
                && !files_to_commit_set.contains(&path)
            {
                files_to_re_stage.push(path);
            }
        }
    }

    // 5. Reset the index to the HEAD. This "unstages" everything.
    index
        .read_tree(&head_tree)
        .map_err(Error::RestoreIndexError)?;

    // 6. Selectively add files_to_commit to the staging area (index).
    for path in files_to_commit {
        let canonical_path = path
            .canonicalize()
            .map_err(|e| Error::CanonicalizePathError {
                path: path.clone(),
                source: e,
            })?;
        let canonical_workdir =
            workdir
                .canonicalize()
                .map_err(|e| Error::CanonicalizePathError {
                    path: workdir.to_path_buf(),
                    source: e,
                })?;

        let relative_path = canonical_path
            .strip_prefix(&canonical_workdir)
            .map_err(|_| Error::PathOutsideWorkdirError {
                file_path: path.clone(),
                workdir: workdir.to_path_buf(),
            })?;
        index
            .add_path(relative_path)
            .map_err(|e| Error::AddFileToIndexError {
                file_path: path.clone(),
                source: e,
            })?;
    }

    // 7. Write the changes to the index to disk.
    index.write().map_err(Error::WriteIndexError)?;

    // 8. Create a new tree based on the current state of the index.
    let tree_oid = index.write_tree().map_err(Error::WriteTreeError)?;
    let tree = repo.find_tree(tree_oid).map_err(|e| Error::FindTreeError {
        oid: tree_oid,
        source: e,
    })?;

    // 9. Create the author and committer signature.
    let signature = Signature::now("vespe", "vespe@example.com") // Using "vespe@example.com" as a placeholder
        .map_err(Error::CreateSignatureError)?;

    // 10. Create the new commit.
    let new_commit_oid = repo
        .commit(
            Some("HEAD"), // Update HEAD to point to the new commit
            &signature,
            &signature,
            message,
            &tree,
            &[&head_commit], // The new commit has the current HEAD as parent
        )
        .map_err(Error::CreateCommitError)?;

    // 11. Reset the index to the *new* HEAD. This is crucial to avoid the "ghost" effect
    //     for the newly committed files.
    let new_head_commit = repo
        .find_commit(new_commit_oid)
        .map_err(|e| Error::FindCommitError {
            oid: new_commit_oid,
            source: e,
        })?;
    let new_head_tree = new_head_commit.tree().map_err(Error::TreeFromCommitError)?;

    index
        .read_tree(&new_head_tree)
        .map_err(Error::RestoreIndexError)?;

    // 12. Re-add the `files_to_re_stage` to the index.
    for path in files_to_re_stage {
        let canonical_path = path
            .canonicalize()
            .map_err(|e| Error::CanonicalizePathError {
                path: path.clone(),
                source: e,
            })?;
        let canonical_workdir =
            workdir
                .canonicalize()
                .map_err(|e| Error::CanonicalizePathError {
                    path: workdir.to_path_buf(),
                    source: e,
                })?;

        let relative_path = canonical_path
            .strip_prefix(&canonical_workdir)
            .map_err(|_| Error::PathOutsideWorkdirError {
                file_path: path.clone(),
                workdir: workdir.to_path_buf(),
            })?;
        index
            .add_path(relative_path)
            .map_err(|e| Error::AddFileToIndexError {
                file_path: path.clone(),
                source: e,
            })?;
    }

    // 13. Write the restored index to disk.
    index.write().map_err(Error::WriteIndexError)?;

    debug!("Commit created with id {}", new_commit_oid);
    Ok(())
}

pub fn is_in_git_repository(root_path: &Path) -> std::result::Result<bool, git2::Error> {
    // Tenta di scoprire un repository Git a partire dalla directory attuale
    // `Repository::discover` cerca verso l'alto nella gerarchia delle directory.
    match Repository::discover(&root_path) {
        Ok(_) => Ok(true), // Un repository è stato trovato
        Err(e) => {
            // Se l'errore indica che non è stato trovato un repository,
            // allora la directory non è in un repo Git.
            // Altrimenti, è un altro tipo di errore.
            if e.code() == git2::ErrorCode::NotFound {
                Ok(false)
            } else {
                // Altri errori (es. permessi, repository corrotto, ecc.)
                Err(e)
            }
        }
    }
}
