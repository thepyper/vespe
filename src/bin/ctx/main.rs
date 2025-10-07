use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use vespe::execute;
use vespe::project::Project;
mod watch;
use tracing::debug;
use vespe::agent::ShellAgentCall;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Specify the project root directory. Defaults to the current directory.
    #[arg(long, value_name = "PATH")]
    project_root: Option<PathBuf>,

    /// Specify the editor interface to use (e.g., "vscode", "none"). Defaults to "vscode".
    #[arg(long, value_name = "INTERFACE", default_value = "vscode")]
    editor_interface: String,

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
        name: String,
    },
    /// Executes a context.
    Execute {
        /// The name of the context to execute.
        name: String,
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

fn main() -> Result<()> {
    vespe::init_telemetry();

    debug!("Starting vespe CLI...");

    let cli = Cli::parse();

    let current_dir = std::env::current_dir()?;
    let project_path = cli.project_root.unwrap_or(current_dir);

    match cli.command {
        Commands::Init {} => {
            let project = Project::init(&project_path, &cli.editor_interface)?;
            println!(
                "Initialized new .ctx project at: {}",
                project.project_home().display()
            );
        }
        Commands::Context { command } => {
            let project = Project::find(&project_path, &cli.editor_interface)?;
            match command {
                ContextCommands::New { name } => {
                    let file_path = project.create_context_file(&name)?;
                    println!("Created new context file: {}", file_path.display());
                }
                ContextCommands::Execute { name } => {
                    println!("Executing context '{}'...", name);
                    let agent = ShellAgentCall::new("gemini -p -y -m gemini-2.5-flash".to_string(), &project)?;
                    execute::execute(&project, &name, &agent)?;
                    println!("Context '{}' executed successfully.", name);
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
            let project = Project::find(&project_path, &cli.editor_interface)?;
            match command {
                SnippetCommands::New { name } => {
                    let file_path = project.create_snippet_file(&name)?;
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
            let project = Project::find(&project_path, &cli.editor_interface)?;
            let agent = ShellAgentCall::new("gemini -p -y -m gemini-2.5-flash".to_string(), &project)?;
            watch::watch(&project, &agent)?;
        }
    }

    Ok(())
}
