use clap::{Parser, Subcommand};
use std::path::PathBuf;
use anyhow::Result;
use vespe::project::Project;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Specify the project root directory. Defaults to the current directory.
    #[arg(long, value_name = "PATH")]
    project_root: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initializes a new .ctx project in the current or specified directory.
    Init {},
    /// Manages contexts within the project.
    Context { 
        #[command(subcommand)]
        command: ContextCommands,
    },
    /// Manages snippets within the project.
    Snippet { 
        #[command(subcommand)]
        command: SnippetCommands,
    },
}

#[derive(Subcommand)]
enum ContextCommands {
    /// Creates a new context file.
    New { 
        /// The name of the context file (e.g., "my_feature/overview").
        name: String 
    },
    /// Executes a context (placeholder).
    Execute {},
}

#[derive(Subcommand)]
enum SnippetCommands {
    /// Creates a new snippet file.
    New { 
        /// The name of the snippet file (e.g., "common/header").
        name: String 
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let current_dir = std::env::current_dir()?;
    let project_path = cli.project_root.unwrap_or(current_dir);

    match cli.command {
        Commands::Init {} => {
            let project = Project::init(&project_path)?;
            println!("Initialized new .ctx project at: {}", project.project_home().display());
        },
        Commands::Context { command } => {
            let project = Project::find(&project_path)?;
            match command {
                ContextCommands::New { name } => {
                    let file_path = project.create_context_file(&name)?;
                    println!("Created new context file: {}", file_path.display());
                },
                ContextCommands::Execute {} => {
                    println!("Executing context (placeholder)...");
                },
            }
        },
        Commands::Snippet { command } => {
            let project = Project::find(&project_path)?;
            match command {
                SnippetCommands::New { name } => {
                    let file_path = project.create_snippet_file(&name)?;
                    println!("Created new snippet file: {}", file_path.display());
                },
            }
        },
    }

    Ok(())
}