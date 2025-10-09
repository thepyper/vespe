use git2::{Repository, Signature, Index, Oid, Tree};
use std::path::{Path, PathBuf};
use tracing::debug;
use anyhow::Context;

/// Esegue un commit selettivo, includendo solo i file specificati,
/// e ripristina l'area di staging al suo stato originale dopo il commit.
///
/// Questo processo garantisce che l'area di staging venga prima allineata con l'HEAD
/// (rimuovendo tutte le modifiche precedentemente stagionate) e poi vengano aggiunti
/// solo i file specificati in `files_to_commit`. Le modifiche non stagionate nella
/// directory di lavoro rimarranno intatte. Dopo il commit, l'indice viene ripristinato
/// allo stato in cui si trovava prima dell'operazione.
///
/// # Argomenti
/// * `repo_path` - Il percorso al repository Git.
/// * `files_to_commit` - Un vettore di stringhe, dove ogni stringa è il percorso di un file
///                       da includere nel commit, relativo alla root del repository.
/// * `commit_message` - Il messaggio del commit.
/// * `author_name` - Il nome dell'autore del commit.
/// * `author_email` - L'email dell'autore del commit.
///
/// # Restituisce
/// Un `Result` che indica successo (`Ok(Oid)`) o fallimento (`Err(String)`).
pub fn git_commit(
    repo_path: &Path,
    files_to_commit: &[PathBuf],
    commit_message: &str,
) -> Result<Oid, String> {
    let repo = Repository::open(repo_path)
        .map_err(|e| format!("Impossibile aprire il repository: {}", e))?;

    let workdir = repo.workdir().context("Repository has no workdir")?;

    // 1. Ottieni il commit HEAD corrente. Questo sarà il genitore del nuovo commit
    // e la base per "pulire" l'area di staging.
    let head_commit = repo.head()
        .and_then(|head| head.peel_to_commit())
        .map_err(|e| format!("Impossibile ottenere il commit HEAD: {}", e))?;

    // 2. Ottieni l'albero (tree) associato al commit HEAD.
    let head_tree = head_commit.tree()
        .map_err(|e| format!("Impossibile ottenere l'albero dal commit HEAD: {}", e))?;

    // 3. Carica l'indice del repository.
    let mut index = repo.index()
        .map_err(|e| format!("Impossibile ottenere l'indice del repository: {}", e))?;

    // 4. *** SALVA LO STATO ATTUALE DELL'INDICE ***
    // Creiamo un tree object temporaneo dall'indice corrente. Questo OID rappresenta
    // lo stato dell'indice prima delle nostre modifiche.
    let original_index_tree_oid = index.write_tree()
        .map_err(|e| format!("Impossibile salvare lo stato originale dell'indice: {}", e))?;

    // 5. Aggiorna l'indice per farlo corrispondere all'albero HEAD.
    // Questa è l'operazione chiave: simula 'git reset --mixed HEAD' o 'git restore --staged .'.
    // Rimuove tutte le modifiche precedentemente stagionate dall'indice,
    // ma *non tocca i file nella directory di lavoro*.
    index.read_tree(&head_tree)
        .map_err(|e| format!("Impossibile leggere l'albero HEAD nell'indice: {}", e))?;

    // 6. Aggiungi selettivamente i files_to_commit all'area di staging (indice).
    for file_path in files_to_commit {
        //let file_path = Path::new(file_path_str);

        let canonical_path = file_path.canonicalize().with_context(|| format!("Failed to canonicalize path: {}", path.display()))?;
        let canonical_workdir = workdir.canonicalize().with_context(|| format!("Failed to canonicalize workdir: {}", workdir.display()))?;

        let relative_path = canonical_path.strip_prefix(&canonical_workdir).with_context(|| {
            format!(
                "File {} is outside the repository workdir at {}",
                path.display(),
                workdir.display()
            )
        })?;

        index.add_path(file_path)
            .map_err(|e| format!("Impossibile aggiungere il file '{}' all'indice: {}", file_path_str, e))?;
    }

    // 7. Scrivi le modifiche all'indice su disco.
    index.write()
        .map_err(|e| format!("Impossibile scrivere l'indice: {}", e))?;

    // 8. Crea un nuovo albero (tree) basato sullo stato attuale dell'indice.
    let tree_oid = index.write_tree()
        .map_err(|e| format!("Impossibile scrivere l'albero dall'indice: {}", e))?;
    let tree = repo.find_tree(tree_oid)
        .map_err(|e| format!("Impossibile trovare l'albero: {}", e))?;

    // 9. Crea la firma dell'autore e del committer.
    let signature = Signature::now("vespe", "")
        .map_err(|e| format!("Impossibile creare la firma: {}", e))?;

    // 10. Crea il nuovo commit.
    let new_commit_oid = repo.commit(
        Some("HEAD"), // Aggiorna HEAD per puntare al nuovo commit
        &signature,
        &signature,
        commit_message,
        &tree,
        &[&head_commit], // Il nuovo commit ha l'HEAD corrente come genitore
    )
    .map_err(|e| format!("Impossibile creare il commit: {}", e))?;

    // 11. *** RIPRISTINA LO STATO ORIGINALE DELL'INDICE ***
    // Trova l'oggetto tree corrispondente all'OID salvato.
    let original_index_tree = repo.find_tree(original_index_tree_oid)
        .map_err(|e| format!("Impossibile trovare l'albero originale dell'indice: {}", e))?;

    // Carica l'albero originale nell'indice.
    index.read_tree(&original_index_tree)
        .map_err(|e| format!("Impossibile ripristinare l'indice allo stato originale: {}", e))?;

    // Scrivi l'indice ripristinato su disco.
    index.write()
        .map_err(|e| format!("Impossibile scrivere l'indice ripristinato: {}", e))?;

    Ok(new_commit_oid)
}