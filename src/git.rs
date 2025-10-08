use std::path::PathBuf;
use anyhow::{Result, Context, anyhow};
use gix::{
    self,
    prelude::*,
    head::Kind as HeadKind,
};

pub fn git_commit(files_to_commit: &[PathBuf], message: &str, comment: &str) -> Result<()> {
    let repo = gix::discover(".")
        .context("Failed to discover git repository")?;

    let head = repo.head()
        .context("Failed to get HEAD reference")?;

    let (parent_ids, mut index) = match head.kind {
        HeadKind::Symbolic(_) | HeadKind::Detached { .. } => {
            let commit = head.try_into_fully_peeled_id()
                .context("Failed to peel HEAD to commit")?
                .object()?
                .into_commit();
            let index = repo.index()
                .context("Failed to load index")?;
            (vec![commit.id], index)
        },
        HeadKind::Unborn(_) => {
            let index = gix::index::File::new(repo.object_hash(), repo.index_path(), gix::index::decode::Options::default());
            (Vec::new(), index)
        },
    };

    let worktree = repo.worktree().context("Failed to get worktree")?;

    // 1. Identify initially staged files
    let mut initially_staged_paths: Vec<PathBuf> = Vec::new();
    for entry in index.entries() {
        if entry.stage() == 0 {
            initially_staged_paths.push(entry.path_in(&worktree).to_path_buf());
        }
    }

    // 2. Add files_to_commit
    for file_path in files_to_commit {
        worktree.add_path_to_index(&mut index, file_path)
            .with_context(|| format!("Failed to add path to index: {:?}", file_path))?;
    }

    // 3. Unstage other files that were initially staged but not in files_to_commit
    let mut to_re_stage: Vec<PathBuf> = Vec::new();
    for staged_path in &initially_staged_paths {
        if !files_to_commit.contains(staged_path) {
            worktree.remove_path_from_index(&mut index, staged_path)
                .with_context(|| format!("Failed to remove path from index: {:?}", staged_path))?;
            to_re_stage.push(staged_path.clone());
        }
    }

    // 4. Write the index
    index.write()
        .context("Failed to write index")?;

    // 5. Create the commit
    let tree_id = index.write_tree()
        .context("Failed to write tree from index")?;

    let config = repo.config_snapshot();
    let author_signature = gix::actor::Signature::now("User", "user@example.com")
        .context("Failed to create author signature")?;
    let committer_signature = author_signature.clone();
    
    let commit_message = format!("{}\n\n{}", message, comment);

    let commit_id = repo.commit(
        &head,
        commit_message.as_str(),
        tree_id,
        parent_ids.iter().map(|id| *id),
    )
    .context("Failed to create commit")?;

    // 6. Update HEAD
    match head.kind {
        HeadKind::Symbolic(_) => {
            // In gix 0.73, symbolic HEAD is automatically updated by repo.commit()
        },
        HeadKind::Detached { .. } => {
            // Detached HEAD is automatically handled by repo.commit()
        },
        HeadKind::Unborn(_) => {
            // For unborn repository, create the main branch
            repo.reference(
                "refs/heads/main",
                commit_id,
                gix::refs::transaction::PreviousValue::MustNotExist,
                "initial commit",
            )
            .context("Failed to create initial 'main' branch for unborn repository")?;
        },
    }

    // 7. Re-stage previously unstaged files
    for file_path in to_re_stage {
        worktree.add_path_to_index(&mut index, &file_path)
            .with_context(|| format!("Failed to re-add path to index: {:?}", file_path))?;
    }

    // 8. Write the index again
    index.write()
        .context("Failed to write index after re-staging")?;

    Ok(())
}