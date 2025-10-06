use clap::{Parser, Subcommand};
use std::path::PathBuf;
use anyhow::Result;
use vespe::project::{Project, ContextInfo, SnippetInfo, Context, Snippet, calculate_context_data};
use ansi_term::Colour::{Cyan, Green, Purple, Red, Yellow};
use std::collections::HashSet;

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
    /// Lists all available contexts.
    List {},
    /// Displays the dependency tree for a context.
    Tree {
        /// The name of the context to display the tree for.
        name: String
    },
}

#[derive(Subcommand)]
enum SnippetCommands {
    /// Creates a new snippet file.
    New {
        /// The name of the snippet file (e.g., "common/header").
        name: String
    },
    /// Lists all available snippets.
    List {},
}

fn print_context_tree(project: &Project, context: &Context, indent: usize, loading_contexts: &mut HashSet<String>) -> Result<()> {
    let indent_str = "  ".repeat(indent);
    println!("{}{}", indent_str, Yellow.paint(format!("Context: {}", context.info.name)));

    let context_data = vespe::project::calculate_context_data(project, context, loading_contexts)?;

    for (line_index, included_context) in &context_data.includes {
        println!("{}{}", indent_str, Green.paint(format!("  @include (line {}): {}", line_index, included_context.info.name)));
        print_context_tree(project, included_context, indent + 2, loading_contexts)?;
    }

    for (line_index, summarized_context) in &context_data.summaries {
        println!("{}{}", indent_str, Purple.paint(format!("  @summary (line {}): {}", line_index, summarized_context.info.name)));
        print_context_tree(project, summarized_context, indent + 2, loading_contexts)?;
    }

    for (line_index, inlined_snippet) in &context_data.inlines {
        println!("{}{}", indent_str, Cyan.paint(format!("  @inline (line {}): {}", line_index, inlined_snippet.name)));
    }

    for line_index in &context_data.answers {
        println!("{}{}", indent_str, Red.paint(format!("  @answer (line {})", line_index)));
    }
    Ok(())
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
                },
                ContextCommands::Tree { name } => {
                    let context_tree = project.get_context_tree(&name)?;
                    let mut loading_contexts = HashSet::new();
                    print_context_tree(&project, &context_tree, 0, &mut loading_contexts)?;
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
                },
            }
        },
    }

    Ok(())
}
