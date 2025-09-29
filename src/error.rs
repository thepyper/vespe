use std::path::PathBuf;
use thiserror::Error;

use crate::task::TaskState;
use crate::memory::MemoryError;


#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Task not found: {0}")]
    TaskNotFound(String),
    #[error("Agent not found: {0}")]
    AgentNotFound(String),
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
    #[error("Invalid project configuration: {0}")]
    InvalidProjectConfig(String),
    #[error("Invalid state transition: from {0:?} to {1:?}")]
    InvalidStateTransition(TaskState, TaskState),
    #[error("Task is in an unexpected state: {0:?}")]
    UnexpectedState(TaskState),
    #[error("Missing required file for task: {0}")]
    MissingRequiredFile(PathBuf),
    #[error("Dependency cycle detected involving task: {0}")]
    DependencyCycle(String),
    #[error("Failed to calculate content hash for {0}: {1}")]
    ContentHashError(PathBuf, String),
    #[error("Failed to generate UID: {0}")]
    UidGenerationError(String),
    #[error("Invalid path: {0}")]
    InvalidPath(PathBuf),
    #[error("Project root not found. Looked in {0}")]
    ProjectRootNotFound(PathBuf),
    #[error("Subtask not found: {0}")]
    SubtaskNotFound(String),
    #[error("Memory error: {0}")]
    Memory(#[from] MemoryError),
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    #[error("LLM client error: {0}")]
    LLMClientError(String),
    #[error("Invalid tool call: {0}")]
    InvalidToolCall(String),
    #[error("Agent protocol not found: {0}")]
    AgentProtocolNotFound(String),
    #[error("Tool execution error: {0}")]
    ToolExecutionError(String),
    #[error("Agent protocol error: {0}")]
    AgentProtocol(#[from] crate::agent_protocol::AgentProtocolError),
    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),

}

// Rappresenta il risultato di un ciclo di `tick`
pub enum AgentTickResult {
    MadeProgress { thought: String },
    TaskCompleted,
    SubtasksCreated(Vec<String>), // Vec<task_uid>
    Waiting,
    Error(String),
}
