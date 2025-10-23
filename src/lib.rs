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

const CTX_DIR_NAME: &str = ".ctx";
const CTX_ROOT_FILE_NAME: &str = ".ctx_root";
const METADATA_DIR_NAME: &str = ".meta";
const CONTEXTS_DIR_NAME: &str = "contexts";

pub fn init_telemetry() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
}
