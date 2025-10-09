use std::path::PathBuf;
use anyhow::Result;
use gix::bstr::{BString, ByteSlice};
use tracing::debug;

pub fn git_commit(files_to_commit: &[PathBuf], message: &str, _comment: &str) -> Result<()> {
    let repo = gix::open(".")?;
    let workdir = repo.work_dir().ok_or_else(|| anyhow::anyhow!("bare repository not supported"))?;

    // 1. Get the parent commit
    let head = repo.head()?;
    let parent_commit = head.peel_to_commit_in_place()?;

    // 2. Create an in-memory index, populated from the parent commit's tree
    let mut index = gix::index::State::from_tree(&parent_commit.tree()?, |oid, buf| repo.objects.find(oid, buf))?;

    // 3. Add the specified files to the index
    for file_path in files_to_commit {
        let relative_path = file_path.strip_prefix(workdir)?;
        let content = std::fs::read(file_path)?;
        let blob_oid = repo.write_blob(content)?;

        let entry_path = gix::bstr::BString::from(relative_path.to_string_lossy().as_bytes());

        let entry = index.entry_mut_by_path_and_stage(&entry_path, 0).ok_or_else(|| anyhow::anyhow!("file not in parent commit"))?;
        // TODO: handle new files
        entry.id = blob_oid;
    }

    // 4. Write the index to a new tree
    let tree_oid = index.write_tree_to(&mut repo.objects)?;

    // 5. Create the commit
    let author = gix::actor::Signature::from_config(&repo.config_snapshot())?
        .ok_or_else(|| anyhow::anyhow!("Could not determine author from git config"))?;
    let committer = author.clone();

    let mut commit = gix::object::Commit::new_root(author, committer, message.into());
    commit.tree_id = tree_oid;
    commit.parents = vec![parent_commit.id].into();

    let commit_oid = repo.objects.write_commit(&commit)?;

    // 6. Update HEAD
    let head_ref = head.into_referent();
    repo.edit_reference(gix::refs::transaction::RefEdit {
        change: gix::refs::transaction::Change::Update {
            log: gix::refs::transaction::LogChange {
                mode: gix::refs::log::RefLog::AndReference,
                force_create_reflog: false,
                message: format!("commit: {}", message).into(),
            },
            expected: gix::refs::transaction::PreviousValue::MustExistAndMatch(gix::refs::Target::Peeled(parent_commit.id)),
            new: gix::refs::Target::Peeled(commit_oid),
        },
        name: head_ref.name().to_owned(),
        deref: true,
    })?;

    Ok(())
}