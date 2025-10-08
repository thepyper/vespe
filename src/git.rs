use std::path::PathBuf;
use anyhow::{Result, Context, anyhow};
use gix::{
    self,
    prelude::*,
    head::Kind as HeadKind,
    actor::Signature,
};

pub fn git_commit(files_to_commit: &[PathBuf], message: &str, comment: &str) -> Result<()> {
    let repo = gix::discover(".")
        .context("Failed to discover git repository")?;

    let head = repo.head()
        .context("Failed to get HEAD reference")?;

    let (parent_ids, initial_index_state) = match head.kind {
        HeadKind::Symbolic(_reference) => {
            let commit = head.peel_to_commit_in_place() 
                .context("Failed to peel HEAD reference to commit")?;
            (vec![commit.id], repo.index_or_load_from_head().context("Failed to get index from HEAD")?)
        },
        HeadKind::Detached { .. } => {
            let commit = head.peel_to_commit_in_place() 
                .context("Failed to peel detached HEAD to commit")?;
            (vec![commit.id], repo.index_or_load_from_head().context("Failed to get index from HEAD")?)
        },
        HeadKind::Unborn(_fullname) => {
            (vec![], gix::index::State::new(repo.object_hash()))
        },
    };

    let mut index = initial_index_state;

    // 1. Identify initially staged files
    let mut initially_staged_paths: Vec<PathBuf> = Vec::new();
    for entry in index.entries() {
        if entry.stage() == 0 { // Stage 0 means it's in the index
            initially_staged_paths.push(entry.path.to_owned().into());
        }
    }

    // 2. Add files_to_commit
    for file_path in files_to_commit {
        index.add_path(file_path)
            .context(format!("Failed to add path to index: {:?}", file_path))?;
    }

    // 3. Unstage other files that were initially staged but not in files_to_commit
    let mut to_re_stage: Vec<PathBuf> = Vec::new();
    for staged_path in initially_staged_paths {
        if !files_to_commit.contains(&staged_path) {
            index.remove_path(&staged_path)
                .context(format!("Failed to remove path from index: {:?}", staged_path))?;
            to_re_stage.push(staged_path);
        }
    }

    // 4. Write the index
    index.write(gix::index::write::Options::default())
        .context("Failed to write index")?;

    // 5. Create the commit
    let tree_id = index.write_tree()
        .context("Failed to write tree from index")?;

    let committer_signature = repo.committer()
        .context("Failed to get committer signature")?;
    let author_signature = repo.author()
        .context("Failed to get author signature")?;
    let commit_message = format!("{}\n\n{}", message, comment);

    let commit_id = repo.commit(
        &author_signature, // Author
        &committer_signature,         // Committer
        commit_message,    // Pass String directly
        tree_id,
        parent_ids.iter(), // Pass parent_ids as an iterator
    )
    .context("Failed to create commit")?;

    // 6. Update HEAD
    match head.kind {
        HeadKind::Symbolic(_reference) => {
            let mut head_ref = repo.find_reference(head.name())?;
            head_ref.set_target_id(commit_id, "commit")
                .context("Failed to update symbolic HEAD reference")?;
        },
        HeadKind::Detached { .. } => {
            repo.set_head_fully_detached(commit_id)
                .context("Failed to set detached HEAD")?;
        },
        HeadKind::Unborn(_fullname) => {
            let mut head_update = repo.head_update();
            let mut head_ref = head_update
                .create_new_branch("main")
                .context("Failed to create initial 'main' branch for unborn repository")?;
            head_ref.set_target_id(commit_id, "initial commit")
                .context("Failed to set HEAD to initial commit")?;
        },
    }

    // 7. Re-stage previously unstaged files
    for file_path in to_re_stage {
        index.add_path(&file_path)
            .context(format!("Failed to re-add path to index: {:?}", file_path))?;
    }

    // 8. Write the index again
    index.write(gix::index::write::Options::default())
        .context("Failed to write index after re-staging")?;

    Ok(())
}
