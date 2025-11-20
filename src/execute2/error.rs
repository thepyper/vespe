//! This module defines the custom error types and a specialized `Result` alias
//! used throughout the `execute2` module. These errors encapsulate various issues
//! that can arise during the processing of directives, file operations, AST parsing,
//! and interactions with external components.
use thiserror::Error;

use uuid::Uuid;

use crate::ast2::{Ast2Error, CommandKind, Range};

/// Represents all possible errors that can occur during the execution phase.
///
/// This enum provides a comprehensive set of error variants, allowing for precise
/// error handling and reporting within the execution engine. Each variant is designed
/// to convey specific information about the nature of the failure.
#[derive(Error, Debug)]
pub enum ExecuteError {
    /// A generic execution error with a descriptive message.
    ///
    /// This variant is used for general errors that do not fit into more specific
    /// categories.
    #[error("Execution error: {0}")]
    Generic(String),

    /// An error originating from the Abstract Syntax Tree (AST) parsing phase.
    ///
    /// This indicates issues encountered while parsing the input document into
    /// an AST, typically due to malformed directives or syntax errors.
    #[error("AST error: {0}")]
    AstError(#[from] Ast2Error),

    /// An error related to file input/output operations.
    ///
    /// This can occur during reading or writing files, for example, when an
    /// `@include` tag references a non-existent file or when attempting to
    /// write to a protected location.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// An error encountered during JSON serialization or deserialization.
    ///
    /// This typically happens when reading or writing the state of dynamic
    /// tags (like `@answer` or `@repeat`) to/from their associated JSON files.
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Indicates that a context with the given name could not be found or resolved.
    ///
    /// This error occurs when an `@include` tag or similar directive references
    /// a context file that does not exist or cannot be located.
    #[error("Context '{0}' not found")]
    ContextNotFound(String),

    /// Signifies that an end anchor tag was not found for a corresponding start anchor.
    ///
    /// This typically points to a malformed document where an opening anchor
    /// (e.g., `<!-- @@answer... -->`) lacks its closing counterpart.
    #[error("End anchor not found for anchor starting at {0:?}")]
    EndAnchorNotFound(Uuid),

    /// Occurs when an attempt is made to pop an item from an empty anchor stack.
    ///
    /// This is an internal consistency error, suggesting a mismatch in how anchors
    /// are being managed during parsing or execution.
    #[error("Attempted to pop from an empty anchor stack at {0:?}")]
    EmptyAnchorStack(Range),

    /// Indicates that a required parameter for a tag or command was not provided.
    ///
    /// For example, an `@include` tag might require a `path` parameter that is missing.
    #[error("Missing parameter '{0}'")]
    MissingParameter(String),

    /// Indicates that a parameter was provided with an unsupported or invalid value.
    ///
    /// For instance, a parameter expecting a boolean might receive a non-boolean string.
    #[error("Unsupported value for parameter '{0}'")]
    UnsupportedParameterValue(String),

    /// Signifies that a command or tag is not recognized or supported by the execution engine.
    ///
    /// This can happen if an unknown tag is encountered in the input document.
    #[error("Unsupported command: {0:?}")]
    UnsupportedCommand(CommandKind),

    /// Indicates that a circular dependency was detected when resolving contexts or includes.
    ///
    /// This prevents infinite loops when, for example, file A includes file B, and file B
    /// attempts to include file A.
    #[error("Circular dependency detected in context: {0}")]
    CircularDependency(String),

    /// An error returned from a shell command execution.
    ///
    /// This captures failures when the execution engine attempts to run external
    /// shell commands, for example, as part of a custom tag's behavior.
    #[error("Shell call error: {0}")]
    ShellError(String),

    /// An error occurring during the resolution of a file system path.
    ///
    /// This can happen if a path is malformed, inaccessible, or cannot be converted
    /// to its canonical form.
    #[error("Path resolution error for '{path}': {source}")]
    PathResolutionError {
        path: String,
        #[source]
        source: anyhow::Error,
    },

    /// An error originating from the Abstract Syntax Tree (AST) parsing phase.
    ///
    /// This indicates issues encountered while parsing the input document into
    /// an AST, typically due to malformed directives or syntax errors.
    #[error("Handlebars render error: {0}")]
    RenderError(#[from] handlebars::RenderError),

    /// Indicates that a string could not be parsed into a status enum.
    #[error("Unsupported status: {0}")]
    UnsupportedStatus(String),

    /// Indicates that the `@include` tag is missing its required context name argument.
    #[error("Missing argument for '@include' tag at {range:?}")]
    MissingIncludeArgument { range: Range },

    /// Indicates that the `data` parameter for an `@include` tag is not a valid object.
    #[error("Unsupported 'data' parameter for '@include' tag at {range:?}, must be an object")]
    UnsupportedDataParameter { range: Range },

    /// Indicates that the `context` is missing from a { context: `file_name`, data: {`...data...`} } declaration
    #[error("Missing 'context' parameter at {range:?}")]
    MissingContextParameter { range: Range },

    /// Indicates that the context parameter has an unsupported value.
    #[error("Wrong 'context' parameter at {range:?}")]
    UnsupportedContextParameter { range: Range },

    /// Indicates that the execution of a context included via `@include` has failed.
    #[error("Execution of included context '{context}' failed at {range:?}")]
    IncludeExecutionFailed { context: String, range: Range },

    /// Indicates that the `prefix_data` parameter has an unsupported value.
    #[error("Unsupported 'prefix_data' parameter at {range:?}, must be an object")]
    UnsupportedPrefixData { range: Range },

    /// Indicates that the `prefix` parameter has an unsupported value.
    #[error("Unsupported 'prefix' parameter at {range:?}, must be a string")]
    UnsupportedPrefix { range: Range },

    /// Indicates that the `postfix_data` parameter has an unsupported value.
    #[error("Unsupported 'postfix_data' parameter at {range:?}, must be an object")]
    UnsupportedPostfixData { range: Range },

    /// Indicates that the `postfix` parameter has an unsupported value.
    #[error("Unsupported 'postfix' parameter at {range:?}, must be a string")]
    UnsupportedPostfix { range: Range },

    /// Indicates that the `provider` parameter has an unsupported value.
    #[error("Unsupported 'provider' parameter at {range:?}, must be a string")]
    UnsupportedProvider { range: Range },

    /// Indicates that the required `provider` parameter is missing.
    #[error("Missing 'provider' parameter at {range:?}")]
    MissingProvider { range: Range },

    /// Indicates that the `output` parameter has an unsupported value.
    #[error("Unsupported 'output' parameter at {range:?}, must be a string")]
    UnsupportedOutput { range: Range },

    /// Indicates that the `input_data` parameter has an unsupported value.
    #[error("Unsupported 'input_data' parameter at {range:?}, must be an object")]
    UnsupportedInputData { range: Range },

    /// Indicates that the `input` parameter has an unsupported value.
    #[error("Unsupported 'input' parameter at {range:?}, must be a string")]
    UnsupportedInput { range: Range },

    /// Indicates that the `@inline` tag is missing its required context name argument.
    #[error("Missing argument for '@inline' tag at {range:?}")]
    MissingInlineArgument { range: Range },

    /// Indicates that the `@repeat` tag is not inside a repeatable anchor.
    #[error("'@repeat' must be used inside a repeatable anchor at {range:?}")]
    RepeatNotAllowed { range: Range },

    /// Indicates that the `@repeat` tag is not inside any anchor.
    #[error("'@repeat' must be used inside an anchor at {range:?}")]
    RepeatNotInAnchor { range: Range },

    /// Indicates that the `@done` tag is being used as an anchor, which is not allowed.
    #[error("'@done' tag cannot be an anchor at {range:?}")]
    DoneTagAsAnchor { range: Range },

    /// Indicates that the `@done` tag is not inside a `@task` anchor.
    #[error("'@done' must be used inside a '@task' anchor at {range:?}")]
    DoneTagOutsideTask { range: Range },

    /// Indicates that the `choice` parameter has an unsupported value.
    #[error(
        "Unsupported 'choice' parameter at {range:?}, must be a string or an array of strings"
    )]
    UnsupportedChoice { range: Range },

    /// Indicates that the required `choice` parameter is missing.
    #[error("Missing 'choice' parameter at {range:?}")]
    MissingChoice { range: Range },

    /// An error originating from the utility module.
    #[error("Utility error: {0}")]
    UtilError(#[from] crate::utils::Error),

    /// A catch-all error for any `anyhow::Error` that occurs.
    ///
    /// This provides a convenient way to propagate errors from libraries that
    /// use `anyhow` for their error handling.
    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),
}

/// A specialized `Result` type for the execution module.
///
/// This type is used as the return type for most functions within the `execute2`
/// module, simplifying error handling by consistently using `ExecuteError`.
pub type Result<T> = std::result::Result<T, ExecuteError>;
