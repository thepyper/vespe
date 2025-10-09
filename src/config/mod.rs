use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::error::Result;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to load configuration: {0}")]
    LoadConfigFailed(String),
    #[error("Failed to save configuration: {0}")]
    SaveConfigFailed(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorInterface {
    /// No specific editor integration
    None,
    /// VSCode integration
    VSCode,
    // Add other editor integrations as needed
}

impl Default for EditorInterface {
    fn default() -> Self {
        EditorInterface::None
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub editor_interface: EditorInterface,
    pub git_integration_enabled: bool,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        ProjectConfig {
            editor_interface: EditorInterface::default(),
            git_integration_enabled: true, // Default to true for git integration
        }
    }
}
