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
// pub mod project_root; // Commented out

use crate::tools::tool_registry::ToolRegistry;
use crate::tools::impls::echo_tool::EchoTool;
use crate::tools::impls::read_file_tool::ReadFileTool;
// use crate::statistics::models::UsageStatistics; // Commented out
// use crate::statistics::STATS_FILE_NAME; // Commented out

use crate::prompt_templating::PromptTemplater;
use project::api as project_api; // Import project API
use crate::cli::commands::TaskCommands; // Import TaskCommands
use project::models::TaskState; // Import TaskState for listing

pub async fn run(project_root: PathBuf, command: cli::commands::Commands /*, stats: Arc<Mutex<UsageStatistics>>*/) -> Result<()> { // stats parameter commented out
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

            let agent_manager = agent::agent_manager::AgentManager::new(project_root, tool_registry, prompt_templater /*, stats*/)?;
            let agent_definition = agent_manager.load_agent_definition(&agent_name).await?;
            let agent = agent_manager.create_agent(&agent_definition)?;

            let response = agent.execute(&message).await?;
            println!("Agent {}: {}", agent.name(), response);
        },
        // cli::commands::Commands::Init { path } => { // Commented out
        //     let target_dir = if let Some(p) = path {
        //         p
        //     } else {
        //         std::env::current_dir()? // Use current directory if no path is specified
        //     };
        //     project::utils::initialize_project_root(&target_dir)?; // Updated call
        //     println!("Vespe project initialized at: {}", target_dir.display());
        // },
        cli::commands::Commands::Init { .. } => { /* Handled by vespe_cli.rs */ }, // Explicitly ignore Init command
        // cli::commands::Commands::ResetStats => { // Commented out
        //     let stats_path = project_root.join(".vespe").join(STATS_FILE_NAME);
        //     if stats_path.exists() {
        //         fs::remove_file(&stats_path).await?;
        //         println!("Statistics file deleted: {}", stats_path.display());
        //     } else {
        //         println!("No statistics file found at: {}. Nothing to reset.", stats_path.display());
        //     }
        // },
        cli::commands::Commands::Task { command } => {
            match command {
                TaskCommands::Create { parent_uid, name, created_by, template_name } => {
                    let task = project_api::create_task(&project_root, parent_uid, name, created_by, template_name)?;
                    println!("Task created: {} (UID: {})", task.config.name, task.uid);
                },
                TaskCommands::Show { uid } => {
                    let task = project_api::load_task(&project_root, &uid)?;
                    println!("Task Details for UID: {}", task.uid);
                    println!("  Name: {}", task.config.name);
                    println!("  State: {:?}", task.status.current_state);
                    println!("  Objective:\n{}", task.objective);
                    if let Some(plan) = task.plan {
                        println!("  Plan:\n{}", plan);
                    }
                    println!("  Created By: {}", task.config.created_by);
                    println!("  Created At: {}", task.config.created_at);
                },
                TaskCommands::DefineObjective { uid, content } => {
                    let task = project_api::define_objective(&project_root, &uid, content)?;
                    println!("Objective defined for task: {} (UID: {})", task.config.name, task.uid);
                },
                TaskCommands::DefinePlan { uid, content } => {
                    let task = project_api::define_plan(&project_root, &uid, content)?;
                    println!("Plan defined for task: {} (UID: {})", task.config.name, task.uid);
                },
                TaskCommands::List => {
                    // This will require a new function in project_api to list all tasks
                    // For now, let's just print a placeholder
                    println!("Listing all tasks (not yet implemented in project_api)");
                },
            }
        },
    }

    Ok(())
}