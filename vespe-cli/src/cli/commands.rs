use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// A fictional versioning CLI
#[derive(Debug, Parser)]
#[command(name = "vespe")]
#[command(about = "A fictional versioning CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    /// Sets the project root path.
    #[arg(long, global = true)]
    pub project_root: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Manage projects
    Project(ProjectCommand),
    /// Manage tasks
    Task(TaskCommand),
    /// Manage tools
    Tool(ToolCommand),
    /// Manage agents
    Agent(AgentCommand),
}

#[derive(Debug, Parser)]
pub struct AgentCommand {
    #[command(subcommand)]
    pub command: AgentSubcommand,
}

#[derive(Debug, Parser)]
pub struct LlmProviderArgs {
    #[command(subcommand)]
    pub provider: LlmProviderSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum LlmProviderSubcommand {
    Ollama {
        #[arg(long)]
        model: String,
        #[arg(long)]
        endpoint: String,
    },
    OpenAI {
        #[arg(long)]
        model: String,
        #[arg(long)]
        api_key_env: String,
    },
    Gemini {
        #[arg(long)]
        model: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum AgentSubcommand {
    /// Create a new AI agent
    CreateAI {
        #[arg(long)]
        name: String,
        #[arg(long)]
        role: String,
        #[clap(flatten)] // Use flatten to include LLM provider arguments
        llm_provider_args: LlmProviderArgs,
        #[arg(long, value_delimiter = ' ')] // Allows multiple values separated by space
        allowed_tools: Vec<String>,
        /// Path to the agent instructions file (.md)
        #[arg(long)]
        agent_instructions: Option<PathBuf>,
    },
    /// Create a new human agent
    CreateHuman {
        #[arg(long)]
        name: String,
        /// Path to the agent instructions file (.md)
        #[arg(long)]
        agent_instructions: Option<PathBuf>,
        // HumanConfig is empty for now, so no specific args needed
    },
    /// Show details for a specific agent
    Show {
        /// The UID or name of the agent
        identifier: String,
    },
    /// List all agents
    List,
    /// Set the default user agent for the project
    SetDefaultUser {
        /// The UID or name of the human agent
        agent_identifier: String,
    },
    /// Unset the default user agent for the project
    UnsetDefaultUser,
}

#[derive(Debug, Parser)]
pub struct ProjectCommand {
    #[command(subcommand)]
    pub command: ProjectSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum ProjectSubcommand {
    /// Initialize a new Vespe project
    Init {
        /// The path to initialize the project in. Defaults to the current directory.
        path: Option<PathBuf>,
    },
    /// Show project information
    Info,
    /// Validate the project structure and files
    Validate,
    /// Chat with a project-specific agent
    Chat(ChatCommand),
}

#[derive(Debug, Parser)]
pub struct TaskCommand {
    #[command(subcommand)]
    pub command: TaskSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum TaskSubcommand {
    /// Create a new task
    Create {
        #[arg(long)]
        name: String,
        #[arg(long)]
        agent_uid: String,
        #[arg(long)]
        parent: Option<String>,
    },
    /// Show details for a specific task
    Show {
        /// The UID or name of the task
        identifier: String,
    },
    /// Define the objective for a task
    DefineObjective {
        /// The UID or name of the task
        identifier: String,
        /// The objective content
        objective: String,
    },
    /// Define the plan for a task
    DefinePlan {
        /// The UID or name of the task
        identifier: String,
        /// The plan content
        plan: String,
    },
    /// List all tasks
    List,
    /// Accept the plan for a task
    AcceptPlan {
        /// The UID or name of the task
        identifier: String,
    },
    /// Reject the plan for a task
    RejectPlan {
        /// The UID or name of the task
        identifier: String,
    },
    /// Chat with a task-specific agent
    Chat(ChatCommand),
    /// Execute a single tick for a task
    Tick {
        /// The UID or name of the task
        identifier: String,
    },
    /// Assign an agent to a task
    Assign {
        /// The UID or name of the task
        #[arg(long)]
        task_identifier: String,
        /// The UID of the agent to assign
        #[arg(long)]
        agent_uid: String,
    },
    /// Unassign an agent from a task
    Unassign {
        /// The UID or name of the task
        #[arg(long)]
        task_identifier: String,
    },
}

#[derive(Debug, Parser)]
pub struct ToolCommand {
    #[command(subcommand)]
    pub command: ToolSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum ToolSubcommand {
    /// Create a new tool
    Create {
        #[arg(long)]
        name: String,
        #[arg(long)]
        description: String,
        /// Path to the JSON schema file for the tool
        #[arg(long)]
        schema: PathBuf,
    },
    /// Show details for a specific tool
    Show {
        /// The UID or name of the tool
        identifier: String,
    },
    /// List all available tools
    List,
    /// Chat with a tool-specific agent
    Chat(ChatCommand),
}

#[derive(Debug, Parser)]
pub struct ChatCommand {
    #[command(subcommand)]
    pub command: ChatSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum ChatSubcommand {
    /// Start a new chat session
    Start,
    /// Continue an existing chat session
    Continue {
        /// The ID of the chat session to continue
        #[arg(long)]
        session_id: String,
    },
}
