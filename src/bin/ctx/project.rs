
use anyhow::{Context as AnyhowContext, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use super::context::{Context, ContextTreeItem, Line};

pub struct Project {
    pub root_path: PathBuf,
}

impl Project {
    pub fn compose(&self, name: &str) -> Result<String> {
        let path = self.resolve_context(name)?;
        let mut visited = HashSet::new();
        self.compose_recursive(&path, &mut visited)
    }

    fn compose_recursive(&self, path: &Path, visited: &mut HashSet<PathBuf>) -> Result<String> {
        if visited.contains(path) {
            return Ok(String::new()); // Circular include
        }
        visited.insert(path.to_path_buf());
        
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {:?}", path))?;
        
        let mut output = String::new();
        
        for line in Context::parse(&content) {
            match line {
                Line::Include { context_name } => {
                    let include_path = self.resolve_context(&context_name)?;
                    output.push_str(&self.compose_recursive(&include_path, visited)?);
                    output.push('\n');
                }
                Line::Text(text) => {
                    output.push_str(&text);
                    output.push('\n');
                }
                Line::Answer => {
                    // For now, do nothing
                }
            }
        }
        
        Ok(output)
    }

    pub fn context_tree(&self, name: &str) -> Result<ContextTreeItem> {
        let path = self.resolve_context(name)?;
        self.context_tree_recursive(&path, &mut HashSet::new())
    }

    fn context_tree_recursive(&self, path: &Path, visited: &mut HashSet<PathBuf>) -> Result<ContextTreeItem> {
        if visited.contains(path) {
            return Ok(ContextTreeItem::Leaf { name: Context::to_name(&path.file_name().unwrap().to_string_lossy()) });
        }
        visited.insert(path.to_path_buf());
    
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {:?}", path))?;
    
        let mut children = Vec::new();
        let current_name = Context::to_name(&path.file_name().unwrap().to_string_lossy());
    
        for line in Context::parse(&content) {
            if let Line::Include { context_name } = line {
                let include_path = self.resolve_context(&context_name)?;
                children.push(self.context_tree_recursive(&include_path, visited)?);
            }
        }
    
        if children.is_empty() {
            Ok(ContextTreeItem::Leaf { name: current_name })
        } else {
            Ok(ContextTreeItem::Node { name: current_name, children })
        }
    }

    pub fn contexts_dir(&self) -> Result<PathBuf> {
        let path = self.root_path.join("contexts");
        std::fs::create_dir_all(&path)?;
        Ok(path)
    }

    pub fn list_contexts(&self) -> Result<Vec<String>> {
        let contexts_path = self.contexts_dir()?;
        let mut context_names = Vec::new();
    
        for entry in std::fs::read_dir(contexts_path)? {
            let entry = entry?;
            if entry.path().extension() == Some("md".as_ref()) {
                let name = entry.file_name().to_string_lossy().to_string();
                context_names.push(Context::to_name(&name));
            }
        }
        Ok(context_names)
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
