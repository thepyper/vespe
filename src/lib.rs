use anyhow::Result;
use std::path::PathBuf;
use tracing::info;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::fs; // Add this import

pub mod agent;
pub mod cli;
pub mod config;
pub mod llm;
pub mod tools;
pub mod statistics;

pub mod prompt_templating;
pub mod project_root;

use crate::tools::tool_registry::ToolRegistry;
use crate::tools::impls::echo_tool::EchoTool;
use crate::tools::impls::read_file_tool::ReadFileTool;
use crate::statistics::models::UsageStatistics;
use crate::statistics::STATS_FILE_NAME; // Import STATS_FILE_NAME

use crate::prompt_templating::PromptTemplater;

pub async fn run(project_root: PathBuf, command: cli::commands::Commands, stats: Arc<Mutex<UsageStatistics>>) -> Result<()> {
    println!("╔═══════════════════════════════════════════════════════╗");
    println!("║ Vespe - Version {}                                    ║", env!("CARGO_PKG_VERSION"));
    println!("║ Copyright (c) ThePyper                                  ║");
    println!("╚═══════════════════════════════════════════════════════╝");
    info!("Vespe application started.");

    let global_config = config::load_global_config().await?;
    let _final_config = config::load_project_config(global_config).await?;

    // Initialize PromptTemplater
    let prompt_templater = PromptTemplater::new(project_root.join(".vespe").join("prompts"))?;

    // Initialize ToolRegistry and register tools
    let mut tool_registry = ToolRegistry::new();
    tool_registry.register_tool(Arc::new(EchoTool));
    tool_registry.register_tool(Arc::new(ReadFileTool));

    match command {
        cli::commands::Commands::Chat { agent_name, message } => {
            info!("Chat command received for agent: {}, message: {}", agent_name, message);

            let agent_manager = agent::agent_manager::AgentManager::new(project_root, tool_registry, prompt_templater, stats)?;
            let agent_definition = agent_manager.load_agent_definition(&agent_name).await?;
            let agent = agent_manager.create_agent(agent_definition)?;

            let response = agent.execute(&message).await?;
            println!("Agent {}: {}", agent.name(), response);
        },
        cli::commands::Commands::Init { path } => {
            let target_dir = if let Some(p) = path {
                p
            } else {
                std::env::current_dir()? // Use current directory if no path is specified
            };
            project_root::initialize_project_root(&target_dir)?;
            println!("Vespe project initialized at: {}", target_dir.display());
        },
        cli::commands::Commands::ResetStats => {
            let stats_path = project_root.join(".vespe").join(STATS_FILE_NAME);
            if stats_path.exists() {
                fs::remove_file(&stats_path).await?;
                println!("Statistics file deleted: {}", stats_path.display());
            } else {
                println!("No statistics file found at: {}. Nothing to reset.", stats_path.display());
            }
        }
    }

    Ok(())
}