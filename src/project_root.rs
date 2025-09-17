use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use std::fs;

const VESPE_DIR: &str = ".vespe";
const VESPE_ROOT_MARKER: &str = ".vespe_root";

pub fn find_project_root(start_dir: &Path) -> Result<PathBuf> {
    let mut current_dir = Some(start_dir);

    while let Some(dir) = current_dir {
        let vespe_dir = dir.join(VESPE_DIR);
        let vespe_root_marker = vespe_dir.join(VESPE_ROOT_MARKER);

        if vespe_root_marker.exists() {
            return Ok(dir.to_path_buf());
        }

        current_dir = dir.parent();
    }

    Err(anyhow!("Project root not found starting from: {}", start_dir.display()))
}

pub fn initialize_project_root(target_dir: &Path) -> Result<()> {
    let absolute_target_dir = target_dir.canonicalize().map_err(|e| anyhow!("Failed to canonicalize target directory {}: {}", target_dir.display(), e))?;

    // Check if target_dir is already inside an existing Vespe project
    if let Ok(existing_root) = find_project_root(&absolute_target_dir) {
        return Err(anyhow!("Cannot initialize a Vespe project inside an existing project. Existing root: {}", existing_root.display()));
    }

    let vespe_dir = absolute_target_dir.join(VESPE_DIR);
    let vespe_root_marker = vespe_dir.join(VESPE_ROOT_MARKER);

    if vespe_root_marker.exists() {
        return Err(anyhow!("Project already initialized at: {}", absolute_target_dir.display()));
    }

    fs::create_dir_all(&vespe_dir)?;

    fs::write(&vespe_root_marker, "Feel The BuZZ!!!!")?;

    let vespe_gitignore = vespe_dir.join(".gitignore");
    fs::write(&vespe_gitignore, "log/")?;

    Ok(())
}
