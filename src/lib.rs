pub mod syntax;
pub mod execute;
pub mod project;
pub mod semantic;
pub mod utils;
pub mod agent;

pub fn init_telemetry() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
}