use std::path::PathBuf;
use anyhow::Result;
use gix::prelude::*;
use gix::index::State as IndexState;
use gix::object::write::Tree;
use gix::object::WriteTo;

pub fn git_commit(files_to_commit: &[PathBuf], message: &str, comment: &str) -> Result<()> {
    // Apri il repo dalla directory corrente (o ascendi finché trovi .git)
    let mut repo = gix::open(Default::default())?;
    // Ottieni l’index mutabile
    let mut index = repo.index_mut()?;
    // Assicurati che l’index rifletta lo stato attuale del worktree
    index.update_working_tree(None, None)?;
    // Ora staggiamo *solo* i file voluti
    for path in files_to_commit {
        index.add_path(path)?;
    }
    // Scrivi l’index su disco
    index.write()?;
    // Prepara il “tree” per il commit
    let tree_id = {
        let snapshot = index.snapshot()?;
        // Questo crea un oggetto tree nel database oggetti
        snapshot.write_tree(&mut repo.objects, Default::default())?
    };
    // Trova il commit padre (HEAD se esiste)
    let parent = repo.head()?.target().and_then(|oid| repo.find_commit(oid).ok());
    // Prepara l’autore / committer (usa configurazione del repo)
    let signature = repo.committer()?.to_owned();
    // Costruisci il commit (metti messaggio + commento come vuoi)
    let full_message = if comment.is_empty() {
        message.to_string()
    } else {
        format!("{}\n\n{}", message, comment)
    };
    let commit_id = repo.objects.write_commit(
        full_message.as_bytes(),
        tree_id,
        parent.as_ref().map(|c| c.id()),
        &signature,
        &signature,
    )?;
    // Aggiorna HEAD (o il ramo corrente)
    repo.refs().find("HEAD")?.into_referent().attach(&repo)?.reference()?.try_into().unwrap()
        .set_target(commit_id, "commit via gix")?;
    Ok(())
}
