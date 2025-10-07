pub mod syntax;
pub mod execute;
pub mod project;
pub mod semantic;
pub mod utils;
pub mod agent;

pub fn init_telemetry() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
}