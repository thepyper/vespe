use serde::{Serialize, Deserialize};
use std::path::{Path, PathBuf};
use crate::error::ProjectError;
use crate::utils::{read_json_file, write_json_file};
use crate::task::Task;
use crate::agent::{Agent, AIConfig, HumanConfig};
use crate::tool::Tool;
use crate::memory::{Message, MessageContent};
use crate::error::AgentTickResult;
use anyhow::{anyhow, Result};
use tracing::debug;
use crate::registry::Registry;

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
    pub default_user_agent_uid: Option<String>,
    // pub manager_agent_config: Option<serde_json::Value>, // Future: specific config for Manager Agent
}

impl Default for ProjectConfig {
    fn default() -> Self {
        ProjectConfig {
            name: None,
            description: None,
            default_agent_uid: None,
            default_user_agent_uid: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Project {
    pub root_path: PathBuf,
    pub config: ProjectConfig,
}

impl Project {
    pub fn initialize(target_dir: &Path) -> Result<Project, ProjectError> {
        // Create the target directory if it doesn't exist
        std::fs::create_dir_all(target_dir).map_err(|e| ProjectError::Io(e))?;

        let absolute_target_dir = target_dir.canonicalize()
            .map_err(|_e| ProjectError::InvalidPath(target_dir.to_path_buf()))?;

        // Check if target_dir is already part of an existing Vespe project
        if let Some(found_project) = Project::find_root(&absolute_target_dir) {
            return Err(ProjectError::InvalidProjectConfig(format!(
                "Cannot initialize a Vespe project inside an existing project. Existing root: {}",
                found_project.root_path.display()
            )));
        }

        let vespe_dir = absolute_target_dir.join(VESPE_DIR);
        let vespe_root_marker = vespe_dir.join(VESPE_ROOT_MARKER);

        std::fs::create_dir_all(&vespe_dir).map_err(|e| ProjectError::Io(e))?;

        std::fs::write(&vespe_root_marker, "Feel The BuZZ!!!!").map_err(|e| ProjectError::Io(e))?;

        let vespe_gitignore = vespe_dir.join(".gitignore");
        std::fs::write(&vespe_gitignore, "log/").map_err(|e| ProjectError::Io(e))?;

        // Create a default ProjectConfig and save it
        let project_config = ProjectConfig::default();
        let project = Project {
            root_path: absolute_target_dir.clone(),
            config: project_config,
        };
        project.save_config()?;

        // Create tasks, tools, agents directories
        std::fs::create_dir_all(project.tasks_dir()).map_err(|e| ProjectError::Io(e))?;
        std::fs::create_dir_all(project.tools_dir()).map_err(|e| ProjectError::Io(e))?;
        std::fs::create_dir_all(project.agents_dir()).map_err(|e| ProjectError::Io(e))?;

        Ok(project)
    }

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

    pub fn log_dir(&self) -> PathBuf {
        self.vespe_dir().join("log")
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
                    match self.load_task(uid_str) {
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
            0 => Err(anyhow!("Task '{}' not found.", identifier.to_string())),
            1 => Ok(matching_tasks.into_iter().next().unwrap()),
            _ => Err(anyhow!(
                "Multiple tasks found with the name '{}'. Please use a unique UID.",
                identifier.to_string()
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
            0 => Err(anyhow!("Tool '{}' not found.", identifier.to_string())),
            1 => Ok(matching_tools.into_iter().next().unwrap()),
            _ => Err(anyhow!(
                "Multiple tools found with the name '{}'. Please use a unique UID.",
                identifier.to_string()
            )),
        }
    }

    /// Resolves an agent identifier (which can be a UID or a name) to an Agent.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The string identifier for the agent (e.g., "agt-...") or its name.
    ///
    /// # Returns
    ///
    /// A `Result` containing the resolved `Agent` or an error if the agent cannot be found
    /// or if the name is ambiguous.
    pub fn resolve_agent(&self, identifier: &str) -> Result<Agent> {
        // 1. Try to load directly as a UID.
        if identifier.starts_with("agt-") || identifier.starts_with("usr-") {
            if let Ok(agent) = self.load_agent(identifier) {
                return Ok(agent);
            }
        }

        // 2. If that fails or it's not a UID, search by name.
        let all_agents = self.list_agents()?;
        let matching_agents: Vec<Agent> = all_agents
            .into_iter()
            .filter(|a| a.metadata.name == identifier)
            .collect();

        // 3. Check the search results.
        match matching_agents.len() {
            0 => Err(anyhow!("Agent '{}' not found.", identifier.to_string())),
            1 => Ok(matching_agents.into_iter().next().unwrap()),
            _ => Err(anyhow!(
                "Multiple agents found with the name '{}'. Please use a unique UID.",
                identifier.to_string()
            )),
        }
    }

    /// Resolves a human agent by identifier, or returns the default human agent.
    pub fn resolve_human_agent_or_default(
        &self,
        identifier: Option<&str>,
    ) -> Result<Agent, ProjectError> {
        // 1. Attempt to resolve the provided identifier first.
        if let Some(id) = identifier {
            match self.resolve_agent(id) {
                Ok(agent) => {
                    if matches!(agent.details, crate::agent::AgentDetails::Human(_)) {
                        return Ok(agent); // Found a human agent with the given ID.
                    } else {
                        // Found an agent, but it's not human. Return specific error.
                        return Err(ProjectError::NotHumanAgent(id.to_string()));
                    }
                }
                Err(_) => {
                    // Agent not found by the given identifier, fall through to default.
                    eprintln!("Warning: Agent '{}' not found. Falling back to default.", id);
                }
            }
        }

        // 2. If no identifier was provided, or if it wasn't found, use the default.
        let default_uid = self
            .config
            .default_user_agent_uid
            .as_ref()
            .ok_or_else(|| {
                ProjectError::InvalidProjectConfig(
                    "No identifier provided and no default human agent is set.".to_string(),
                )
            })?;

        let default_agent = self.load_agent(default_uid)?;

        if matches!(default_agent.details, crate::agent::AgentDetails::Human(_)) {
            Ok(default_agent)
        } else {
            Err(ProjectError::InvalidProjectConfig(
                "The configured default user agent is not a human agent.".to_string(),
            ))
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
                    match self.load_tool(uid_str) {
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
        Task::create(&self.root_path, parent_uid, name, created_by_agent_uid, _template_name)
    }



    /// Loads a task from the filesystem given its UID.
    pub fn load_task(
        &self,
        uid: &str
    ) -> Result<Task, ProjectError> {
        Task::load(&self.root_path, uid)
    }




    /// Transitions from `CREATED` to `OBJECTIVE_DEFINED`.
    /// Writes the objective content to `objective.md`.
    pub fn define_objective(
        &self,
        task_uid: &str,
        objective_content: String
    ) -> Result<(), ProjectError> {
        let mut task = self.resolve_task(task_uid)?;
        task.define_objective(objective_content)?;
        Ok(())
    }

    /// Transitions from `OBJECTIVE_DEFINED` to `PLAN_DEFINED`.
    /// Writes the plan content to `plan.md`.
    pub fn define_plan(
        &self,
        task_uid: &str,
        plan_content: String,
    ) -> Result<(), ProjectError> {
        let mut task = self.resolve_task(task_uid)?;
        task.define_plan(plan_content)?;
        Ok(())
    }

    pub fn accept_plan(&self, task_uid: &str) -> Result<(), ProjectError> {
        let mut task = self.resolve_task(task_uid)?;
        task.accept_plan()?;
        Ok(())
    }

    pub fn reject_plan(&self, task_uid: &str) -> Result<(), ProjectError> {
        let mut task = self.resolve_task(task_uid)?;
        task.reject_plan()?;
        Ok(())
    }



    pub fn error(&self, task_uid: &str, details: String) -> Result<(), ProjectError> {
        let mut task = self.resolve_task(task_uid)?;
        task.error(details)?;
        Ok(())
    }

    pub fn failure(&self, task_uid: &str, details: String) -> Result<(), ProjectError> {
        let mut task = self.resolve_task(task_uid)?;
        task.failure(details)?;
        Ok(())
    }

    pub fn work_completed(&self, task_uid: &str) -> Result<(), ProjectError> {
        let mut task = self.resolve_task(task_uid)?;
        task.work_completed()?;
        Ok(())
    }

    pub fn abort(&self, task_uid: &str, reason: String) -> Result<(), ProjectError> {
        let mut task = self.resolve_task(task_uid)?;
        task.abort(reason)?;
        Ok(())
    }



    pub fn get_task_state(&self, task_uid: &str) -> Result<crate::task::TaskState, ProjectError> {
        let task = self.resolve_task(task_uid)?;
        Ok(task.get_task_state())
    }

    pub fn is_task_paused(&self, task_uid: &str) -> Result<bool, ProjectError> {
        let task = self.resolve_task(task_uid)?;
        Ok(task.is_task_paused())
    }

    pub fn set_task_name(&self, task_uid: &str, new_name: String) -> Result<(), ProjectError> {
        let mut task = self.resolve_task(task_uid)?;
        task.set_name(new_name)?;
        Ok(())
    }



    /// Calculates the SHA256 hash of the `result/` folder content for a task.
    pub fn calculate_result_hash(
        &self,
        task_uid: &str
    ) -> Result<String, ProjectError> {
        let task = self.resolve_task(task_uid)?;
        task.calculate_result_hash()
    }

    /// Adds a file to the `result/` folder of the task.
    pub fn add_result_file(
        &self,
        task_uid: &str,
        filename: &str,
        content: Vec<u8>
    ) -> Result<(), ProjectError> {
        let task = self.resolve_task(task_uid)?;
        task.add_result_file(filename, content)?;
        Ok(())
    }

    /// Creates a new tool.
    pub fn create_tool(
        &self,
        name: String,
        description: String,
        schema: serde_json::Value,
        _implementation_details: serde_json::Value,
    ) -> Result<Tool, ProjectError> {
        let tools_base_path = self.tools_dir();
        Tool::create(name, description, schema, &tools_base_path)
    }

    /// Loads a tool from the filesystem given its UID.
    pub fn load_tool(
        &self,
        uid: &str
    ) -> Result<Tool, ProjectError> {
        let tool_path = self.tools_dir().join(uid);
        crate::Tool::from_path(&tool_path)
    }

    /// Creates a new AI agent.
    pub fn create_ai_agent(
        &self,
        name: String,
        config: AIConfig,
        agent_instructions: Option<String>,
    ) -> Result<Agent, ProjectError> {
        Agent::create_ai(&self.root_path, name, config, agent_instructions, None)
    }

    /// Creates a new human agent.
    pub fn create_human_agent(
        &self,
        name: String,
        config: HumanConfig,
        agent_instructions: Option<String>,
    ) -> Result<Agent, ProjectError> {
        Agent::create_human(&self.root_path, name, config, agent_instructions, None)
    }

    /// Loads an agent from the filesystem given its UID.
    pub fn load_agent(
        &self,
        agent_uid: &str,
    ) -> Result<Agent, ProjectError> {
        Agent::load(&self.root_path, agent_uid)
    }

    /// Lists all agents available in the project.
    pub fn list_agents(
        &self,
    ) -> Result<Vec<Agent>, ProjectError> {
        let agents_base_path = self.agents_dir();
        let mut agents = Vec::new();

        if !agents_base_path.exists() {
            return Ok(agents);
        }

        for entry in std::fs::read_dir(agents_base_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if let Some(uid_str) = path.file_name().and_then(|s| s.to_str()) {
                    match self.load_agent(uid_str) {
                        Ok(agent) => agents.push(agent),
                        Err(e) => eprintln!("Warning: Could not load agent {}: {}", uid_str, e),
                    }
                }
            }
        }

        Ok(agents)
    }

    /// Saves an agent's state to its state.json file.
    pub fn save_agent_state(
        &self,
        agent: &Agent,
    ) -> Result<(), ProjectError> {
        agent.save_state(&self.root_path)
    }


    /// Assigns a default user agent for the project.
    pub fn assign_default_user_agent(&mut self, agent_uid: &str) -> Result<(), ProjectError> {
        let agent = self.resolve_agent(agent_uid)?;
        if !matches!(agent.details, crate::agent::AgentDetails::Human(_)) {
            return Err(ProjectError::NotHumanAgent(agent_uid.to_string()));
        }
        self.config.default_user_agent_uid = Some(agent_uid.to_string());
        self.save_config()
    }

    /// Unassigns the default user agent for the project.
    pub fn unassign_default_user_agent(&mut self) -> Result<(), ProjectError> {
        self.config.default_user_agent_uid = None;
        self.save_config()
    }
    /// Assigns a task to an agent. This modifies the Task's status.
    pub fn assign_task_to_agent(
        &self,
        task_uid: &str,
        agent_uid: &str,
    ) -> Result<(), ProjectError> {
        let mut task = self.resolve_task(task_uid)?;
        // Check if agent exists
        let agent = self.resolve_agent(agent_uid)?;

        task.assign_agent(&agent.metadata.uid)
    }

    /// Unassigns an agent from a task. This modifies the Task's status.
    pub fn unassign_agent_from_task(
        &self,
        task_uid: &str,
    ) -> Result<(), ProjectError> {
        let mut task = self.resolve_task(task_uid)?;
        task.unassign_agent()
    }

    /// Adds a message to an agent's memory.
    pub fn add_message_to_agent_memory(
        &self,
        agent_uid: &str,
        author_uid: &str,
        content: MessageContent,
    ) -> Result<Message, ProjectError> {
        let mut agent = self.resolve_agent(agent_uid)?;
        let message = agent.memory.add_message(author_uid.to_string(), content).map_err(|e| ProjectError::Memory(e))?;
        Ok(message.clone())
    }

    /// Adds a message to a task's memory.
    pub fn add_message_to_task_memory(
        &self,
        task_uid: &str,
        author_uid: &str,
        content: MessageContent,
    ) -> Result<Message, ProjectError> {
        let mut task = self.resolve_task(task_uid)?;
        let message = task.memory.add_message(author_uid.to_string(), content).map_err(|e| ProjectError::Memory(e))?;
        Ok(message.clone())
    }

    /// Executes a single reasoning cycle for a given task, using the assigned agent.
    pub async fn tick_task(
        &self,
        task_uid: &str,
    ) -> Result<AgentTickResult, ProjectError> {
        let task = self.resolve_task(task_uid)?;
        let agent_uid = task.status.assigned_agent_uid.ok_or_else(|| ProjectError::InvalidOperation(format!("Task {} has no agent assigned.", task_uid)))?;
        let agent = self.resolve_agent(&agent_uid)?;

        // 1. Check for Objective
        if task.objective.is_empty() {
            return Err(ProjectError::InvalidOperation(format!("Task {} has no objective defined. Cannot tick.", task_uid)));
        }

        // 2. Determine step_objective based on task state
        let step_objective = match task.status.current_state {
            crate::task::TaskState::Created => "Define the objective for the task.",
            crate::task::TaskState::ObjectiveDefined => "Create a plan to achieve the objective.",
            crate::task::TaskState::PlanDefined => "Execute the plan.",
            crate::task::TaskState::Working => "Continue executing the plan or take the next step.",
            _ => return Err(ProjectError::InvalidOperation(format!("Task {} is in an untickable state: {:?}", task_uid, task.status.current_state))),
        };

        let system_instructions = format!(
            "You are an AI agent working on task '{}'.\nObjective: {}\nStep Objective: {}",
            task.config.name,
            task.objective,
            step_objective
        );

        let ai_config = match &agent.details {
            crate::agent::AgentDetails::AI(config) => config,
            crate::agent::AgentDetails::Human(_) => return Err(ProjectError::InvalidOperation("Cannot call LLM for a Human agent.".to_string())),
        };

        let allowed_tool_names = &ai_config.allowed_tools;
        let mut available_tools_for_protocol = Vec::new();
        let tool_registry = &crate::registry::TOOL_REGISTRY;

        for tool_name in allowed_tool_names {
            if let Some(tool_config) = tool_registry.get(tool_name) {
                available_tools_for_protocol.push(tool_config.config.clone());
            } else {
                eprintln!("Warning: Allowed tool '{}' not found in TOOL_REGISTRY.", tool_name);
            }
        }

        let task_context_messages: Vec<Message> = task.memory.get_context().into_iter().cloned().collect();

        debug!("Ticking task {} with agent {}. Agent details: {:?}", task_uid, agent_uid, agent.details);
        debug!("System Instructions for LLM: {}", system_instructions);

        // 3. Call the LLM
        let _llm_response_messages = agent.call_llm(
            &self.root_path,
            &task_context_messages,
            &available_tools_for_protocol,
            Some(&system_instructions),
        ).await?;

        // For now, just return Waiting. The actual processing of llm_response_messages will come later.
        Ok(AgentTickResult::Waiting)
    }
}