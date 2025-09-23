use clap::Parser;
use vespe::cli::commands::{Cli, Commands, ProjectSubcommand, TaskSubcommand, ToolSubcommand};
use project::utils::{find_project_root, initialize_project_root};
use std::path::PathBuf;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing for logging
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let cli = Cli::parse();

    // The `init` command is special as it might be run outside of a project root.
    // We handle it before attempting to find the project root.
    if let Commands::Project(project_command) = &cli.command {
        if let ProjectSubcommand::Init { path } = &project_command.command {
            let target_dir = path.clone().unwrap_or(std::env::current_dir()?);
            match initialize_project_root(&target_dir) {
                Ok(_) => {
                    println!("Vespe project initialized at: {}", target_dir.display());
                },
                Err(e) => {
                    eprintln!("Error initializing project: {}", e);
                }
            }
            return Ok(());
        }
    }

    // For all other commands, we need to be inside a project root.
    let project_root = if let Some(path) = cli.project_root {
        path
    } else {
        find_project_root(&std::env::current_dir()?)
            .ok_or_else(|| anyhow::anyhow!("Project root not found. Please run 'vespe project init' or specify --project-root."))?
    };

    match &cli.command {
        Commands::Project(project_command) => match &project_command.command {
            ProjectSubcommand::Init { .. } => { /* Handled above */ }
            ProjectSubcommand::Info => {
                println!("Project info for: {}", project_root.display());
                // Implementation for `project info` will go here
            }
            ProjectSubcommand::Validate => {
                println!("Validating project at: {}", project_root.display());
                // Implementation for `project validate` will go here
            }
        },
        Commands::Task(task_command) => match &task_command.command {
            TaskSubcommand::Create { name, template, parent } => {
                println!("Creating task: {}, template: {}, parent: {:?}", name, template, parent);
                // Implementation for `task create` will go here
            }
            TaskSubcommand::Show { identifier } => {
                println!("Showing task: {}", identifier);
                // Implementation for `task show` will go here
            }
            TaskSubcommand::DefineObjective { identifier, objective } => {
                println!("Defining objective for task: {}, objective: {}", identifier, objective);
                // Implementation for `task define-objective` will go here
            }
            TaskSubcommand::DefinePlan { identifier, plan } => {
                println!("Defining plan for task: {}, plan: {}", identifier, plan);
                // Implementation for `task define-plan` will go here
            }
            TaskSubcommand::List => {
                println!("Listing all tasks");
                // Implementation for `task list` will go here
            }
        },
        Commands::Tool(tool_command) => match &tool_command.command {
            ToolSubcommand::Create { name, description, schema } => {
                println!("Creating tool: {}, desc: {}, schema: {}", name, description, schema.display());
                // Implementation for `tool create` will go here
            }
            ToolSubcommand::Show { identifier } => {
                println!("Showing tool: {}", identifier);
                // Implementation for `tool show` will go here
            }
            ToolSubcommand::List => {
                println!("Listing all tools");
                // Implementation for `tool list` will go here
            }
        },
    }

    Ok(())
}
