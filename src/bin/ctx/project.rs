
use anyhow::{Context as AnyhowContext, Result};
use std::path::{Path, PathBuf};

use super::context::Context;

pub struct Project {
    pub root_path: PathBuf,
}

impl Project {
    pub fn contexts_dir(&self) -> Result<PathBuf> {
        let path = self.root_path.join("contexts");
        std::fs::create_dir_all(&path)?;
        Ok(path)
    }

    pub fn list_contexts(&self) -> Result<()> {
        let contexts_path = self.contexts_dir()?;
    
        let mut found_one = false;
        for entry in std::fs::read_dir(contexts_path)? {
            let entry = entry?;
            if entry.path().extension() == Some("md".as_ref()) {
                let name = entry.file_name().to_string_lossy().to_string();
                println!("{}", Context::to_name(&name));
                found_one = true;
            }
        }
    
        if !found_one {
            println!("No contexts found.");
        }
    
        Ok(())
    }

     pub fn new_context(&self, name: &str) -> Result<()> {
        let path = self.contexts_dir()?.join(Context::to_filename(name));
        
        if path.exists() {
            anyhow::bail!("Context '{}' already exists", name);
        }
        
        std::fs::write(&path, format!("# {}\n\n", name))?;
        println!("Created {}", path.display());
        Ok(())
    }

    pub fn resolve_context(&self, name: &str) -> Result<PathBuf> {
        let path = self.contexts_dir()?.join(Context::to_filename(name));
        if !path.is_file() {
            anyhow::bail!("Context '{}' does not exist", name);
        }
        Ok(path)
    }

    pub fn init(path: &Path) -> Result<Project> {
        let ctx_dir = path.join(".ctx");
        if ctx_dir.is_dir() && ctx_dir.join(".ctx_root").is_file() {
            anyhow::bail!("ctx project already initialized in this directory.");
        }

        std::fs::create_dir_all(&ctx_dir).context("Failed to create .ctx directory")?;

        let ctx_root_file = ctx_dir.join(".ctx_root");
        std::fs::write(&ctx_root_file, "Feel The BuZZ!!")
            .context("Failed to write .ctx_root file")?;

        Ok(Project {
            root_path: ctx_dir.canonicalize()?,
        })
    }

    pub fn find(path: &Path) -> Result<Project> {
        let mut current_path = path.to_path_buf();

        loop {
            let ctx_dir = current_path.join(".ctx");
            if ctx_dir.is_dir() && ctx_dir.join(".ctx_root").is_file() {
                return Ok(Project {
                    root_path: ctx_dir.canonicalize()?,
                });
            }

            if !current_path.pop() {
                break;
            }
        }

        anyhow::bail!("No .ctx project found in the current directory or any parent directory.")
    }
}
