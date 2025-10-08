use std::path::PathBuf;
use anyhow::{Context, Result, anyhow};
use gix::{
    prelude::*,
    head::Kind as HeadKind,
    worktree::index::File,
};

pub fn git_commit(files_to_commit: &[PathBuf], message: &str, comment: &str) -> Result<()> {
    // Scopri il repository (cerca da cartella corrente verso sopra)
    let repo = gix::discover(".").context("Failed to discover git repository")?;
    let head = repo.head().context("Failed to get HEAD")?;

    // Apri il worktree
    let worktree = repo.worktree().context("Failed to open worktree")?;

    // Carica l'indice
    let mut index = worktree.index().context("Failed to load worktree index")?;

    // Determina i genitori del commit
    let parent_ids = match head.kind {
        HeadKind::Symbolic(_) | HeadKind::Detached { .. } => {
            let commit = head.try_into_fully_peeled_id()
                .context("Failed to peel HEAD to commit")?
                .object()?
                .into_commit();
            vec![commit.id]
        }
        HeadKind::Unborn(_) => Vec::new(),
    };

    // Memorizza i file già stagiati (stage == 0)
    let initially_staged: Vec<PathBuf> = index
        .entries()
        .filter(|e| e.stage() == 0)
        .map(|e| e.path_in(&worktree).to_path_buf())
        .collect();

    // Aggiungi i file che vogliamo includere nel commit
    for file in files_to_commit {
        worktree.add_path_to_index(&mut index, file)
            .with_context(|| format!("Failed to add path: {:?}", file))?;
    }

    // Rimuovi quelli che non vogliamo includere
    let mut to_restage: Vec<PathBuf> = Vec::new();
    for path in &initially_staged {
        if !files_to_commit.contains(path) {
            worktree.remove_path_from_index(&mut index, path)
                .with_context(|| format!("Failed to remove path: {:?}", path))?;
            to_restage.push(path.clone());
        }
    }

    // Scrivi l'indice
    index.write().context("Failed to write index")?;

    // Scrivi l'albero
    let tree_id = index.write_tree().context("Failed to write tree")?;

    // Ottieni autore / committer
    let config = repo.config_snapshot();
    let author = gix::actor::Signature::now("User", "user@example.com")
        .context("Failed to create author signature")?;
    let committer = author.clone();

    let commit_message = format!("{}\n\n{}", message, comment);

    // Crea il commit
    let commit_id = repo.commit(
        &head,
        commit_message.as_str(),
        tree_id,
        parent_ids.iter().map(|id| *id),
    ).context("Failed to create commit")?;

    // Aggiorna HEAD / reference
    match head.kind {
        HeadKind::Symbolic(_) => {
            // In gix 0.73, il commit già aggiorna HEAD automaticamente
            // Non è necessario fare nulla qui
        }
        HeadKind::Detached { .. } => {
            // Anche per detached HEAD, il commit già gestisce l'aggiornamento
        }
        HeadKind::Unborn(_) => {
            // Per repository unborn, crea il branch main
            repo.reference(
                "refs/heads/main",
                commit_id,
                gix::refs::transaction::PreviousValue::MustNotExist,
                "initial commit"
            ).context("Failed to create initial branch")?;
        }
    }

    // Ri-aggiungi i file che avevamo rimosso per preservare lo staging precedente
    for path in to_restage {
        worktree.add_path_to_index(&mut index, &path)
            .with_context(|| format!("Failed to re-add path: {:?}", path))?;
    }
    index.write().context("Failed to write index after re-staging")?;

    Ok(())
}