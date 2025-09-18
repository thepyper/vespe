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
}
