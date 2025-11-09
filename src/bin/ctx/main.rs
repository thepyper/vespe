use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use vespe::project::Project;
mod watch;

use handlebars::Handlebars;
use serde_json::json;

const DIARY_CONTEXT_FORMAT: &str = "diary/%Y-%m-%d";
const DEFAULT_CONTEXT_TEMPLATE: &str = r#"@include rules

# {{title}}
"#;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Specify the project root directory. Defaults to the current directory.
    #[arg(long, value_name = "PATH")]
    project_root: Option<PathBuf>,

    /// Specify a Handlebars template file for new contexts.
    #[arg(long, value_name = "FILE")]
    context_template: Option<PathBuf>,

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
    /// Watches for changes in context files and re-executes them.
    Watch {},
}

#[derive(Subcommand)]
enum ContextCommands {
    /// Creates a new context file.
    New {
        /// The name of the context file (e.g., "my_feature/overview").
        #[arg(value_name = "NAME")]
        name: Option<String>,
        /// If specified, the context name will be automatically generated as "diary/YYYY-mm-DD".
        #[arg(long)]
        today: bool,
    },
    /// Executes a context.
    Execute {
        /// The name of the context to execute.
        #[arg(value_name = "NAME")]
        name: Option<String>,
        /// If specified, the context name will be automatically generated as "diary/YYYY-mm-DD".
        #[arg(long)]
        today: bool,
        /// The arguments to pass to the context.
        #[arg()]
        args: Vec<String>,
    },
}

fn get_context_name(today: bool, name: Option<String>, format_str: &str) -> Result<String> {
    if today {
        Ok(chrono::Local::now().format(format_str).to_string())
    } else {
        name.ok_or_else(|| anyhow::anyhow!("Context name is required unless --today is specified."))
    }
}

fn main() -> Result<()> {
    vespe::init_telemetry();

    let cli = Cli::parse();

    let current_dir = std::env::current_dir()?;
    let project_path = cli.project_root.unwrap_or(current_dir);

    match cli.command {
        Commands::Init {} => {
            let _ = Project::init(&project_path)?;
            tracing::info!(
                "Initialized new .ctx project at: {}",
                project_path.display()
            );
        }
        Commands::Context { command } => {
            let project = Project::find(&project_path)?;
            match command {
                ContextCommands::New { name, today } => {
                    let context_name = get_context_name(today, name, DIARY_CONTEXT_FORMAT)?;

                    let mut handlebars = Handlebars::new();
                    handlebars.register_template_string("context_template", {
                        if let Some(template_path) = &cli.context_template {
                            std::fs::read_to_string(template_path)?
                        } else {
                            DEFAULT_CONTEXT_TEMPLATE.to_string()
                        }
                    })?;

                    let title = String::new();
                    let data = json!({
                        "context_name": context_name,
                        "title": title,
                    });

                    let rendered_content = handlebars.render("context_template", &data)?;

                    let file_path =
                        project.create_context_file(&context_name, Some(rendered_content))?;
                    tracing::info!("Created new context file: {}", file_path.display());
                }
                ContextCommands::Execute { name, today, args } => {
                    let context_name = get_context_name(today, name, DIARY_CONTEXT_FORMAT)?;
                    tracing::info!(
                        "Executing context '{}' with args {:?}...",
                        context_name,
                        args
                    );
                    project.execute_context(&context_name, Some(args))?;
                    tracing::info!("Context '{}' executed successfully.", context_name);
                }
            }
        }
        Commands::Watch {} => {
            let project = Project::find(&project_path)?;
            watch::watch(&project)?;
        }
    }

    Ok(())
}
