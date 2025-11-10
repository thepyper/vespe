pub mod agent;
pub mod ast2;
pub mod config;
pub mod constants;
pub mod editor;
pub mod execute2;
pub mod file;
pub mod git;
pub mod path;
pub mod project;

pub fn init_telemetry() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_writer(std::io::stderr)
        .init();
}
