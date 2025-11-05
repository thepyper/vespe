use thiserror::Error;
use std::path::PathBuf;
use uuid::Uuid;

use crate::ast2::{Ast2Error, CommandKind, Range};

/// Represents all possible errors that can occur during the execution phase.
#[derive(Error, Debug)]
pub enum ExecuteError {
    /// Generic error message.
    #[error("Execution error: {0}")]
    Generic(String),

    /// Error originating from the AST parsing phase.
    #[error("AST error: {0}")]
    AstError(#[from] Ast2Error),

    /// Error related to file I/O.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Error during JSON serialization or deserialization.
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// A context with the given name could not be found or resolved.
    #[error("Context '{0}' not found")]
    ContextNotFound(String),

    /// An anchor's end tag was not found.
    #[error("End anchor not found for anchor starting at {0:?}")]
    EndAnchorNotFound(Uuid),

    /// Attempted to pop from an empty anchor stack.
    #[error("Attempted to pop from an empty anchor stack at {0:?}")]
    EmptyAnchorStack(Range),

    /// A required parameter was missing.
    #[error("Missing parameter '{0}'")]
    MissingParameter(String),

    /// A parameter had an unsupported or invalid value.
    #[error("Unsupported value for parameter '{0}'")]
    UnsupportedParameterValue(String),

    /// A command is not supported by the execution engine.
    #[error("Unsupported command: {0:?}")]
    UnsupportedCommand(CommandKind),

    /// Circular dependency detected when resolving contexts.
    #[error("Circular dependency detected in context: {0}")]
    CircularDependency(String),

    /// Error from the shell call
    #[error("Shell call error: {0}")]
    ShellError(String),

    /// Error resolving path
    #[error("Path resolution error for '{path}': {source}")]
    PathResolutionError {
        path: String,
        #[source]
        source: anyhow::Error,
    },

    /// Anyhow error
    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),
}

/// A specialized `Result` type for the execution module.
pub type Result<T> = std::result::Result<T, ExecuteError>;
