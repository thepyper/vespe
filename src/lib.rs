pub mod agent;
pub mod config;
pub mod editor;
pub mod execute;
pub mod project;
pub mod semantic;
pub mod syntax;
pub mod utils;
pub mod git;

pub fn init_telemetry() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
}
