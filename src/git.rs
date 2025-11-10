use anyhow::{Context, Result};
use git2::{Repository, Signature, StatusOptions};
use std::collections::HashSet;
#[allow(unused_imports)]
use std::path::Path;
use std::path::PathBuf;
use tracing::debug; // Added HashSet

pub fn git_commit_files(files_to_commit: &[PathBuf], message: &str) -> Result<()> {
    debug!(
        "Running git_commit_files wth message {} on files {:?}",
        message, files_to_commit
    );

    let repo = Repository::open(".").context("Failed to open repository")?;
    let workdir = repo.workdir().context("Repository has no workdir")?;

    // Convert files_to_commit to a HashSet for efficient lookup
    let files_to_commit_set: HashSet<PathBuf> = files_to_commit.iter().cloned().collect();

    // 1. Get the current HEAD commit. This will be the parent of the new commit
    // and the base for "cleaning" the staging area.
    let head_commit = repo
        .head()
        .and_then(|head| head.peel_to_commit())
        .context("Failed to get HEAD commit")?;

    // 2. Get the tree associated with the HEAD commit.
    // This represents the state of the repository at the last commit.
    let head_tree = head_commit
        .tree()
        .context("Failed to get tree from HEAD commit")?;

    // 3. Load the repository index.
    let mut index = repo.index().context("Failed to get repository index")?;

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
        .context("Failed to get repository status")?;

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
        .context("Failed to read HEAD tree into index")?;

    // 6. Selectively add files_to_commit to the staging area (index).
    for path in files_to_commit {
        let canonical_path = path
            .canonicalize()
            .with_context(|| format!("Failed to canonicalize path: {}", path.display()))?;
        let canonical_workdir = workdir
            .canonicalize()
            .with_context(|| format!("Failed to canonicalize workdir: {}", workdir.display()))?;

        let relative_path = canonical_path
            .strip_prefix(&canonical_workdir)
            .with_context(|| {
                format!(
                    "File {} is outside the repository workdir at {}",
                    path.display(),
                    workdir.display()
                )
            })?;
        index
            .add_path(relative_path)
            .with_context(|| format!("Failed to add file '{}' to index", path.display()))?;
    }

    // 7. Write the changes to the index to disk.
    index.write().context("Failed to write index")?;

    // 8. Create a new tree based on the current state of the index.
    let tree_oid = index
        .write_tree()
        .context("Failed to write tree from index")?;
    let tree = repo.find_tree(tree_oid).context("Failed to find tree")?;

    // 9. Create the author and committer signature.
    let signature = Signature::now("vespe", "vespe@example.com") // Using "vespe@example.com" as a placeholder
        .context("Failed to create signature")?;

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
        .context("Failed to create commit")?;

    // 11. Reset the index to the *new* HEAD. This is crucial to avoid the "ghost" effect
    //     for the newly committed files.
    let new_head_commit = repo
        .find_commit(new_commit_oid)
        .context("Failed to find the new commit")?;
    let new_head_tree = new_head_commit
        .tree()
        .context("Failed to get tree from the new commit")?;

    index
        .read_tree(&new_head_tree)
        .context("Failed to restore index to new HEAD state")?;

    // 12. Re-add the `files_to_re_stage` to the index.
    for path in files_to_re_stage {
        let canonical_path = path
            .canonicalize()
            .with_context(|| format!("Failed to canonicalize path: {}", path.display()))?;
        let canonical_workdir = workdir
            .canonicalize()
            .with_context(|| format!("Failed to canonicalize workdir: {}", workdir.display()))?;

        let relative_path = canonical_path
            .strip_prefix(&canonical_workdir)
            .with_context(|| {
                format!(
                    "File {} is outside the repository workdir at {}",
                    path.display(),
                    workdir.display()
                )
            })?;
        index
            .add_path(relative_path)
            .with_context(|| format!("Failed to re-add file '{}' to index", path.display()))?;
    }

    // 13. Write the restored index to disk.
    index.write().context("Failed to write restored index")?;

    debug!("Commit created with id {}", new_commit_oid);
    Ok(())
}

pub fn is_in_git_repository(path: &Path) -> Result<bool, git2::Error> {
    // Tenta di scoprire un repository Git a partire dalla directory attuale
    // `Repository::discover` cerca verso l'alto nella gerarchia delle directory.
    match Repository::discover(&path) {
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

pub struct Commit {
    pub files: HashSet<PathBuf>,
}

impl Commit {
    pub fn new() -> Self {
        Commit {
            files: HashSet::new(),
        }
    }
    pub fn commit(&self, message: &str) -> Result<()> {
        if self.files.is_empty() {
            return Ok(());
        }
        let files = self.files.iter().cloned().collect::<Vec<PathBuf>>();
        git_commit_files(&files, message)
    }
}
