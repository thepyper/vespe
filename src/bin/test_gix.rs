use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use gix::prelude::FindExt;
use gix::{
    index::State as IndexState,
    open::Options,
    status::{Status, StatusEntry},
    Repository,
};
use tempfile::TempDir;

fn create_test_repo() -> Result<(TempDir, Repository)> {
    let tmp_dir = TempDir::new()?;
    let repo_path = tmp_dir.path().join("repo");
    let repo = gix::init_bare(&repo_path)?;
    let repo = Repository::open_from_paths(repo_path)?;
    Ok((tmp_dir, repo))
}

fn create_file_and_add(repo: &Repository, path: &Path, content: &str) -> Result<()> {
    let full_path = repo.work_dir().context("no work dir")?.join(path);
    std::fs::create_dir_all(full_path.parent().context("no parent")?)?;
    std::fs::write(&full_path, content)?;
    let relative_path = path.strip_prefix(repo.work_dir().context("no work dir")?)?;
    let mut index = repo.index_mut()?;
    index.add_path(relative_path, None, gix::index::entry::Mode::FILE)?;
    index.write()?;
    Ok(())
}

fn modify_file(repo: &Repository, path: &Path, content: &str) -> Result<()> {
    let full_path = repo.work_dir().context("no work dir")?.join(path);
    std::fs::write(&full_path, content)?;
    Ok(())
}

fn get_status_entries(repo: &Repository) -> Result<Vec<StatusEntry>> {
    let mut status = Status::new(repo)?;
    let (mut entries, _) = status.workdir_untracked_and_others()?;
    entries.extend(status.staged_and_unstaged()?);
    Ok(entries)
}

fn main() -> Result<()> {
    let (_tmp_dir, repo) = create_test_repo()?;
    let work_dir = repo.work_dir().context("no work dir")?;

    // 1. Create file_a, file_b, file_c and commit them
    let file_a_path = work_dir.join("file_a.txt");
    let file_b_path = work_dir.join("file_b.txt");
    let file_c_path = work_dir.join("file_c.txt");

    create_file_and_add(&repo, &file_a_path, "content A v1")?;
    create_file_and_add(&repo, &file_b_path, "content B v1")?;
    create_file_and_add(&repo, &file_c_path, "content C v1")?;

    let tree_id = repo.index()?.write_tree()?;
    let sig = gix::actor::Signature::new("Test User", "test@example.com", &gix::date::Time::now_utc())?;
    repo.commit(sig, sig, "Initial commit", tree_id, None)?;

    println!("--- Initial commit ---");
    for entry in get_status_entries(&repo)? {
        println!("{:?}", entry);
    }

    // 2. Modify file_a, file_b, file_c
    modify_file(&repo, &file_a_path, "content A v2")?;
    modify_file(&repo, &file_b_path, "content B v2")?;
    modify_file(&repo, &file_c_path, "content C v2")?;

    println!("
--- After modifying all files ---");
    for entry in get_status_entries(&repo)? {
        println!("{:?}", entry);
    }

    // 3. Stage file_c (simulating user staging it)
    let mut index = repo.index_mut()?;
    index.add_path(
        file_c_path.strip_prefix(work_dir)?,
        None,
        gix::index::entry::Mode::FILE,
    )?;
    index.write()?;

    println!("
--- After staging file_c ---");
    for entry in get_status_entries(&repo)? {
        println!("{:?}", entry);
    }

    // 4. Agent's turn: Unstage file_c, stage file_a and file_b, commit
    println!("
--- Agent's operations ---");

    // Unstage file_c
    let mut index = repo.index_mut()?;
    index.remove_path(file_c_path.strip_prefix(work_dir)?)?;
    index.write()?;
    println!("Unstaged file_c.txt");

    // Stage file_a and file_b
    index.add_path(
        file_a_path.strip_prefix(work_dir)?,
        None,
        gix::index::entry::Mode::FILE,
    )?;
    index.add_path(
        file_b_path.strip_prefix(work_dir)?,
        None,
        gix::index::entry::Mode::FILE,
    )?;
    index.write()?;
    println!("Staged file_a.txt and file_b.txt");

    // Commit changes for file_a and file_b
    let tree_id = repo.index()?.write_tree()?;
    let head_id = repo.head()?.try_into_id()?;
    let parent_commit = repo.find_commit(head_id)?;
    let sig = gix::actor::Signature::new("Agent User", "agent@example.com", &gix::date::Time::now_utc())?;
    repo.commit(sig, sig, "Agent commit: Modified A and B", tree_id, Some(&parent_commit.id()))?;
    println!("Committed changes for file_a.txt and file_b.txt");

    println!("
--- After agent's commit ---");
    for entry in get_status_entries(&repo)? {
        println!("{:?}", entry);
    }

    // 5. Agent re-stages file_c (if it was staged by user before)
    let mut index = repo.index_mut()?;
    index.add_path(
        file_c_path.strip_prefix(work_dir)?,
        None,
        gix::index::entry::Mode::FILE,
    )?;
    index.write()?;
    println!("Re-staged file_c.txt");

    println!("
--- After re-staging file_c ---");
    for entry in get_status_entries(&repo)? {
        println!("{:?}", entry);
    }

    Ok(())
}
