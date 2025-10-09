use git2::{IndexAddOption, Repository, Signature};
use std::path::PathBuf;
use anyhow::Result;

pub fn git_commit(files_to_commit: &[PathBuf], message: &str, comment: &str) -> Result<()> {
    // Apri il repository corrente
    let repo = Repository::open(".")?;

    // Ottieni l'index (staging area)
    let mut index = repo.index()?;

    // Svuota lo stage per sicurezza
    index.clear()?;

    // Aggiungi solo i file specificati
    for path in files_to_commit {
        index.add_path(path)?;
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
    let sig = Signature::now(comment, "example@example.com")?;

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
