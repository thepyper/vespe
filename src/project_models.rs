use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::error::ProjectError;
use crate::utils::{read_json_file, write_json_file};
use crate::{Task, Tool};
use crate::api::{load_task, list_all_tasks, list_available_tools};
use anyhow::{anyhow, Result};

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

    /// Resolves a task identifier (which can be a UID or a name) to a Task.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The string identifier for the task (e.g., "tsk-...") or its name.
    ///
    /// # Returns
    ///
    /// A `Result` containing the resolved `Task` or an error if the task cannot be found
    /// or if the name is ambiguous.
    pub fn resolve_task(&self, identifier: &str) -> Result<Task> {
        // 1. Try to load directly as a UID.
        if identifier.starts_with("tsk-") {
            if let Ok(task) = load_task(self, identifier) {
                return Ok(task);
            }
        }

        // 2. If that fails or it\'s not a UID, search by name.
        let all_tasks = list_all_tasks(self)?;
        let matching_tasks: Vec<Task> = all_tasks
            .into_iter()
            .filter(|t| t.config.name == identifier)
            .collect();

        // 3. Check the search results.
        match matching_tasks.len() {
            0 => Err(anyhow!("Task \'{}\' not found.", identifier)),
            1 => Ok(matching_tasks.into_iter().next().unwrap()),
            _ => Err(anyhow!(
                "Multiple tasks found with the name \'{}\'. Please use a unique UID.",
                identifier
            )),
        }
    }

    /// Resolves a tool identifier (which can be a UID or a name) to a Tool.
    ///
    /// This function is a placeholder and needs to be adapted once `load_tool` by UID is available.
    pub fn resolve_tool(&self, identifier: &str) -> Result<Tool> {
        // For now, we only resolve by name as `load_tool` takes a path, not a UID.
        // This will be updated once a `load_tool_by_uid` function is available.

        let all_tools = list_available_tools(self, &ProjectConfig::default())?;
        let matching_tools: Vec<Tool> = all_tools
            .into_iter()
            .filter(|t| t.config.name == identifier || t.uid == identifier)
            .collect();

        match matching_tools.len() {
            0 => Err(anyhow!("Tool \'{}\' not found.", identifier)),
            1 => Ok(matching_tools.into_iter().next().unwrap()),
            _ => Err(anyhow!(
                "Multiple tools found with the name \'{}\'. Please use a unique UID.",
                identifier
            )),
        }
    }
}