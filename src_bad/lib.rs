pub mod agent;
pub mod ast2;
pub mod config;
pub mod constants;
pub mod editor;
pub mod error;
pub mod execute2;
pub mod utils;



pub mod project;

pub fn init_telemetry() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();
}
