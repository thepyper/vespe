pub mod agent;
pub mod ast2;
pub mod config;
pub mod editor;
pub mod execute2;
pub mod git;
pub mod project;
pub mod utils;
pub mod file;
pub mod path;
pub mod constants;

pub fn init_telemetry() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
}
