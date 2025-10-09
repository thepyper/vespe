use git2::{Repository, Signature};
use std::path::PathBuf;
use anyhow::{Context, Result};

pub fn git_commit(files_to_commit: &[PathBuf], message: &str, comment: &str) -> Result<()> {
    // Apri il repository corrente
    let repo = Repository::open(".").context("Failed to open repository")?;
    let workdir = repo.workdir().context("Repository has no workdir")?;

    // Ottieni l'index (staging area)
    let mut index = repo.index()?;
    
    // Aggiungi solo i file specificati
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
        index.add_path(relative_path)?;
    }

    // Scrivi l'index su disco e ottieni l'albero
    index.write()?;
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    // Ottieni il commit HEAD corrente (se esiste)
    let parent_commit = repo.head()
        .ok()
        .and_then(|h| h.target())
        .and_then(|oid| repo.find_commit(oid).ok());

    // Crea la firma (autore + committer)
    let sig = Signature::now(comment, "vespe")?;

    // Crea il commit
    let commit_id = match parent_commit {
        Some(parent) => repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            message,
            &tree,
            &[&parent],
        )?,
        None => repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            message,
            &tree,
            &[],
        )?,
    };

    println!("Commit creato con id {}", commit_id);
    Ok(())
}
