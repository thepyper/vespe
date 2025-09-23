use clap::Parser;
use vespe::cli::commands::{Cli, Commands};
// use vespe::project_root::get_project_root_path; // Commented out
use project::utils::{find_project_root, initialize_project_root}; // New import
use vespe::statistics::models::UsageStatistics;
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

    // Handle Init command here, as it's now part of vespe_cli
    if let Commands::Init { path } = &cli.command {
        let target_dir = if let Some(p) = path {
            p.clone()
        } else {
            std::env::current_dir()? // Use current directory if no path is specified
        };
        initialize_project_root(&target_dir)?;
        println!("Vespe project initialized at: {}", target_dir.display());
        return Ok(());
    }

    let project_root = if let Some(path) = cli.project_root {
        path
    } else {
        find_project_root(&std::env::current_dir()?) // Use new find_project_root
            .ok_or_else(|| anyhow::anyhow!("Project root not found. Please run 'vespe init' or specify --project-root."))?
    };

    // Initialize statistics
    let stats = Arc::new(Mutex::new(UsageStatistics::load(&project_root).await?));

    vespe::run(project_root, cli.command, stats.clone()).await?;

    // Save statistics before exiting
    stats.lock().await.save(&project_root).await?;

    Ok(())
}
