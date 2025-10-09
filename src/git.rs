use std::path::PathBuf;
use anyhow::Result;
use gix::bstr::{BString, ByteSlice};

fn git_commit(files_to_commit: &[PathBuf], message: &str, comment: &str) -> Result<()> {
    // Apri il repository nella directory corrente
    let repo = gix::open(".")?;
    
    let mut index = repo.index_mut()?;
    
    // 1) Ottieni i files attualmente in staging
    let mut files_in_staging = Vec::new();
    for entry in index.entries() {
        // Converti BStr in PathBuf correttamente
        let path_bytes = entry.path(&index);
        let path_str = std::str::from_utf8(path_bytes.as_ref())?;
        let path = PathBuf::from(path_str);
        files_in_staging.push(path);
    }
    
    // 2) Crea una nuova versione dell'index con le modifiche necessarie
    let odb = repo.objects.clone();
    
    // Aggiungi i files_to_commit allo staging (se non già presenti)
    for file in files_to_commit {
        if !files_in_staging.contains(file) {
            // Usa gix_worktree per aggiungere il file
            let abs_path = repo.workdir().ok_or_else(|| anyhow::anyhow!("No workdir"))?.join(file);
            if abs_path.exists() {
                let entry = gix::index::Entry::from_path(
                    &abs_path,
                    file,
                    &odb,
                )?;
                index.add_entry(entry)?;
            }
        }
    }
    
    // 3) Rimuovi dallo staging i files che non sono in files_to_commit
    let mut removed_files = Vec::new();
    for staged_file in &files_in_staging {
        if !files_to_commit.contains(staged_file) {
            removed_files.push(staged_file.clone());
            
            // Rimuovi dall'index
            let path_bstr = BString::from(staged_file.to_string_lossy().as_bytes());
            if let Ok(idx) = index.entry_index_by_path(path_bstr.as_bstr()) {
                index.remove_entry_at_index(idx);
            }
        }
    }
    
    // Scrivi l'index
    let index_path = repo.index_path();
    let mut file = std::fs::File::create(&index_path)?;
    index.write_to(&mut file, gix::index::write::Options::default())?;
    
    // 4) Crea il tree dall'index
    let tree_id = index.write_tree()?;
    
    // Costruisci il messaggio di commit completo
    let full_message = if comment.is_empty() {
        message.to_string()
    } else {
        format!("{}\n\n{}", message, comment)
    };
    
    // Ottieni il commit HEAD corrente
    let head = repo.head_commit()?;
    let parent_id = head.id;
    
    // Crea il commit
    let reference = repo.find_reference("HEAD")?;
    repo.commit(
        reference.name().as_bstr(),
        full_message,
        tree_id,
        [parent_id],
    )?;

    // 5) Ri-aggiungi i files rimossi
    for file in removed_files {
        let abs_path = repo.workdir().ok_or_else(|| anyhow::anyhow!("No workdir"))?.join(&file);
        if abs_path.exists() {
            let entry = gix::index::Entry::from_path(
                &abs_path,
                file,
                &odb,
            )?;
            index.add_entry(entry)?;
        }
    }
    
    index.write(gix::index::write::Options::default())?;
    
    Ok(())
}