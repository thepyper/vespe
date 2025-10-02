use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

mod project;
use project::Project;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new ctx project
    Init,
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

fn contexts_dir(project: &Project) -> PathBuf {
    project.root_path.join("contexts")
}

fn compose(project: &Project, name: &str) -> Result<String> {
    let path = contexts_dir(project).join(format!("{}.md", name));
    let mut visited = HashSet::new();
    compose_recursive(project, &path, &mut visited)
}

fn compose_recursive(project: &Project, path: &Path, visited: &mut HashSet<PathBuf>) -> Result<String> {
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
            let include_path = contexts_dir(project).join(format!("{}.md", include_name));
            
            output.push_str(&compose_recursive(project, &include_path, visited)?);
            output.push('\n');
        } else {
            output.push_str(line);
            output.push('\n');
        }
    }
    
    Ok(output)
}

fn list(project: &Project) -> Result<()> {
    for entry in std::fs::read_dir(contexts_dir(project))? {
        let entry = entry?;
        if entry.path().extension() == Some("md".as_ref()) {
            println!("{}", entry.path().file_stem().unwrap().to_string_lossy());
        }
    }
    Ok(())
}

fn new(project: &Project, name: &str) -> Result<()> {
    let path = contexts_dir(project).join(format!("{}.md", name));
    
    if path.exists() {
        anyhow::bail!("Context '{}' already exists", name);
    }
    
    std::fs::write(&path, format!("# {}\n\n", name))?;
    println!("Created {}", path.display());
    Ok(())
}

fn edit(project: &Project, name: &str) -> Result<()> {
    let path = contexts_dir(project).join(format!("{}.md", name));
    
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
    
    std::process::Command::new(editor)
        .arg(&path)
        .status()?;
    
    Ok(())
}

fn tree(project: &Project, name: &str, depth: usize) -> Result<()> {
    let path = contexts_dir(project).join(format!("{}.md", name));
    let content = std::fs::read_to_string(&path)?;
    
    println!("{}{}", "  ".repeat(depth), name);
    
    for line in content.lines() {
        if let Some(include) = line.strip_prefix("@include ") {
            tree(project, include.trim(), depth + 1)?;
        }
    }
    
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            let current_dir = std::env::current_dir()?;
            let project = Project::init(&current_dir)?;
            println!("Initialized ctx project in {:?}", project.root_path);
        }
        Commands::Compose { name } => {
            let project = Project::find(&std::env::current_dir()?)?;
            let output = compose(&project, &name)?;
            print!("{}", output);
        }
        Commands::List => {
            let project = Project::find(&std::env::current_dir()?)?;
            list(&project)?;
        }
        Commands::New { name } => {
            let project = Project::find(&std::env::current_dir()?)?;
            new(&project, &name)?;
        }
        Commands::Edit { name } => {
            let project = Project::find(&std::env::current_dir()?)?;
            edit(&project, &name)?;
        }
        Commands::Tree { name } => {
            let project = Project::find(&std::env::current_dir()?)?;
            tree(&project, &name, 0)?;
        }
    }
    
    Ok(())
}
