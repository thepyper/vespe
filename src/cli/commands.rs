use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// A fictional versioning CLI
#[derive(Debug, Parser)]
#[command(name = "vespe")]
#[command(about = "A fictional versioning CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    /// Sets the project root path.
    #[arg(long, global = true)]
    pub project_root: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Manage projects
    Project(ProjectCommand),
    /// Manage tasks
    Task(TaskCommand),
    /// Manage tools
    Tool(ToolCommand),
}

#[derive(Debug, Parser)]
pub struct ProjectCommand {
    #[command(subcommand)]
    pub command: ProjectSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum ProjectSubcommand {
    /// Initialize a new Vespe project
    Init {
        /// The path to initialize the project in. Defaults to the current directory.
        path: Option<PathBuf>,
    },
    /// Show project information
    Info,
    /// Validate the project structure and files
    Validate,
}

#[derive(Debug, Parser)]
pub struct TaskCommand {
    #[command(subcommand)]
    pub command: TaskSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum TaskSubcommand {
    /// Create a new task
    Create {
        #[arg(long)]
        name: String,
        #[arg(long, default_value = "default")]
        template: String,
        #[arg(long)]
        parent: Option<String>,
    },
    /// Show details for a specific task
    Show {
        /// The UID or name of the task
        identifier: String,
    },
    /// Define the objective for a task
    DefineObjective {
        /// The UID or name of the task
        identifier: String,
        /// The objective content
        objective: String,
    },
    /// Define the plan for a task
    DefinePlan {
        /// The UID or name of the task
        identifier: String,
        /// The plan content
        plan: String,
    },
    /// List all tasks
    List,
    /// Review a task
    Review {
        /// The UID or name of the task
        identifier: String,
        /// Approve the task
        #[arg(long, conflicts_with = "reject")]
        approve: bool,
        /// Reject the task and mark for replanning
        #[arg(long, conflicts_with = "approve")]
        reject: bool,
        /// Optional: New name for the replanned task (if rejected)
        #[arg(long, requires = "reject")]
        new_name: Option<String>,
    },
}

#[derive(Debug, Parser)]
pub struct ToolCommand {
    #[command(subcommand)]
    pub command: ToolSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum ToolSubcommand {
    /// Create a new tool
    Create {
        #[arg(long)]
        name: String,
        #[arg(long)]
        description: String,
        /// Path to the JSON schema file for the tool
        #[arg(long)]
        schema: PathBuf,
    },
    /// Show details for a specific tool
    Show {
        /// The UID or name of the tool
        identifier: String,
    },
    /// List all available tools
    List,
}
