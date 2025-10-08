use std::path::PathBuf;
use anyhow::{Context, Result, anyhow};
use gix::{
    prelude::*,
    head::Kind as HeadKind,
    worktree::IndexPersistedOrInMemory,
};

pub fn git_commit(files_to_commit: &[PathBuf], message: &str, comment: &str) -> Result<()> {
    // Scopri il repository (cerca da cartella corrente verso sopra)
    let repo = gix::discover(".").context("Failed to discover git repository")?;
    let head = repo.head().context("Failed to get HEAD")?;

    // Apri il worktree per manipolare l’area di lavoro
    let mut worktree = repo.open_worktree()
        .context("Failed to open worktree")?;

    // Carica l’indice (potrebbe essere persistente o in memoria)
    let mut index: IndexPersistedOrInMemory = worktree.load_index()
        .context("Failed to load worktree index")?;

    // Determina i genitori del commit (se non è un repository “unborn”)
    let parent_ids = match head.kind {
        HeadKind::Symbolic(_) | HeadKind::Detached { .. } => {
            let commit = head.peel_to_commit_in_place()
                .context("Failed to peel HEAD to commit")?;
            vec![commit.id()]
        }
        HeadKind::Unborn(_) => Vec::new(),
    };

    // Memoriza i file già stagiati (stage == 0)
    let initially_staged: Vec<PathBuf> = index
        .entries()
        .filter_map(|e| {
            if e.stage() == 0 {
                Some(e.path.to_owned().into())
            } else {
                None
            }
        })
        .collect();

    // Aggiungi i file che vogliamo includere nel commit
    for file in files_to_commit {
        index.add_path(file)
            .with_context(|| format!("Failed to add path: {:?}", file))?;
    }

    // Rimuovi quelli che non vogliamo includere
    let mut to_restage: Vec<PathBuf> = Vec::new();
    for path in initially_staged {
        if !files_to_commit.contains(&path) {
            index.remove_path(&path)
                .with_context(|| format!("Failed to remove path: {:?}", path))?;
            to_restage.push(path);
        }
    }

    // Scrivi l’indice
    index.write()
        .context("Failed to write index")?;

    // Scrivi l’albero
    let tree_id = index.write_tree()
        .context("Failed to write tree")?;

    // Ottieni autore / committer
    let author = repo.author()?.ok_or_else(|| anyhow!("Missing author signature"))?;
    let committer = repo.committer()?.ok_or_else(|| anyhow!("Missing committer signature"))?;
    let commit_message = format!("{}\n\n{}", message, comment);

    // Crea il commit
    let commit_id = repo.commit(
        &author,
        &committer,
        commit_message.as_str(),
        tree_id,
        parent_ids.iter(),
    ).context("Failed to create commit")?;

    // Aggiorna HEAD / reference
    match head.kind {
        HeadKind::Symbolic(_) => {
            let head_name = head.name().ok_or_else(|| anyhow!("HEAD has no name"))?;
            let mut head_ref = repo.find_reference(head_name)
                .with_context(|| format!("Failed to find reference {}", head_name))?;
            head_ref.set_target_id(commit_id, "commit")
                .context("Failed to update symbolic HEAD ref")?;
        }
        HeadKind::Detached { .. } => {
            // Metodo per HEAD detached — in gix 0.73 dovrebbe esserci qualcosa del genere
            repo.set_head_detached(commit_id)
                .context("Failed to set detached HEAD")?;
        }
        HeadKind::Unborn(_) => {
            repo.references()?
                .create_branch("main", commit_id, true, "initial commit")
                .context("Failed to create initial branch")?;
        }
    }

    // Ri-aggiungi i file che avevamo rimosso
    for path in to_restage {
        index.add_path(&path)
            .with_context(|| format!("Failed to re-add path: {:?}", path))?;
    }
    index.write()
        .context("Failed to write index after re-staging")?;

    Ok(())
}
