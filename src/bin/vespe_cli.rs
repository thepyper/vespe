use clap::Parser;
use vespe::cli::commands::{Cli, Commands};
// use vespe::project_root::get_project_root_path; // Commented out
use project::utils::{find_project_root, initialize_project_root}; // New import
// use vespe::statistics; // Commented out
// use vespe::statistics::models::UsageStatistics; // Commented out
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing for logging
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let cli = Cli::parse();

    // Determine project_root early, before handling specific commands
    let project_root = if let Some(path) = cli.project_root.clone() { // Clone here to avoid move
        path
    } else {
        find_project_root(&std::env::current_dir()?) // Use new find_project_root
            .ok_or_else(|| anyhow::anyhow!("Project root not found. Please run 'vespe init' or specify --project-root."))?
    };

    // Handle Init command here, as it's now part of vespe_cli
    if let Commands::Init { path } = &cli.command {
        let target_dir = if let Some(p) = path {
            p.clone()
        } else {
            std::env::current_dir()? // Use current directory if no path is specified
        };
        match initialize_project_root(&target_dir) {
            Ok(_) => {
                println!("Vespe project initialized at: {}", target_dir.display());
            },
            Err(e) => {
                eprintln!("Error initializing project: {}", e);
            }
        }
        return Ok(()); // Still return Ok, as the error is printed
    }



            // // Initialize statistics (Commented out)
            // let stats = Arc::new(Mutex::new(statistics::load_statistics(&project_root).await?)); // Updated call

                    vespe::run(project_root.clone(), cli.command /*, stats.clone()*/).await?; // stats argument commented out
            // // Save statistics before exiting (Commented out)
            // statistics::save_statistics(&project_root, &*stats.lock().await).await?; // Updated call

        Ok(())
}
