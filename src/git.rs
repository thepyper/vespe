use std::path::PathBuf;
use anyhow::{Context, Result, anyhow};
use gix::{
    actor::Signature,
    index::{
        self,
        entry::Stage,
        State,
    },
    reference::{
        self,
        Kind as HeadKind,
    },
    Repository,
};

pub fn git_commit(files_to_commit: &[PathBuf], message: &str, comment: &str) -> Result<()> {
    // Scopri il repository (cerca da cartella corrente verso sopra)
    let repo = gix::discover(".").context("Failed to discover git repository")?;
    let head = repo.head().context("Failed to get HEAD")?;

    // Apri il worktree per manipolare l’area di lavoro
    let mut worktree = repo.worktree()
        .context("Failed to open worktree")?;

    // Carica l’indice (potrebbe essere persistente o in memoria)
    let mut index = worktree.index()
        .context("Failed to load worktree index")?;

    // Determina i genitori del commit (se non è un repository “unborn”)
    let (parent_ids, initially_staged_paths) = match repo.head()?.kind {
        HeadKind::Symbolic => {
            let head_commit = repo.head()?.peel_to_commit_in_place()?.id;
            (vec![head_commit], Vec::new())
        },
        HeadKind::Detached => {
            let head_commit = repo.head()?.peel_to_commit_in_place()?.id;
            (vec![head_commit], Vec::new())
        },
    };

    let mut index = repo.index_or_load_from_head()?;

    let initially_staged_paths: Vec<PathBuf> = if !repo.head().is_unborn() {
        index.entries().iter().filter_map(|e| {
            if e.stage() == Stage::Unconflicted  {
                Some(e.path(&(*index).state()).to_path().to_owned())
            } else {
                None
            }
        }).collect()
    } else {
        Vec::new()
    };

    // Memoriza i file già stagiati (stage == 0)
    let initially_staged: Vec<PathBuf> = index
        .entries()
        .into_iter()
        .filter_map(|e| {
            if e.stage() == Stage::Unconflicted  {
                Some(e.path(&index.state()).to_path_buf())
            } else {
                None
            }
        })
        .collect();

    // Aggiungi i file che vogliamo includere nel commit
    for file in files_to_commit {
        (*index).state_mut().add(file).with_context(|| format!("Failed to add path: {:?}", file))?;
    }

    // Rimuovi quelli che non vogliamo includere
    let mut to_restage: Vec<PathBuf> = Vec::new();
    for path in initially_staged {
        if !files_to_commit.contains(&path) {
            (*index).state_mut().remove(&path).with_context(|| format!("Failed to remove path: {:?}", path))?;
            to_restage.push(path);
        }
    }

    // Scrivi l’indice
    index.write(gix::index::write::Options::default())?;

    let tree_id = (*index).state_mut().write_tree()?;

    // Ottieni autore / committer
    let author = repo.author().ok_or_else(|| anyhow!("Missing author signature"))?;
    let committer = repo.committer().ok_or_else(|| anyhow!("Missing committer signature"))?;
    let commit_message = format!("{}\n\n{}", message, comment);

    // Crea il commit
    let commit_id = repo.commit(
        &author,
        &committer,
        commit_message.as_str(),
        tree_id,
        parent_ids.iter(),
    )?;

    // Aggiorna HEAD / reference
    match head.kind {
        HeadKind::Symbolic(_) => {
            let head_name = head.name().ok_or_else(|| anyhow!("HEAD has no name"))?.to_owned();
            let mut head_ref = repo.find_reference(head_name)
                .with_context(|| format!("Failed to find reference {}", head_name))?;
            head_ref.set_target_id(commit_id, "commit")
                .context("Failed to update symbolic HEAD ref")?;
        }
        HeadKind::Detached { .. } => {
            // Metodo per HEAD detached — in gix 0.73 dovrebbe esserci qualcosa del genere
            repo.set_head(commit_id).context("Failed to set detached HEAD")?;
        }
        HeadKind::Unborn(_) => {
            repo.head_update().create_new_branch("main", commit_id).context("Failed to create initial branch")?;
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
