use thiserror::Error;

use super::Position;

/// Represents all possible errors that can occur during the AST parsing phase.
///
/// This enum is designed with `thiserror` to provide rich, user-friendly error
/// messages. Most variants include a `Position` to indicate the exact location
/// of the error in the source document.
#[derive(Error, Debug)]
pub enum Ast2Error {
    /// A generic parsing error with a custom message.
    #[error("Parsing error at {position:?}: {message}")]
    ParsingError { position: Position, message: String },
    /// An error occurred while parsing a JSON value.
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
    /// An error occurred while parsing an integer value.
    #[error("Integer parsing error: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
    /// An error occurred while parsing a floating-point value.
    #[error("Float parsing error: {0}")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    /// The parser reached the end of the document unexpectedly.
    #[error("Unexpected end of document at {position:?}")]
    UnexpectedEndOfDocument { position: Position },
    /// The parser expected a specific character but found a different one (or none).
    #[error("Expected character '{expected}' but found '{found:?}' at {position:?}")]
    ExpectedChar {
        position: Position,
        expected: char,
        found: Option<char>,
    },
    /// The parser expected a specific string but found a different one (or none).
    #[error("Expected string '{expected}' but found '{found:?}' at {position:?}")]
    ExpectedString {
        position: Position,
        expected: String,
        found: Option<String>,
    },
    /// An unrecognized command was found.
    #[error("Invalid command kind at {position:?}")]
    InvalidCommandKind { position: Position },
    /// An unrecognized anchor kind (`begin` or `end`) was found.
    #[error("Invalid anchor kind at {position:?}")]
    InvalidAnchorKind { position: Position },
    /// A string could not be parsed as a valid UUID.
    #[error("Invalid UUID at {position:?}")]
    InvalidUuid { position: Position },
    /// A key was missing in a key-value parameter.
    #[error("Missing parameter key at {position:?}")]
    MissingParameterKey { position: Position },
    /// The colon separator was missing in a key-value parameter.
    #[error("Missing colon in parameter at {position:?}")]
    MissingParameterColon { position: Position },
    /// The value was missing in a key-value parameter.
    #[error("Missing parameter value at {position:?}")]
    MissingParameterValue { position: Position },
    /// A string literal was not closed with a matching quote.
    #[error("Unclosed string at {position:?}")]
    UnclosedString { position: Position },
    /// A value was malformed and could not be parsed.
    #[error("Malformed value at {position:?}")]
    MalformedValue { position: Position },
    /// A comma was expected between parameters but was not found.
    #[error("Missing comma in parameters at {position:?}")]
    MissingCommaInParameters { position: Position },
    /// A parameter was expected but could not be parsed.
    #[error("Parameter not parsed at {position:?}")]
    ParameterNotParsed { position: Position },
    /// A line was expected to start at the beginning, but had preceding characters.
    #[error("Expected beginning of line at {position:?}")]
    ExpectedBeginOfLine { position: Position },
    /// Unterminated object
    #[error("Unterminated object at {position:?}")]
    UnterminatedObject { position: Position },
    /// Unterminated array
    #[error("Unterminated array {position:?}")]
    UnterminatedArray { position: Position },
}

/// A specialized `Result` type for the AST parsing module.
pub type Result<T> = std::result::Result<T, Ast2Error>;
