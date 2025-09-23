use clap::Parser;
use vespe::cli::commands::Cli;
use vespe::project_root::get_project_root_path;
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

    let project_root = if let Some(path) = cli.project_root {
        path
    } else {
        get_project_root_path()?;
        // If get_project_root_path() succeeds, it means .vespe exists somewhere up the hierarchy.
        // We need to return the actual root path found by get_project_root_path().
        // Let's re-call it to get the value.
        get_project_root_path()? 
    };

    // Initialize statistics
    let stats = Arc::new(Mutex::new(UsageStatistics::load(&project_root).await?));

    vespe::run(project_root, cli.command, stats.clone()).await?;

    // Save statistics before exiting
    stats.lock().await.save(&project_root).await?;

    Ok(())
}
