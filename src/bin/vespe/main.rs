use anyhow::Result;
use clap::{Parser, Subcommand};
use handlebars::Handlebars;
use serde_json::json;
use std::io::{self, Read};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use vespe::project::Project;

mod watch;

const DIARY_CONTEXT_FORMAT: &str = "diary/%Y-%m-%d";
const DEFAULT_CONTEXT_TEMPLATE: &str = r#"
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
    /// Initializes a new .vespe project in the current or specified directory.
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
    /// Runs a context.
    Run {
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
            tracing::info!(
                "Found .ctx project at: {}",
                project.project_home().display()
            );
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
                ContextCommands::Run { name, today, args } => {
                    let context_name = get_context_name(today, name, DIARY_CONTEXT_FORMAT)?;
                    tracing::info!(
                        "Executing context '{}' with args {:?}...",
                        context_name,
                        args
                    );
                    let input = read_input()?;
                    let content = project.execute_context(&context_name, input, Some(args))?;
                    tracing::info!("Context '{}' executed successfully.", context_name);
                    print!("{}", content.to_string());
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

fn read_input() -> Result<Option<String>> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input).unwrap();
        let _ = tx.send(input);
    });

    match rx.recv_timeout(Duration::from_millis(250)) {
        Ok(data) => Ok(Some(data)),
        Err(mpsc::RecvTimeoutError::Timeout) => Ok(None),
        Err(e) => Err(e.into()),
    }
}
