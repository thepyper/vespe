use anyhow::{Context as AnyhowContext, Result};
use clap::{Parser, Subcommand};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

mod context;
mod project;
use context::{Context, ContextTreeItem, Line};
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

    /// Execute a context by composing it and piping to an LLM
    Execute { name: String },
    
    /// Edit a context
    Edit { name: String },
    
    /// Show dependency tree
    Tree { name: String },
}

fn print_tree(item: &ContextTreeItem, depth: usize) {
    match item {
        ContextTreeItem::Node { name, children } => {
            println!("{}{}", "  ".repeat(depth), name);
            for child in children {
                print_tree(child, depth + 1);
            }
        }
        ContextTreeItem::Leaf { name } => {
            println!("{}{}", "  ".repeat(depth), name);
        }
    }
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
            let output = project.compose(&name)?;
            print!("{}", output);
        }
        Commands::List => {
            let contexts = project.list_contexts()?;
            if contexts.is_empty() {
                println!("No contexts found.");
            } else {
                for context_name in contexts {
                    println!("{}", context_name);
                }
            }
        }
        Commands::New { name } => {
            project.new_context(&name)?;
        }
        Commands::Edit { name } => {
            project.edit_context(&name)?;
        }
        Commands::Tree { name } => {
            let tree = project.context_tree(&name)?;
            print_tree(&tree, 0);
        }
        Commands::Execute { name } => {
            let composed_context = project.compose(&name)?;

            let mut child = Command::new("gemini")
                .arg("-p")
                .arg("-y")
                .arg("-m")
                .arg("gemini-2.5-flash")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .context("Failed to spawn gemini command")?;

            // Write the composed context to gemini's stdin
            use std::io::Write;
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(composed_context.as_bytes())?;
            }

            let output = child.wait_with_output()?;

            if output.status.success() {
                print!("{}", String::from_utf8_lossy(&output.stdout));
            } else {
                eprint!("gemini command failed: {}", String::from_utf8_lossy(&output.stderr));
                anyhow::bail!("gemini command failed");
            }
        }
        Commands::Init => {
            // Unreachable, but needed for the match to be exhaustive
            unreachable!();
        }
    }
    
    Ok(())
}
