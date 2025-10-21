use clap::builder::Str;
use serde_json::json;
use std::str::Chars;
use std::str::FromStr;
use thiserror::Error;
use uuid::Uuid;

use super::{Position};

#[derive(Error, Debug)]
pub enum Ast2Error {
    #[error("Parsing error at {position:?}: {message}")]
    ParsingError { position: Position, message: String },
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Integer parsing error: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("Float parsing error: {0}")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("Unexpected end of document at {position:?}")]
    UnexpectedEndOfDocument { position: Position },
    #[error("Expected character '{expected}' but found '{found:?}' at {position:?}")]
    ExpectedChar {
        position: Position,
        expected: char,
        found: Option<char>,
    },
    #[error("Expected string '{expected}' but found '{found:?}' at {position:?}")]
    ExpectedString {
        position: Position,
        expected: String,
        found: Option<String>,
    },
    #[error("Invalid command kind at {position:?}")]
    InvalidCommandKind { position: Position },
    #[error("Invalid anchor kind at {position:?}")]
    InvalidAnchorKind { position: Position },
    #[error("Invalid UUID at {position:?}")]
    InvalidUuid { position: Position },
    #[error("Missing parameter key at {position:?}")]
    MissingParameterKey { position: Position },
    #[error("Missing colon in parameter at {position:?}")]
    MissingParameterColon { position: Position },
    #[error("Missing parameter value at {position:?}")]
    MissingParameterValue { position: Position },
    #[error("Unclosed string at {position:?}")]
    UnclosedString { position: Position },
    #[error("Malformed value at {position:?}")]
    MalformedValue { position: Position },
    #[error("Missing comma in parameters at {position:?}")]
    MissingCommaInParameters { position: Position },
    #[error("Parameter not parsed at {position:?}")]
    ParameterNotParsed { position: Position },
    #[error("Expected beginning of line at {position:?}")]
    ExpectedBeginOfLine { position: Position },
}

pub type Result<T> = std::result::Result<T, Ast2Error>;