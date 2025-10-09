use std::path::PathBuf;
use anyhow::Result;

fn git_commit(files_to_commit: &[PathBuf], message: &str, comment: &str) -> Result<()> {
    // Apri il repository nella directory corrente
    let repo = gix::open(".")?;
   
    // 1) Controlla lo stato corrente con git status
    let mut index = repo.index()?;
    let head = repo.head_commit()?;
   
    // Ottieni i files attualmente in staging
    let mut files_in_staging = Vec::new();
    for entry in index.entries() {
        let path = PathBuf::from(entry.path(&index));
        files_in_staging.push(path);
    }
   
    // 2) Aggiungi i files_to_commit allo staging (se non già presenti)
    for file in files_to_commit {
        if !files_in_staging.contains(file) {
            // Usa gix per aggiungere il file all'index
            index.add_path(file)?;
        }
    }
   
    // 3) Rimuovi dallo staging i files che non sono in files_to_commit
    let mut removed_files = Vec::new();
    for staged_file in &files_in_staging {
        if !files_to_commit.contains(staged_file) {
            // Ricorda il file per riaggiungerlo dopo
            removed_files.push(staged_file.clone());
           
            // Rimuovi dall'index (unstage)
            index.remove_path(staged_file)?;
        }
    }
   
    // Scrivi le modifiche all'index
    index.write(gix::index::write::Options::default())?;
   
    // 4) Esegui il commit con i files in staging
    let tree_id = index.write_tree()?;
   
    // Costruisci il messaggio di commit completo
    let full_message = if comment.is_empty() {
        message.to_string()
    } else {
        format!("{}\n\n{}", message, comment)
    };
   
    // Ottieni la signature dell'autore
    let signature = repo.committer().unwrap_or_else(|_| {
        gix::actor::SignatureRef {
            name: "Default User".into(),
            email: "user@example.com".into(),
            time: gix::date::Time::now_local_or_utc(),
        }.to_owned()
    });
    
    // Crea il commit
    let parent_id = head.id;
    
    // Prepara i riferimenti per il commit
    let reference = repo.find_reference("HEAD")?;
    let reference_name = reference.name().as_bstr();
    
    repo.commit(
        reference_name,
        full_message,
        tree_id,
        std::iter::once(parent_id),
    )?;
   
    // 5) Ri-aggiungi i files che erano stati rimossi dallo staging
    let mut index = repo.index()?;
    for file in removed_files {
        index.add_path(&file)?;
    }
    index.write(gix::index::write::Options::default())?;
   
    Ok(())
}