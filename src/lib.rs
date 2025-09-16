use anyhow::Result;
use std::path::PathBuf;
use tracing::info;
use clap::Parser;

pub mod agent;
pub mod cli;
pub mod config;
pub mod llm;

pub async fn run() -> Result<()> {
    info!("Vespe application started.");

    let cli = cli::commands::Cli::parse();

    let global_config = config::load_global_config().await?;
    let _final_config = config::load_project_config(global_config).await?;

    match cli.command {
        cli::commands::Commands::Chat { agent_name, message } => {
            info!("Chat command received for agent: {}, message: {}", agent_name, message);

            let agent_manager = agent::agent_manager::AgentManager::new(PathBuf::from("sandbox"));
            let agent_definition = agent_manager.load_agent_definition(&agent_name).await?;
            let agent = agent_manager.create_agent(agent_definition)?;

            let response = agent.execute(&message).await?;
            println!("Agent {}: {}", agent.name(), response);
        }
    }

    Ok(())
}