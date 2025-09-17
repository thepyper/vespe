use anyhow::Result;
use std::path::PathBuf;
use tracing::info;
use clap::Parser;
use std::sync::Arc;

pub mod agent;
pub mod cli;
pub mod config;
pub mod llm;
pub mod tools;

pub mod prompt_templating;
pub mod project_root;

use crate::tools::tool_registry::ToolRegistry;
use crate::tools::impls::echo_tool::EchoTool;
use crate::tools::impls::read_file_tool::ReadFileTool;

use crate::prompt_templating::PromptTemplater;

pub async fn run(project_root: PathBuf) -> Result<()> {
    println!("╔═══════════════════════════════════════════════════════╗");
    println!("║ Vespe - Version {}                                    ║", env!("CARGO_PKG_VERSION"));
    println!("║ Copyright (c) ThePyper                                  ║");
    println!("╚═══════════════════════════════════════════════════════╝");
    info!("Vespe application started.");

    let cli = cli::commands::Cli::parse();

    let global_config = config::load_global_config().await?;
    let _final_config = config::load_project_config(global_config).await?;

    // Initialize PromptTemplater
    let prompt_templater = PromptTemplater::new(project_root.join(".vespe").join("prompts"))?;

    // Initialize ToolRegistry and register tools
    let mut tool_registry = ToolRegistry::new();
    tool_registry.register_tool(Arc::new(EchoTool));
    tool_registry.register_tool(Arc::new(ReadFileTool));

    match cli.command {
        cli::commands::Commands::Chat { agent_name, message } => {
            info!("Chat command received for agent: {}, message: {}", agent_name, message);

            let agent_manager = agent::agent_manager::AgentManager::new(project_root, tool_registry, prompt_templater)?;
            let agent_definition = agent_manager.load_agent_definition(&agent_name).await?;
            let agent = agent_manager.create_agent(agent_definition)?;

            let response = agent.execute(&message).await?;
            println!("Agent {}: {}", agent.name(), response);
        }
    }

    Ok(())
}