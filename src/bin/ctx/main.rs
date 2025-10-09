use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use vespe::project::Project;
mod watch;
use tracing::debug;
use vespe::agent::ShellAgentCall;

use handlebars::Handlebars;
use serde_json::json;

const DIARY_CONTEXT_FORMAT: &str = "diary/%Y-%m-%d";
const DEFAULT_CONTEXT_TEMPLATE: &str = r#"@include rules

# {{context_name}} - {{title}}
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
    /// Manages snippets within the project.
    Snippet {
        #[command(subcommand)]
        command: SnippetCommands,
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
    },
    /// Lists all available contexts.
    List {},
    /// Displays the dependency tree for a context.
    Tree {
        /// The name of the context to display the tree for.
        name: String,
    },
}

#[derive(Subcommand)]
enum SnippetCommands {
    /// Creates a new snippet file.
    New {
        /// The name of the snippet file (e.g., "common/header").
        name: String,
        /// Optional initial content for the snippet file.
        #[arg(long, value_name = "CONTENT")]
        content: Option<String>,
    },
    /// Lists all available snippets.
    List {},
}

/*fn print_context_tree(context: &Context, indent: usize) {
     TODO redo
    let indent_str = "  ".repeat(indent);
    println!(
        "{}{}",
        indent_str,
        //Yellow.paint(format!("Context: {}", context.name))
    );

    for (line_index, included_context) in &context.includes {
        println!(
            "{}{}",
            indent_str,
            //Green.paint(format!(
                "  @include (line {}): {}",
                line_index, included_context.name
            ))
        );
        //print_context_tree(included_context, indent + 2);
    }

    for (line_index, summarized_context) in &context.summaries {
        println!(
            "{}{}",
            indent_str,
            //Purple.paint(format!(
                "  @summary (line {}): {}",
                line_index, summarized_context.name
            ))
        );
        //print_context_tree(summarized_context, indent + 2);
    }

    for (line_index, inlined_snippet) in &context.inlines {
        println!(
            "{}{}",
            indent_str,
            //Cyan.paint(format!(
                "  @inline (line {}): {}",
                line_index, inlined_snippet.name
            ))
        );
    }

    for line_index in &context.answers {
        println!(
            "{}{}",
            indent_str,
            //Red.paint(format!("  @answer (line {})", line_index))
        );
    }

}*/

fn get_context_name(today: bool, name: Option<String>, format_str: &str) -> Result<String> {
    if today {
        Ok(chrono::Local::now().format(format_str).to_string())
    } else {
        name.ok_or_else(|| anyhow::anyhow!("Context name is required unless --today is specified."))
    }
}

fn main() -> Result<()> {
    vespe::init_telemetry();

    debug!("Starting vespe CLI...");

    let cli = Cli::parse();

    let current_dir = std::env::current_dir()?;
    let project_path = cli.project_root.unwrap_or(current_dir);

    match cli.command {
        Commands::Init {} => {
            let project = Project::init(&project_path)?;
            println!(
                "Initialized new .ctx project at: {}",
                project.project_home().display()
            );

            let ctx_dir = project.project_home();
            let ctx_root_file = ctx_dir.join(".ctx_root");
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

                    let title = context_name
                        .split('/')
                        .last()
                        .unwrap_or(&context_name)
                        .replace('_', " ");
                    let data = json!({
                        "context_name": context_name,
                        "title": title,
                    });

                    let rendered_content = handlebars.render("context_template", &data)?;

                    let file_path =
                        project.create_context_file(&context_name, Some(rendered_content))?;
                    println!("Created new context file: {}", file_path.display());                    
                }
                ContextCommands::Execute { name, today } => {
                    let context_name = get_context_name(today, name, DIARY_CONTEXT_FORMAT)?;
                    println!("Executing context '{}'...", context_name);
                    let agent = ShellAgentCall::new(
                        "gemini -p -y -m gemini-2.5-flash".to_string(),
                        &project,
                    )?;
                    project.execute_context(&context_name, &agent)?;
                    println!("Context '{}' executed successfully.", context_name);
                }
                ContextCommands::List {} => {
                    let contexts = project.list_contexts()?;
                    if contexts.is_empty() {
                        println!("No contexts found.");
                    } else {
                        println!("Available contexts:");
                        for context in contexts {
                            println!("  - {} ({})", context.name, context.path.display());
                        }
                    }
                }
                ContextCommands::Tree { name: _name } => {
                    // TODO redo let context_tree = project.get_context_tree(&name)?;
                    //print_context_tree(&context_tree, 0);
                }
            }
        }
        Commands::Snippet { command } => {
            let project = Project::find(&project_path)?;
            match command {
                SnippetCommands::New { name, content } => {
                    let file_path = project.create_snippet_file(&name, content)?;
                    println!("Created new snippet file: {}", file_path.display());                  
                }
                SnippetCommands::List {} => {
                    let snippets = project.list_snippets()?;
                    if snippets.is_empty() {
                        println!("No snippets found.");
                    } else {
                        println!("Available snippets:");
                        for snippet in snippets {
                            println!("  - {} ({})", snippet.name, snippet.path.display());
                        }
                    }
                }
            }
        }
        Commands::Watch {} => {
            let project = Project::find(&project_path)?;
            let agent =
                ShellAgentCall::new("gemini -p -y -m gemini-2.5-flash".to_string(), &project)?;
            watch::watch(&project, &agent)?;
        }
    }

    Ok(())
}
