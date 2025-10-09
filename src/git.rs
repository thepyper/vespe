use std::path::PathBuf;
use anyhow::Result;
use gix::bstr::BString;

fn git_commit(files_to_commit: &[PathBuf], message: &str, comment: &str) -> Result<()> {
    // Apri il repository nella directory corrente
    let repo = gix::open(".")?;
    
    // Ottieni l'index modificabile attraverso il worktree
    let mut worktree = repo.worktree()?;
    let index = worktree.index()?;
    
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
    let mut index = worktree.index()?.clone();
    
    // Aggiungi i files_to_commit allo staging (se non già presenti)
    for file in files_to_commit {
        if !files_in_staging.contains(file) {
            // Usa gix_worktree per aggiungere il file
            let abs_path = repo.work_dir().unwrap().join(file);
            if abs_path.exists() {
                let entry = gix::index::entry::Entry::from_path(
                    &abs_path,
                    file,
                    &odb,
                )?;
                index.add(entry);
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
            if let Some(idx) = index.entry_index_by_path(&path_bstr) {
                index.remove_entry(idx);
            }
        }
    }
    
    // Scrivi l'index
    let index_path = repo.index_path();
    let mut file = std::fs::File::create(&index_path)?;
    index.write_to(&mut file, gix::index::write::Options::default())?;
    
    // 4) Crea il tree dall'index
    let tree_id = index.write_tree_to(&odb)?;
    
    // Costruisci il messaggio di commit completo
    let full_message = if comment.is_empty() {
        message.to_string()
    } else {
        format!("{}\n\n{}", message, comment)
    };
    
    // Ottieni la signature
    let signature = repo.committer().unwrap_or_else(|| {
        gix::actor::Signature {
            name: "Default User".into(),
            email: "user@example.com".into(),
            time: gix::date::Time::now_local_or_utc(),
        }
    });
    
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
    let mut index = gix::index::File::at(
        index_path.clone(),
        repo.object_hash(),
        false,
        gix::index::decode::Options::default()
    )?;
    
    for file in removed_files {
        let abs_path = repo.work_dir().unwrap().join(&file);
        if abs_path.exists() {
            let entry = gix::index::entry::Entry::from_path(
                &abs_path,
                &file,
                &odb,
            )?;
            index.add(entry);
        }
    }
    
    let mut file = std::fs::File::create(&index_path)?;
    index.write_to(&mut file, gix::index::write::Options::default())?;
    
    Ok(())
}