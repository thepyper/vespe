use anyhow::Result;
use tracing_subscriber::{fmt, filter::EnvFilter, Layer, Registry};
use tracing_subscriber::prelude::*;
use tracing_appender::rolling::{RollingFileAppender, Rotation};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let file_appender = RollingFileAppender::new(Rotation::DAILY, ".", "vespe.log");
    let (non_blocking_appender, _guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = fmt::layer()
        .with_ansi(false)
        .with_writer(non_blocking_appender)
        .with_filter(EnvFilter::from_default_env());

    let stdout_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_filter(EnvFilter::from_default_env());

    Registry::default()
        .with(file_layer)
        .with(stdout_layer)
        .init();

    // Call the main run function from the library
    vespe::run().await
}
