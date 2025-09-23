use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(long, global = true, help = "Specify the project root directory")]
    pub project_root: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Interact with an agent
    Chat {
        /// Name of the agent to chat with
        agent_name: String,
        /// Message to send to the agent
        message: String,
    },
    /// Initializes the current directory as a Vespe project root
    Init {
        #[arg(help = "Optional path to initialize as a Vespe project root. Defaults to current directory.")]
        path: Option<PathBuf>,
    },
    /// Resets all collected statistics by deleting the statistics file
    ResetStats,

    /// Manage tasks
    Task {
        #[command(subcommand)]
        command: TaskCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum TaskCommands {
    /// Create a new task
    Create {
        #[arg(long, help = "Optional parent task UID")]
        parent_uid: Option<String>,
        #[arg(help = "Name of the task")]
        name: String,
        #[arg(long, help = "User or agent creating the task")]
        created_by: String,
        #[arg(long, help = "Template name for the task (e.g., 'default')")]
        template_name: String,
    },
    /// Show details of a task
    Show {
        #[arg(help = "UID of the task to show")]
        uid: String,
    },
    /// Define the objective of a task
    DefineObjective {
        #[arg(help = "UID of the task")]
        uid: String,
        #[arg(help = "Content of the objective")]
        content: String,
    },
    /// Define the plan for a task
    DefinePlan {
        #[arg(help = "UID of the task")]
        uid: String,
        #[arg(help = "Content of the plan")]
        content: String,
    },
    /// List all tasks
    List,
}
