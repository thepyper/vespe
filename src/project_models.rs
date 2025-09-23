use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::error::ProjectError;
use crate::utils::{read_json_file, write_json_file};

// Constants for project root detection
const VESPE_DIR: &str = ".vespe";
const PROJECT_CONFIG_FILE: &str = "config.json";

// Corresponds to project_config.json
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectConfig {
    pub name: Option<String>,
    pub description: Option<String>,
    pub default_agent_uid: Option<String>,
    // pub manager_agent_config: Option<serde_json::Value>, // Future: specific config for Manager Agent
}

impl Default for ProjectConfig {
    fn default() -> Self {
        ProjectConfig {
            name: None,
            description: None,
            default_agent_uid: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Project {
    pub root_path: PathBuf,
    pub config: ProjectConfig,
}

impl Project {
    pub fn load(project_root_path: &Path) -> Result<Self, ProjectError> {
        let config_path = project_root_path.join(VESPE_DIR).join(PROJECT_CONFIG_FILE);
        let config = if config_path.exists() {
            read_json_file(&config_path)?
        } else {
            ProjectConfig::default()
        };

        Ok(Project {
            root_path: project_root_path.to_path_buf(),
            config,
        })
    }

    pub fn save_config(&self) -> Result<(), ProjectError> {
        let config_path = self.root_path.join(VESPE_DIR).join(PROJECT_CONFIG_FILE);
        write_json_file(&config_path, &self.config)
    }

    pub fn vespe_dir(&self) -> PathBuf {
        self.root_path.join(VESPE_DIR)
    }

    pub fn tasks_dir(&self) -> PathBuf {
        self.vespe_dir().join("tasks")
    }

    pub fn tools_dir(&self) -> PathBuf {
        self.vespe_dir().join("tools")
    }

    pub fn agents_dir(&self) -> PathBuf {
        self.vespe_dir().join("agents")
    }
}