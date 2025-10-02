use anyhow::Result;
use std::path::{Path, PathBuf};

use super::project::Project;

pub struct Context;

impl Context {
    pub fn new(project: &Project, name: &str) -> Result<()> {
        let path = to_path(project.contexts_dir()?, Self::to_filename(name));
        
        if path.exists() {
            anyhow::bail!("Context '{}' already exists", name);
        }
        
        std::fs::write(&path, format!("# {}\n\n", name))?;
        println!("Created {}", path.display());
        Ok(())
    }

    pub fn to_name(name: &str) -> String {
        name.strip_suffix(".md").unwrap_or(name).to_string()
    }

    pub fn to_filename(name: &str) -> String {
        if name.ends_with(".md") {
            name.to_string()
        } else {
            format!("{}.md", name)
        }
    }

    pub fn to_path(context_root: &Path, name: &str) -> PathBuf {        
        context_root.join(Self::to_filename(name))
    }
}
