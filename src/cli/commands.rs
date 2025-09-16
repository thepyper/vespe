use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
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
}
