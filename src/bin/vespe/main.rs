use anyhow::Result;
use clap::{ArgAction, Parser, Subcommand};
use handlebars::Handlebars;
use serde_json::json;
use std::io::{self, Read};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use vespe::error::Error;
use vespe::execute2::{AnchorAnalysis, AnchorState, ContextAnalysis};
use vespe::project::Project;

mod watch;

const DEFAULT_TRUNCATION_LIMIT: usize = 100;
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
    /// Manages project-level configurations.
    Project {
        #[command(subcommand)]
        command: ProjectCommands,
    },
    /// Watches for changes in context files and re-executes them.
    Watch {},
}

#[derive(Subcommand)]
enum ProjectCommands {
    /// Adds an auxiliary path to the project configuration.
    AddAuxPath {
        /// The path to add as an auxiliary path.
        #[arg(value_name = "PATH")]
        path: PathBuf,
    },
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
        /// Define a key-value pair for the context.
        #[arg(short = 'D', long = "define", value_name = "KEY=VALUE", action = ArgAction::Append)]
        defines: Option<Vec<String>>,
        /// Additional auxiliary paths to search for input files.
        #[arg(short = 'I', long = "aux_path", value_name = "PATH", action = ArgAction::Append)]
        aux_paths: Option<Vec<PathBuf>>,
        /// Alternative output path for the context.
        #[arg(short = 'O', long = "output-path", value_name = "PATH")]
        output_path: Option<PathBuf>,
    },
    /// Analyzes a context file.
    Analyze {
        /// The name of the context to analyze.
        #[arg(value_name = "NAME")]
        context_name: String,
        #[arg(long = "filter-uuid", help = "Filter anchors by UUID prefix")]
        filter_uuid: Option<String>,
    },
}

fn get_context_name(today: bool, name: Option<String>, format_str: &str) -> Result<String> {
    let context_name = if today {
        Ok(chrono::Local::now().format(format_str).to_string())
    } else {
        name.ok_or_else(|| anyhow::Error::from(Error::ContextNameRequired))
    };
    Ok(format!("{}.md", &context_name?))
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
                ContextCommands::Run {
                    name,
                    today,
                    args,
                    defines,
                    aux_paths,
                    output_path,
                } => {
                    let context_name = get_context_name(today, name, DIARY_CONTEXT_FORMAT)?;
                    tracing::info!(
                        "Executing context '{}' with args {:?}...",
                        context_name,
                        args
                    );
                    let input = read_input()?;
                    let input_data = vespe::project::ExecuteContextInput {
                        context_name: context_name.clone(),
                        input_file: input,
                        args: Some(args),
                        defines,
                        additional_aux_paths: aux_paths,
                        output_path,
                    };
                    let content = project.execute_context(input_data)?;
                    tracing::info!("Context '{}' executed successfully.", context_name);
                    print!("{}", content.to_string());
                }
                ContextCommands::Analyze {
                    context_name,
                    filter_uuid,
                } => {
                    let context_name = format!("{}.md", &context_name);
                    let mut analysis = project.analyze_context(&context_name)?;

                    if let Some(filter) = &filter_uuid {
                        analysis
                            .anchors
                            .retain(|uuid, _| uuid.to_string().starts_with(filter));
                    }

                    display_analysis_report(&analysis)?;
                }
            }
        }
        Commands::Project { command } => {
            let mut project = Project::find(&project_path)?;
            tracing::info!(
                "Found .ctx project at: {}",
                project.project_home().display()
            );
            match command {
                ProjectCommands::AddAuxPath { path } => {
                    project.add_aux_path(path.clone())?;
                    tracing::info!("Added auxiliary path: {}", path.display());
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
    let (tx, rx) = mpsc::channel::<Result<String, Error>>();

    thread::spawn(move || {
        let mut input = String::new();
        let res = io::stdin().read_to_string(&mut input)
            .map(|_| input)
            .map_err(Error::StdinReadError);
        let _ = tx.send(res);
    });

    match rx.recv_timeout(Duration::from_millis(250)) {
        Ok(Ok(data)) => Ok(Some(data)),
        Ok(Err(e)) => Err(e.into()),
        Err(mpsc::RecvTimeoutError::Timeout) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

fn display_analysis_report(analysis: &ContextAnalysis) -> Result<()> {
    println!("Context Analysis Report");
    println!("=======================");
    if analysis.anchors.is_empty() {
        println!("No relevant anchors found.");
    } else {
        let answer_anchor_count = analysis
            .anchors
            .values()
            .filter(|aa| matches!(aa.state, AnchorState::Answer(_)))
            .count();

        for (_, anchor_analysis) in &analysis.anchors {
            match &anchor_analysis.state {
                AnchorState::Answer(_) => {
                    display_answer_analysis(anchor_analysis, answer_anchor_count)
                }
                AnchorState::Inline(_) => display_inline_analysis(anchor_analysis),
                AnchorState::Task(_) => display_task_analysis(anchor_analysis),
            }
            println!("------------------------------------------------------------");
        }
    }
    Ok(())
}

fn display_answer_analysis(analysis: &AnchorAnalysis, answer_anchor_count: usize) {
    if let AnchorState::Answer(state) = &analysis.state {
        let truncation_limit = if answer_anchor_count == 1 {
            usize::MAX // No truncation
        } else if answer_anchor_count >= 2 && answer_anchor_count <= 5 {
            DEFAULT_TRUNCATION_LIMIT * 20
        } else {
            DEFAULT_TRUNCATION_LIMIT
        };

        println!("Anchor (Answer): {}", analysis.anchor.uuid);
        println!("  Status: {:?}", state.status);

        let query_display = if state.query.len() > truncation_limit {
            format!(
                "{:.limit$}...",
                state
                    .query
                    .chars()
                    .take(truncation_limit)
                    .collect::<String>(),
                limit = truncation_limit
            )
        } else {
            state.query.clone()
        };
        println!(
            "+ Query: +++++++++++++++++++++++++++++++++++++++++++++++++++\n{}",
            query_display
        );

        let reply_display = if state.reply.len() > truncation_limit {
            format!(
                "{:.limit$}...",
                state
                    .reply
                    .chars()
                    .take(truncation_limit)
                    .collect::<String>(),
                limit = truncation_limit
            )
        } else {
            state.reply.clone()
        };
        println!(
            "+ Reply:  +++++++++++++++++++++++++++++++++++++++++++++++++++\n{}",
            reply_display
        );
    }
}

fn display_inline_analysis(analysis: &AnchorAnalysis) {
    if let AnchorState::Inline(state) = &analysis.state {
        println!("Anchor (Inline): {}", analysis.anchor.uuid);
        println!("  Status: {:?}", state.status);
        if let Some(arg) = analysis.anchor.arguments.arguments.get(0) {
            println!("  Source: {}", arg.value);
        }
    }
}

fn display_task_analysis(analysis: &AnchorAnalysis) {
    if let AnchorState::Task(state) = &analysis.state {
        println!("Anchor (Task): {}", analysis.anchor.uuid);
        println!("  Status: {:?}", state.status);
    }
}
