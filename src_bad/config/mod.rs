use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
    pub aux_paths: Vec<PathBuf>,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        ProjectConfig {
            editor_interface: EditorInterface::default(),
            git_integration_enabled: true, // Default to true for git integration
            aux_paths: Vec::new(),
        }
    }
}
