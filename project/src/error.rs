use std::path::PathBuf;
use thiserror::Error;

use super::models::TaskState;

#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Task not found: {0}")]
    TaskNotFound(String),
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
}
