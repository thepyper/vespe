use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use std::fs;

const VESPE_DIR: &str = ".vespe";
const VESPE_ROOT_MARKER: &str = ".vespe_root";

pub fn find_project_root(start_dir: &Path) -> Option<PathBuf> {
    let mut current_dir = Some(start_dir);

    while let Some(dir) = current_dir {
        let vespe_dir = dir.join(VESPE_DIR);
        let vespe_root_marker = vespe_dir.join(VESPE_ROOT_MARKER);

        if vespe_root_marker.exists() {
            return Some(dir.to_path_buf());
        }

        current_dir = dir.parent();
    }

    None
}

pub fn initialize_project_root(target_dir: &Path, current_project_root: Option<&Path>) -> Result<()> {
    let absolute_target_dir = target_dir.canonicalize().map_err(|e| anyhow!("Failed to canonicalize target directory {}: {}", target_dir.display(), e))?;

    println!("initialize_project_root: absolute_target_dir = {}", absolute_target_dir.display());
    if let Some(root) = current_project_root {
        println!("initialize_project_root: current_project_root = {}", root.display());
    } else {
        use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use std::fs;

const VESPE_DIR: &str = ".vespe";
const VESPE_ROOT_MARKER: &str = ".vespe_root";

pub fn find_project_root(start_dir: &Path) -> Option<PathBuf> {
    let mut current_dir = Some(start_dir);

    while let Some(dir) = current_dir {
        let vespe_dir = dir.join(VESPE_DIR);
        let vespe_root_marker = vespe_dir.join(VESPE_ROOT_MARKER);

        if vespe_root_marker.exists() {
            return Some(dir.to_path_buf());
        }

        current_dir = dir.parent();
    }

    None
}

pub fn initialize_project_root(target_dir: &Path, current_project_root: Option<&Path>) -> Result<()> {
    let absolute_target_dir = target_dir.canonicalize().map_err(|e| anyhow!("Failed to canonicalize target directory {}: {}", target_dir.display(), e))?;

    let vespe_dir = absolute_target_dir.join(VESPE_DIR);
    let vespe_root_marker = vespe_dir.join(VESPE_ROOT_MARKER);

    // First, check if the target_dir itself is already initialized
    if vespe_root_marker.exists() {
        return Err(anyhow!("Project already initialized at: {}", absolute_target_dir.display()));
    }

    // Check if target_dir is already part of an existing Vespe project (excluding the current running project)
    if let Some(found_root) = find_project_root(&absolute_target_dir) {
        let mut is_current_project_root = false;
        if let Some(current_root_path) = current_project_root {
            let canonical_found_root = found_root.canonicalize().unwrap_or_else(|_| found_root.clone());
            let canonical_current_root = current_root_path.canonicalize().unwrap_or_else(|_| current_root_path.to_path_buf());
            if canonical_found_root == canonical_current_root {
                is_current_project_root = true;
            }
        }

        if !is_current_project_root {
            return Err(anyhow!("Cannot initialize a Vespe project inside an existing project. Existing root: {}", found_root.display()));
        }
    }

    fs::create_dir_all(&vespe_dir)?;

    fs::write(&vespe_root_marker, "Feel The BuZZ!!!!")?;

    let vespe_gitignore = vespe_dir.join(".gitignore");
    fs::write(&vespe_gitignore, "log/")?;

    Ok(())
}
    }

    let vespe_dir = absolute_target_dir.join(VESPE_DIR);
    let vespe_root_marker = vespe_dir.join(VESPE_ROOT_MARKER);

    // First, check if the target_dir itself is already initialized
    if vespe_root_marker.exists() {
        return Err(anyhow!("Project already initialized at: {}", absolute_target_dir.display()));
    }

    // Then, check if target_dir is a subdirectory of an existing Vespe project
    let mut current_parent = absolute_target_dir.parent();
    while let Some(parent) = current_parent {
        println!("  Checking parent: {}", parent.display());
        let parent_vespe_dir = parent.join(VESPE_DIR);
        let parent_vespe_root_marker = parent_vespe_dir.join(VESPE_ROOT_MARKER);
        if parent_vespe_root_marker.exists() {
            println!("    Found existing root marker in parent: {}", parent.display());
            // If an existing root is found, check if it's the current project's root
            if let Some(current_root_path) = current_project_root {
                let canonical_parent = parent.canonicalize().unwrap_or_else(|_| parent.to_path_buf());
                let canonical_current_root = current_root_path.canonicalize().unwrap_or_else(|_| current_root_path.to_path_buf());
                println!("      Comparing canonical_parent ({}) with canonical_current_root ({})", canonical_parent.display(), canonical_current_root.display());
                if canonical_parent == canonical_current_root {
                    // This is the current project's root, so it's okay
                    println!("      Parent is current_root. Continuing upwards.");
                    current_parent = parent.parent();
                    continue;
                }
            }
            println!("    Returning error: Nested project detected. Existing root: {}", parent.display());
            return Err(anyhow!("Cannot initialize a Vespe project inside an existing project. Existing root: {}", parent.display()));
        }
        current_parent = parent.parent();
    }

    fs::create_dir_all(&vespe_dir)?;

    fs::write(&vespe_root_marker, "Feel The BuZZ!!!!")?;

    let vespe_gitignore = vespe_dir.join(".gitignore");
    fs::write(&vespe_gitignore, "log/")?;

    Ok(())
}
