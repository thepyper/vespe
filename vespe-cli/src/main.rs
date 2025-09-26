mod cli;

use clap::Parser;
use crate::cli::commands::{Cli, Commands, ProjectSubcommand, TaskSubcommand, ToolSubcommand};
 // Import the api module
use vespe::project::Project;
use vespe::TaskState;
use std::fs;
use tracing_subscriber::{EnvFilter, FmtSubscriber};
use tracing::{info, debug, error}; // New import

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing for logging
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let cli = Cli::parse();

    // The `init` command is special as it might be run outside of a project root.
    if let Commands::Project(project_command) = &cli.command {
        if let ProjectSubcommand::Init { path } = &project_command.command {
            let target_dir = path.clone().unwrap_or(std::env::current_dir()?);
            match Project::initialize(&target_dir) {
                Ok(_) => {
                    println!("Vespe project initialized at: {}", target_dir.display());
                },
                Err(e) => {
                    eprintln!("Error initializing project: {}", e);
                }
            }
            return Ok(());
        }
    }

    // For all other commands, we need to be inside a project root.
    let project_root = if let Some(path) = cli.project_root {
        vespe::Project::load(&path)?
    } else {
        Project::find_root(&std::env::current_dir()?)
            .ok_or_else(|| anyhow::anyhow!("Project root not found. Please run 'vespe project init' or specify --project-root."))?
    };

    match &cli.command {
        Commands::Project(project_command) => match &project_command.command {
            ProjectSubcommand::Init { .. } => { /* Handled above */ }
            ProjectSubcommand::Info => {
                println!("Vespe Project Information");
                println!("-------------------------");
                println!("Root Path: {}", project_root.root_path.display());

                let task_count = project_root.list_all_tasks().map_or(0, |t| t.len());
                println!("Task Count: {}", task_count);

                let tool_count = project_root.list_available_tools().map_or(0, |t| t.len());
                println!("Tool Count: {}", tool_count);
            }
            ProjectSubcommand::Validate => {
                println!("Validating Vespe project...");
                let vespe_dir = project_root.root_path.join(".vespe");
                let tasks_dir = vespe_dir.join("tasks");
                let tools_dir = vespe_dir.join("tools");

                let mut is_valid = true;
                if !vespe_dir.exists() {
                    eprintln!("Error: .vespe directory not found.");
                    is_valid = false;
                }
                if !tasks_dir.exists() {
                    eprintln!("Error: .vespe/tasks directory not found.");
                    is_valid = false;
                }
                if !tools_dir.exists() {
                    eprintln!("Error: .vespe/tools directory not found.");
                    is_valid = false;
                }

                if is_valid {
                    println!("Project structure is valid.");
                } else {
                    println!("Project structure is invalid.");
                }
            },
            ProjectSubcommand::Chat(chat_command) => {
                println!("Project Chat command called: {:?}", chat_command);
            }
        },
        Commands::Task(task_command) => match &task_command.command {
            TaskSubcommand::Create { name, agent_uid, parent } => {
                // For now, created_by is hardcoded. This could be taken from config in the future.
                match project_root.create_task(parent.clone(), name.clone(), agent_uid.clone(), "".to_string()) {
                    Ok(task) => {
                        println!("Task created successfully:");
                        println!("  UID: {}", task.uid);
                        println!("  Name: {}", task.config.name);
                        println!("  State: {:?}", task.status.current_state);
                    }
                    Err(e) => eprintln!("Error creating task: {}", e),
                }
            }
            TaskSubcommand::Show { identifier } => {
                match project_root.resolve_task(identifier) {
                    Ok(task) => {
                        println!("Task Details:");
                        println!("  UID: {}", task.uid);
                        println!("  Name: {}", task.config.name);
                        println!("  State: {:?}", task.status.current_state);
                        println!("  Paused: {}", task.status.is_paused);
                        if let Some(details) = task.status.error_details {
                            println!("  Error Details: {}", details);
                        }
                        if let Some(prev_state) = task.status.previous_state {
                            println!("  Previous State: {:?}", prev_state);
                        }
                        println!("  Retry Count: {}", task.status.retry_count);
                        println!("  Created At: {}", task.config.created_at);
                        println!("  Objective: {}", task.objective);
                        if let Some(plan_content) = task.plan {
                            println!("  Plan: {}", plan_content);
                        }
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            TaskSubcommand::DefineObjective { identifier, objective } => {
                match project_root.define_objective(identifier, objective.clone()) {
                    Ok(_) => {
                        println!("Objective defined for task {}.", identifier);
                    }
                    Err(e) => eprintln!("Error defining objective: {}", e),
                }
            }
            TaskSubcommand::DefinePlan { identifier, plan } => {
                match project_root.define_plan(identifier, plan.clone()) {
                    Ok(_) => {
                        println!("Plan defined for task {}.", identifier);
                    }
                    Err(e) => eprintln!("Error defining plan: {}", e),
                }
            }
            TaskSubcommand::List => {
                match project_root.list_all_tasks() {
                    Ok(tasks) => {
                        if tasks.is_empty() {
                            println!("No tasks found.");
                        } else {
                            println!("{:<38} {:<25} {:<20} {:<10}", "UID", "Name", "State", "Paused");
                            println!("{:-<38} {:-<25} {:-<20} {:-<10}", "", "", "", "");
                            for task in tasks {
                                println!("{:<38} {:<25} {:<20?} {:<10}",
                                    task.uid,
                                    task.config.name,
                                    task.status.current_state,
                                    task.status.is_paused
                                );
                            }
                        }
                    }
                    Err(e) => eprintln!("Error listing tasks: {}", e),
                }
            },
            TaskSubcommand::AcceptPlan { identifier } => {
                match project_root.accept_plan(identifier) {
                    Ok(_) => {
                        println!("Plan accepted for task {}.", identifier);
                    }
                    Err(e) => eprintln!("Error accepting plan: {}", e),
                }
            },
            TaskSubcommand::RejectPlan { identifier } => {
                match project_root.reject_plan(identifier) {
                    Ok(_) => {
                        println!("Plan rejected for task {}.", identifier);
                    }
                    Err(e) => eprintln!("Error rejecting plan: {}", e),
                }
            },            TaskSubcommand::Chat(chat_command) => {
                println!("Task Chat command called: {:?}", chat_command);
            }
        },
        Commands::Tool(tool_command) => match &tool_command.command {
            ToolSubcommand::Create { name, description, schema } => {
                let schema_content = match fs::read_to_string(schema) {
                    Ok(content) => content,
                    Err(e) => {
                        eprintln!("Error reading schema file: {}", e);
                        return Ok(()); // Exit gracefully
                    }
                };
                let schema_json: serde_json::Value = match serde_json::from_str(&schema_content) {
                    Ok(json) => json,
                    Err(e) => {
                        eprintln!("Error parsing schema JSON: {}", e);
                        return Ok(()); // Exit gracefully
                    }
                };

                // Implementation_details is hardcoded for now
                let implementation_details = serde_json::json!({ "type": "command_line" });

                match project_root.create_tool(name.clone(), description.clone(), schema_json, implementation_details) {
                    Ok(tool) => {
                        println!("Tool created successfully:");
                        println!("  UID: {}", tool.uid);
                        println!("  Name: {}", tool.config.name);
                    }
                    Err(e) => eprintln!("Error creating tool: {}", e),
                }
            }
            ToolSubcommand::Show { identifier } => {
                match project_root.resolve_tool(identifier) {
                    Ok(tool) => {
                        println!("Tool Details:");
                        println!("  UID: {}", tool.uid);
                        println!("  Name: {}", tool.config.name);
                        println!("  Description: {}", tool.config.description);
                        println!("  Schema: {:#}", tool.config.schema);
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            ToolSubcommand::List => {
                match project_root.list_available_tools() {
                    Ok(tools) => {
                        if tools.is_empty() {
                            println!("No tools found.");
                        } else {
                            println!("{:<38} {:<25}", "UID", "Name");
                            println!("{:-<38} {:-<25}", "", "");
                            for tool in tools {
                                println!("{:<38} {:<25}", tool.uid, tool.config.name);
                            }
                        }
                    }
                    Err(e) => eprintln!("Error listing tools: {}", e),
                }
            },
            ToolSubcommand::Chat(chat_command) => {
                println!("Tool Chat command called: {:?}", chat_command);
            }
        },
    }

    Ok(())
}