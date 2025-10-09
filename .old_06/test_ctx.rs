use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compose a context by resolving includes
    Compose { name: String },
    
    /// List available contexts
    List,
    
    /// Create new context
    New { name: String },
    
    /// Edit a context
    Edit { name: String },
    
    /// Show dependency tree
    Tree { name: String },
}

fn contexts_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap()
        .join("ctx")
        .join("contexts")
}

fn compose(name: &str) -> Result<String> {
    let path = contexts_dir().join(format!("{}.md", name));
    let mut visited = HashSet::new();
    compose_recursive(&path, &mut visited)
}

fn compose_recursive(path: &Path, visited: &mut HashSet<PathBuf>) -> Result<String> {
    if visited.contains(path) {
        return Ok(String::new()); // Circular include
    }
    visited.insert(path.to_path_buf());
    
    let content = std::fs::read_to_string(path)
        .context(format!("Failed to read {:?}", path))?;
    
    let mut output = String::new();
    
    for line in content.lines() {
        if let Some(include) = line.strip_prefix("@include ") {
            let include_name = include.trim();
            let include_path = contexts_dir().join(format!("{}.md", include_name));
            
            output.push_str(&compose_recursive(&include_path, visited)?);
            output.push('\n');
        } else {
            output.push_str(line);
            output.push('\n');
        }
    }
    
    Ok(output)
}

fn list() -> Result<()> {
    for entry in std::fs::read_dir(contexts_dir())? {
        let entry = entry?;
        if entry.path().extension() == Some("md".as_ref()) {
            println!("{}", entry.path().file_stem().unwrap().to_string_lossy());
        }
    }
    Ok(())
}

fn new(name: &str) -> Result<()> {
    let path = contexts_dir().join(format!("{}.md", name));
    
    if path.exists() {
        anyhow::bail!("Context '{}' already exists", name);
    }
    
    std::fs::write(&path, format!("# {}\n\n", name))?;
    println!("Created {}", path.display());
    Ok(())
}

fn edit(name: &str) -> Result<()> {
    let path = contexts_dir().join(format!("{}.md", name));
    
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
    
    std::process::Command::new(editor)
        .arg(&path)
        .status()?;
    
    Ok(())
}

fn tree(name: &str, depth: usize) -> Result<()> {
    let path = contexts_dir().join(format!("{}.md", name));
    let content = std::fs::read_to_string(&path)?;
    
    println!("{}{}", "  ".repeat(depth), name);
    
    for line in content.lines() {
        if let Some(include) = line.strip_prefix("@include ") {
            tree(include.trim(), depth + 1)?;
        }
    }
    
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Ensure contexts directory exists
    std::fs::create_dir_all(contexts_dir())?;
    
    match cli.command {
        Commands::Compose { name } => {
            let output = compose(&name)?;
            print!("{}", output);
        }
        Commands::List => list()?,
        Commands::New { name } => new(&name)?,
        Commands::Edit { name } => edit(&name)?,
        Commands::Tree { name } => tree(&name, 0)?,
    }
    
    Ok(())
}