use anyhow::{Context as AnyhowContext, Result};
use clap::{Parser, Subcommand};
use std::collections::HashSet;
use std::os::windows::process;
use std::path::{Path, PathBuf};

mod context;
mod project;
use context::Context;
use project::Project;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[clap(global = true, long)]
    project_root: Option<PathBuf>,
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

fn compose(project: &Project, name: &str) -> Result<String> {
    let path = project.contexts_dir()?.join(Context::to_filename(name));
    let mut visited = HashSet::new();
    compose_recursive(project, &path, &mut visited)
}

fn compose_recursive(project: &Project, path: &Path, visited: &mut HashSet<PathBuf>) -> Result<String> {
    if visited.contains(path) {
        return Ok(String::new()); // Circular include
    }
    visited.insert(path.to_path_buf());
    
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {:?}", path))?;
    
    let mut output = String::new();
    
    for line in content.lines() {
        if let Some(include) = line.strip_prefix("@include ") {
            let include_name = include.trim();
            let include_path = project.contexts_dir()?.join(Context::to_filename(include_name));
            
            output.push_str(&compose_recursive(project, &include_path, visited)?);
            output.push('\n');
        } else {
            output.push_str(line);
            output.push('\n');
        }
    }
    
    Ok(output)
}

fn edit(project: &Project, name: &str) -> Result<()> {
    let path = project.contexts_dir()?.join(Context::to_filename(name));
    
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
    
    std::process::Command::new(editor)
        .arg(&path)
        .status()?;
    
    Ok(())
}

fn tree(project: &Project, name: &str, depth: usize) -> Result<()> {
    let path = project.contexts_dir()?.join(Context::to_filename(name));
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

    if let Commands::Init = &cli.command {
        let path = cli.project_root.unwrap_or(std::env::current_dir()?);
        let project = Project::init(&path)?;
        println!("Initialized ctx project in {:?}", project.root_path);
        return Ok(());
    }

    let project = {
        let search_path = cli.project_root.unwrap_or(std::env::current_dir()?);
        Project::find(&search_path)?
    };

    match cli.command {
        Commands::Compose { name } => {
            let output = compose(&project, &name)?;
            print!("{}", output);
        }
        Commands::List => {
            project.list_contexts()?;
        }
        Commands::New { name } => {
            project.new_context(&name)?;
        }
        Commands::Edit { name } => {
            edit(&project, &name)?;
        }
        Commands::Tree { name } => {
            tree(&project, &name, 0)?;
        }
        Commands::Init => {
            // Unreachable, but needed for the match to be exhaustive
            unreachable!();
        }
    }
    
    Ok(())
}

