use anyhow::Result;
use tracing_subscriber::{fmt, filter::EnvFilter, Layer, Registry};
use tracing_subscriber::prelude::*;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use clap::Parser;
use std::path::PathBuf;

use vespe::cli::commands::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let project_root = PathBuf::from(cli.project_root);

    // Initialize logging
    let log_dir = project_root.join(".vespe").join("log");
    std::fs::create_dir_all(&log_dir)?;

    let file_appender = RollingFileAppender::new(Rotation::DAILY, log_dir, "vespe.log");
    let (non_blocking_appender, _guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = fmt::layer()
        .with_ansi(false)
        .with_writer(non_blocking_appender)
        .with_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()));

    let stdout_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()));

    Registry::default()
        .with(file_layer)
        .with(stdout_layer)
        .init();

    // Call the main run function from the library
    vespe::run(project_root).await
}