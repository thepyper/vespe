use anyhow::Result;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    fmt().with_env_filter(EnvFilter::from_default_env()).init();

    // Call the main run function from the library
    vespe::run().await
}