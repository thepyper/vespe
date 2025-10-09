use git2::{Repository, Signature, Index, Oid, Tree};
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use tracing::debug;

pub fn git_commit(files_to_commit: &[PathBuf], message: &str, author_name: &str) -> Result<()> {
    let repo = Repository::open(".")
        .context("Failed to open repository")?;
    let workdir = repo.workdir().context("Repository has no workdir")?;

    // 1. Get the current HEAD commit. This will be the parent of the new commit
    // and the base for "cleaning" the staging area.
    let head_commit = repo.head()
        .and_then(|head| head.peel_to_commit())
        .context("Failed to get HEAD commit")?;

    // 2. Get the tree associated with the HEAD commit.
    // This represents the state of the repository at the last commit.
    let head_tree = head_commit.tree()
        .context("Failed to get tree from HEAD commit")?;

    // 3. Load the repository index.
    let mut index = repo.index()
        .context("Failed to get repository index")?;

    // 4. *** SAVE THE ORIGINAL INDEX STATE ***
    // Create a temporary tree object from the current index. This OID represents
    // the state of the index before our modifications.
    let original_index_tree_oid = index.write_tree()
        .context("Failed to save original index state")?;

    // 5. Update the index to match the HEAD tree.
    // This is the key operation: it simulates 'git reset --mixed HEAD' or 'git restore --staged .'.
    // It removes all previously staged changes from the index,
    // but *does not touch files in the working directory*.
    index.read_tree(&head_tree)
        .context("Failed to read HEAD tree into index")?;

    // 6. Selectively add files_to_commit to the staging area (index).
    for path in files_to_commit {
        let canonical_path = path.canonicalize().with_context(|| format!("Failed to canonicalize path: {}", path.display()))?;
        let canonical_workdir = workdir.canonicalize().with_context(|| format!("Failed to canonicalize workdir: {}", workdir.display()))?;

        let relative_path = canonical_path.strip_prefix(&canonical_workdir).with_context(|| {
            format!(
                "File {} is outside the repository workdir at {}",
                path.display(),
                workdir.display()
            )
        })?;
        index.add_path(relative_path)
            .with_context(|| format!("Failed to add file '{}' to index", path.display()))?;
    }

    // 7. Write the changes to the index to disk.
    index.write()
        .context("Failed to write index")?;

    // 8. Create a new tree based on the current state of the index.
    let tree_oid = index.write_tree()
        .context("Failed to write tree from index")?;
    let tree = repo.find_tree(tree_oid)
        .context("Failed to find tree")?;

    // 9. Create the author and committer signature.
    let signature = Signature::now(author_name, "vespe@example.com") // Using "vespe@example.com" as a placeholder
        .context("Failed to create signature")?;

    // 10. Create the new commit.
    let new_commit_oid = repo.commit(
        Some("HEAD"), // Update HEAD to point to the new commit
        &signature,
        &signature,
        message,
        &tree,
        &[&head_commit], // The new commit has the current HEAD as parent
    )
    .context("Failed to create commit")?;

    // 11. *** RESTORE THE ORIGINAL INDEX STATE ***
    // Find the tree object corresponding to the saved OID.
    let original_index_tree = repo.find_tree(original_index_tree_oid)
        .context("Failed to find original index tree")?;

    // Load the original tree into the index.
    index.read_tree(&original_index_tree)
        .context("Failed to restore index to original state")?;

    // Write the restored index to disk.
    index.write()
        .context("Failed to write restored index")?;

    debug!("Commit created with id {}", new_commit_oid);

    Ok(())
}

