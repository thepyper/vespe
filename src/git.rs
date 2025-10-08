use std::path::PathBuf;
use anyhow::{Result, Context};
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
            let commit = head.try_into_peeled_id()
                .context("Failed to peel HEAD to commit")?
                .unwrap() // TODO better handling!!
                .object()?
                .into_commit();
            let index = repo.index()
                .context("Failed to load index")?;
            (vec![commit.id], index)
        },
        HeadKind::Unborn(_) => {
            let index = gix::index::File::at_or_default(repo.index_path(), repo.object_hash(), false, gix::index::decode::Options::default())
                .context("Failed to create new index for unborn branch")?;
            (Vec::new(), index)
        },
    };

    // 1. Identify initially staged files
    let mut initially_staged_paths: Vec<PathBuf> = Vec::new();
    for entry in index.entries() {
        if entry.stage == 0 {
            initially_staged_paths.push(gix::path::from_bstr(entry.path.as_ref()).into_owned());
        }
    }

    // 2. Add files_to_commit
    for file_path in files_to_commit {
        index.add_path(file_path)
            .map(|_| ())
            .with_context(|| format!("Failed to add path to index: {:?}", file_path))?;
    }

    // 3. Unstage other files that were initially staged but not in files_to_commit
    let mut to_re_stage: Vec<PathBuf> = Vec::new();
    for staged_path in &initially_staged_paths {
        if !files_to_commit.contains(staged_path) {
            index.remove_path(staged_path);
            to_re_stage.push(staged_path.clone());
        }
    }

    // 4. Write the index
    index.write(gix::index::write::Options::default())
        .context("Failed to write index")?;

    // 5. Create the commit
    let tree_id = index.write_tree()
        .context("Failed to write tree from index")?;

    //let author_signature = gix::actor::Signature::new_now("User", "user@example.com")
    //    .context("Failed to create author signature")?;
    //let committer_signature = author_signature.clone();
    
    let committer_signature = gix::actor::Signature::default(); // TODO better?

    let commit_message = format!("{}\n\n{}", message, comment);

    if let HeadKind::Unborn(_) = head.kind {
        let commit_id = repo.commit(
            "refs/heads/main",
            &commit_message,
            tree_id,
            parent_ids.iter().map(|id| *id),
        )
        .context("Failed to create initial commit")?;

        // TODO serve!?!?!?
        //repo.refs.set_symbolic_ref("HEAD", "refs/heads/main")
        //    .context("Failed to set HEAD to main branch")?;
    } else {
        repo.commit(
            "HEAD",
            &commit_message,
            tree_id,
            parent_ids.iter().map(|id| *id),
        )
        .context("Failed to create commit")?;
    };

    // 7. Re-stage previously unstaged files
    for file_path in to_re_stage {
        index.add_path(&file_path)
            .map(|_|())
            .with_context(|| format!("Failed to re-add path to index: {:?}", file_path))?;
    }

    // 8. Write the index again
    index.write(gix::index::write::Options::default())
        .context("Failed to write index after re-staging")?;

    Ok(())
}