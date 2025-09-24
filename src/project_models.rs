use serde::{Serialize, Deserialize};
use std::path::{Path, PathBuf};
use crate::error::ProjectError;
use crate::utils::{read_json_file, write_json_file};
use crate::{Task, TaskConfig, TaskDependencies, TaskState, TaskStatus, Tool};
use crate::api::{load_tool};
use crate::utils::{generate_uid, get_entity_path, read_file_content, read_json_file, write_file_content, write_json_file};
use anyhow::{anyhow, Result};
use chrono::Utc;

// Constants for project root detection
pub const VESPE_DIR: &str = ".vespe";
pub const PROJECT_CONFIG_FILE: &str = "config.json";
pub const VESPE_ROOT_MARKER: &str = ".vespe_root";

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

    /// Lists all tasks in the project.
    pub fn list_all_tasks(&self) -> Result<Vec<Task>, ProjectError> {
        let tasks_base_path = self.tasks_dir();
        let mut tasks = Vec::new();

        if !tasks_base_path.exists() {
            return Ok(tasks);
        }

        for entry in std::fs::read_dir(tasks_base_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if let Some(uid_str) = path.file_name().and_then(|s| s.to_str()) {
                    match load_task(self, uid_str) {
                        Ok(task) => tasks.push(task),
                        Err(e) => eprintln!("Warning: Could not load task {}: {}", uid_str, e),
                    }
                }
            }
        }

        Ok(tasks)
    }

    /// Checks if a given directory is a Vespe project root by looking for the .vespe/.vespe_root marker file.
    pub fn is_root(dir: &Path) -> bool {
        dir.join(VESPE_DIR).join(VESPE_ROOT_MARKER).exists()
    }

    /// Finds the project root by traversing up the directory tree until a .vespe/ directory is found.
    pub fn find_root(start_dir: &Path) -> Option<Project> {
        let mut current_dir = Some(start_dir);

        while let Some(dir) = current_dir {
            if Self::is_root(dir) {
                return Some(Project::load(dir).ok()?);
            }
            current_dir = dir.parent();
        }
        None
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
            if let Ok(task) = self.load_task(identifier) {
                return Ok(task);
            }
        }

        // 2. If that fails or it's not a UID, search by name.
        let all_tasks = self.list_all_tasks()?;
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

    /// or if the name is ambiguous.
    pub fn resolve_tool(&self, identifier: &str) -> Result<Tool> {
        let all_tools = self.list_available_tools()?;
        let matching_tools: Vec<Tool> = all_tools
            .into_iter()
            .filter(|t| t.config.name == identifier || t.uid == identifier)
            .collect();

        match matching_tools.len() {
            0 => Err(anyhow!("Tool '{}' not found.", identifier)),
            1 => Ok(matching_tools.into_iter().next().unwrap()),
            _ => Err(anyhow!(
                "Multiple tools found with the name '{}'. Please use a unique UID.",
                identifier
            )),
        }
    }

    /// Lists all tools available for the project.
    /// This method corrects a bug from the previous implementation where it looked in `tasks_dir` instead of `tools_dir`.
    pub fn list_available_tools(&self) -> Result<Vec<Tool>, ProjectError> {
        let tools_base_path = self.tools_dir();
        let mut tools = Vec::new();
        if !tools_base_path.exists() {
            return Ok(tools);
        }

        for entry in std::fs::read_dir(tools_base_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if let Some(uid_str) = path.file_name().and_then(|s| s.to_str()) {
                    match load_tool(&path) {
                        Ok(tool) => tools.push(tool),
                        Err(e) => eprintln!("Warning: Could not load tool {}: {}", uid_str, e),
                    }
                }
            }
        }
        Ok(tools)
    }

    /// Creates a new task or subtask.
    /// Initializes the task directory with config.json, empty objective.md, etc.
    /// The task is created in the `CREATED` state.
    pub fn create_task(
        &self,
        parent_uid: Option<String>,
        name: String,
        created_by_agent_uid: String,
        _template_name: String, // Template not yet implemented, ignored for now
    ) -> Result<Task, ProjectError> {
        let uid = generate_uid("tsk")?;
        let tasks_base_path = self.tasks_dir();
        let task_path = get_entity_path(&tasks_base_path, &uid)?;

        // Create task directory and subdirectories
        std::fs::create_dir_all(&task_path).map_err(|e| ProjectError::Io(e))?;
        std::fs::create_dir_all(task_path.join("persistent")).map_err(|e| ProjectError::Io(e))?;
        std::fs::create_dir_all(task_path.join("result")).map_err(|e| ProjectError::Io(e))?;

        let now = Utc::now();

        // Initialize config.json
        let config = TaskConfig {
            uid: uid.clone(),
            name: name.clone(),
            created_by_agent_uid: created_by_agent_uid.clone(),
            created_at: now,
            parent_uid,
        };
        write_json_file(&task_path.join("config.json"), &config)?;

        // Initialize status.json
        let status = TaskStatus {
            current_state: TaskState::Created,
            last_updated_at: now,
            progress: None,
            parent_content_hashes: std::collections::HashMap::new(),
        };
        write_json_file(&task_path.join("status.json"), &status)?;

        // Create empty objective.md and plan.md
        write_file_content(&task_path.join("objective.md"), "")?;
        write_file_content(&task_path.join("plan.md"), "")?;

        // Initialize dependencies.json
        let dependencies = TaskDependencies { depends_on: Vec::new() };
        write_json_file(&task_path.join("dependencies.json"), &dependencies)?;

        // Load the newly created task to return it
        self.load_task(&uid)
    }

    /// Loads a task from the filesystem given its UID.
    pub fn load_task(
        &self,
        uid: &str
    ) -> Result<Task, ProjectError> {
        let tasks_base_path = self.tasks_dir();
        let task_path = get_entity_path(&tasks_base_path, uid)?;

        if !task_path.exists() {
            return Err(ProjectError::TaskNotFound(uid.to_string()));
        }

        let config: TaskConfig = read_json_file(&task_path.join("config.json"))?;
        let status: TaskStatus = read_json_file(&task_path.join("status.json"))?;
        let dependencies: TaskDependencies = read_json_file(&task_path.join("dependencies.json"))?;
        let objective = read_file_content(&task_path.join("objective.md"))?;
        let plan = Some(read_file_content(&task_path.join("plan.md"))?);

        Ok(Task {
            uid: uid.to_string(),
            root_path: task_path,
            config,
            status,
            objective,
            plan,
            dependencies,
        })
    }
}