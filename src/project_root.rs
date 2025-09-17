use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use std::fs;

const VESPE_DIR: &str = ".vespe";
const VESPE_ROOT_MARKER: &str = ".vespe_root";

pub fn is_project_root(dir: &Path) -> bool {
    return dir.join(VESPE_DIR).join(VESPE_ROOT_MARKER).exists();    
}

pub fn find_project_root(start_dir: &Path) -> Option<PathBuf> {
    let mut current_dir = Some(start_dir);

    while let Some(dir) = current_dir {
        
        if is_project_root(dir) {
            return Some(dir.to_path_buf());
        }
        
        current_dir = dir.parent();
    }

    None
}

pub fn initialize_project_root(target_dir: &Path) -> Result<()> {
    let absolute_target_dir = target_dir.canonicalize().map_err(|e| anyhow!("Failed to canonicalize target directory {}: {}", target_dir.display(), e))?;

    // Check if target_dir is already part of an existing Vespe project
    if let Some(found_root) = find_project_root(&absolute_target_dir) {
        return Err(anyhow!("Cannot initialize a Vespe project inside an existing project. Existing root: {}", found_root.display()));
    }

    let vespe_dir = absolute_target_dir.join(VESPE_DIR);
    let vespe_root_marker = vespe_dir.join(VESPE_ROOT_MARKER);

    fs::create_dir_all(&vespe_dir)?;

    fs::write(&vespe_root_marker, "Feel The BuZZ!!!!")?;

    let vespe_gitignore = vespe_dir.join(".gitignore");
    fs::write(&vespe_gitignore, "log/")?;

    Ok(())
}
