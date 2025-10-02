use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use notify::{RecommendedWatcher, Watcher, RecursiveMode};
use std::sync::mpsc::{channel, RecvTimeoutError};
use std::time::Duration;
use std::collections::HashSet;

mod project;
mod agent_call;
mod ast;
mod composer;

use project::Project;
use crate::ast::LineData;
use agent_call::ShellAgentCall;

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

    /// Watch for changes in context files and execute them
    Watch,
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
            let agent = ShellAgentCall::new("echo".to_string()); // Dummy agent for compose
            let composed_lines = project.compose(&name, &agent)?;
            for line in composed_lines {
                if let LineData::Text(text) = line.data {
                    println!("{}", text);
                }
            }
        }
        Commands::List => {
            let contexts = project.list_contexts()?;
            if contexts.is_empty() {
                println!("No contexts found.");
            } else {
                for context_info in contexts {
                    println!("{}", context_info.name);
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
            println!("{}", tree);
        }
        Commands::Execute { name } => {
            let agent = ShellAgentCall::new("gemini -p -y -m gemini-2.5-flash".to_string());
            project.execute_context(&name, &agent)?;
        }
        Commands::Watch => {
            println!("Watching for changes in {:?}... (Press Ctrl+C to stop)", project.contexts_dir());

            let (tx, rx) = channel();
            let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())?;
            let contexts_dir = project.contexts_dir()?;
            watcher.watch(&contexts_dir, RecursiveMode::Recursive)?;

            let mut known_contexts: HashSet<String> = project.list_contexts()?.into_iter().map(|ci| ci.name).collect();

            let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
            let r = running.clone();
            ctrlc::set_handler(move || {
                r.store(false, std::sync::atomic::Ordering::SeqCst);
            }).expect("Error setting Ctrl-C handler");

            while running.load(std::sync::atomic::Ordering::SeqCst) {
                match rx.recv_timeout(Duration::from_secs(1)) {
                    Ok(Ok(event)) => {
                        if event.kind.is_create() || event.kind.is_modify() {
                            for path in event.paths {
                                if path.extension().map_or(false, |ext| ext == "md") {
                                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                                        println!("Executing context: {}", name);
                                        let agent = ShellAgentCall::new("gemini -p -y -m gemini-2.5-flash".to_string());
                                        if let Err(e) = project.execute_context(&name.to_string(), &agent) {
                                            eprintln!("Error executing context {}: {}", name, e);
                                            return Err(e);
                                        }
                                    }
                                }
                            }
                        }
                    },
                    Ok(Err(e)) => eprintln!("Watch error: {:?}", e),
                    Err(RecvTimeoutError::Timeout) => {
                        // Check for new contexts periodically
                        let current_contexts: HashSet<String> = project.list_contexts()?.into_iter().map(|ci| ci.name).collect();
                        for new_context in current_contexts.difference(&known_contexts) {
                            println!("New context detected: {}. Executing...", new_context);
                            let agent = ShellAgentCall::new("gemini -p -y -m gemini-2.5-flash".to_string());
                            if let Err(e) = project.execute_context(new_context, &agent) {
                                eprintln!("Error executing new context {}: {}", new_context, e);
                                return Err(e);
                            }
                        }
                        known_contexts = current_contexts;
                    },
                    Err(RecvTimeoutError::Disconnected) => {
                        println!("Watcher disconnected.");
                        break;
                    }
                }
            }
            println!("Watch stopped.");
        }
        Commands::Init => {
            // Unreachable, but needed for the match to be exhaustive
            unreachable!();
        }
    }
    
    Ok(())
}
