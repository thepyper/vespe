use clap::Parser;
use crate::cli::commands::{Cli, Commands, ProjectSubcommand, TaskSubcommand, ToolSubcommand};
use crate::cli::resolve::{resolve_task, resolve_tool};
use vespe::api; // Import the api module
use vespe::utils::{find_project_root, initialize_project_root};
use std::fs;
use std::path::PathBuf;
use vespe::ProjectConfig;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

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
            match initialize_project_root(&target_dir) {
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
        path
    } else {
        find_project_root(&std::env::current_dir()?)
            .ok_or_else(|| anyhow::anyhow!("Project root not found. Please run 'vespe project init' or specify --project-root."))?
    };

    match &cli.command {
        Commands::Project(project_command) => match &project_command.command {
            ProjectSubcommand::Init { .. } => { /* Handled above */ }
            ProjectSubcommand::Info => {
                println!("Vespe Project Information");
                println!("-------------------------");
                println!("Root Path: {}", project_root.display());

                let task_count = api::list_all_tasks(&project_root).map_or(0, |t| t.len());
                println!("Task Count: {}", task_count);

                let tool_count = api::list_available_tools(&project_root, &ProjectConfig::default()).map_or(0, |t| t.len());
                println!("Tool Count: {}", tool_count);
            }
            ProjectSubcommand::Validate => {
                println!("Validating Vespe project...");
                let vespe_dir = project_root.join(".vespe");
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
            TaskSubcommand::Create { name, template, parent } => {
                // For now, created_by is hardcoded. This could be taken from config in the future.
                match api::create_task(&project_root, parent.clone(), name.clone(), "user".to_string(), template.clone()) {
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
                match resolve_task(&project_root, identifier) {
                    Ok(task) => {
                        println!("Task Details:");
                        println!("  UID: {}", task.uid);
                        println!("  Name: {}", task.config.name);
                        println!("  State: {:?}", task.status.current_state);
                        println!("  Created At: {}", task.config.created_at);
                        println!("  Objective: {}", task.objective);
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            TaskSubcommand::DefineObjective { identifier, objective } => {
                match api::define_objective(&project_root, identifier, objective.clone()) {
                    Ok(task) => {
                        println!("Objective defined for task {}. New state: {:?}", task.uid, task.status.current_state);
                    }
                    Err(e) => eprintln!("Error defining objective: {}", e),
                }
            }
            TaskSubcommand::DefinePlan { identifier, plan } => {
                match api::define_plan(&project_root, identifier, plan.clone()) {
                    Ok(task) => {
                        println!("Plan defined for task {}. New state: {:?}", task.uid, task.status.current_state);
                    }
                    Err(e) => eprintln!("Error defining plan: {}", e),
                }
            }
            TaskSubcommand::List => {
                match api::list_all_tasks(&project_root) {
                    Ok(tasks) => {
                        if tasks.is_empty() {
                            println!("No tasks found.");
                        } else {
                            println!("{:<38} {:<25} {:<20}", "UID", "Name", "State");
                            println!("{:-<38} {:-<25} {:-<20}", "", "", "");
                            for task in tasks {
                                println!("{:<38} {:<25} {:<20?}", task.uid, task.config.name, task.status.current_state);
                            }
                        }
                    }
                    Err(e) => eprintln!("Error listing tasks: {}", e),
                }
            },
            TaskSubcommand::Review { identifier, approve, reject, new_name } => {
                if *approve && *reject {
                    eprintln!("Error: Cannot approve and reject a task simultaneously.");
                    return Ok(());
                }
                if !*approve && !*reject {
                    eprintln!("Error: Must specify either --approve or --reject.");
                    return Ok(());
                }

                match resolve_task(&project_root, identifier) {
                    Ok(task) => {
                        if task.status.current_state != vespe::models::TaskState::NeedsReview {
                            eprintln!("Error: Task must be in 'NeedsReview' state to be reviewed. Current state: {:?}", task.status.current_state);
                            return Ok(());
                        }

                        if *approve {
                            match api::review_task(&project_root, &task.uid, true) {
                                Ok(updated_task) => {
                                    println!("Task {} approved. New state: {:?}", updated_task.uid, updated_task.status.current_state);
                                }
                                Err(e) => eprintln!("Error approving task: {}", e),
                            }
                        } else if *reject {
                            match api::review_task(&project_root, &task.uid, false) {
                                Ok(updated_task) => {
                                    println!("Task {} rejected. New state: {:?}", updated_task.uid, updated_task.status.current_state);
                                    // If rejected, create a new task for replanning
                                    if let Some(name) = new_name {
                                        match api::create_task(&project_root, Some(task.uid.clone()), name.clone(), "user".to_string(), "default".to_string()) {
                                            Ok(new_task) => {
                                                println!("New task created for replanning: {} (UID: {})", new_task.config.name, new_task.uid);
                                            }
                                            Err(e) => eprintln!("Error creating new task for replanning: {}", e),
                                        }
                                    } else {
                                        eprintln!("Warning: Task rejected, but no new name provided for replanning. A new task was not created.");
                                    }
                                }
                                Err(e) => eprintln!("Error rejecting task: {}", e),
                            }
                        }
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            },
            TaskSubcommand::Chat(chat_command) => {
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

                match api::create_tool(&project_root, name.clone(), description.clone(), schema_json, implementation_details) {
                    Ok(tool) => {
                        println!("Tool created successfully:");
                        println!("  UID: {}", tool.uid);
                        println!("  Name: {}", tool.config.name);
                    }
                    Err(e) => eprintln!("Error creating tool: {}", e),
                }
            }
            ToolSubcommand::Show { identifier } => {
                match resolve_tool(&project_root, identifier) {
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
                match api::list_available_tools(&project_root, &ProjectConfig::default()) {
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
