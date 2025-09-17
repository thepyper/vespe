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
    let vespe_dir = target_dir.join(VESPE_DIR);
    fs::create_dir_all(&vespe_dir)?;

    let vespe_root_marker = vespe_dir.join(VESPE_ROOT_MARKER);
    fs::write(&vespe_root_marker, "Feel The BuZZ!!!!")?;

    Ok(())
}
