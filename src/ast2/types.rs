use serde_json::json;
use std::str::Chars;
use std::str::FromStr;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    /// 0-based character offset
    pub offset: usize,
    /// 1-based line
    pub line: usize,
    /// 1-based column
    pub column: usize,
}

impl Position {
    pub fn null() -> Self {
        Position {
            offset: 0,
            line: 0,
            column: 0,
        }
    }
    pub fn is_valid(&self) -> bool {
        (self.line > 0) && (self.column > 0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Range {
    pub begin: Position,
    pub end: Position,
}

impl Range {
    pub fn null() -> Self {
        Range {
            begin: Position::null(),
            end: Position::null(),
        }
    }
    pub fn is_valid(&self) -> bool {
        if !self.begin.is_valid() {
            false
        } else if !self.end.is_valid() {
            false
        } else {
            self.begin.offset <= self.end.offset
        }
    }
}

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

pub struct Text {
    pub range: Range,
}

#[derive(Debug, PartialEq)]
pub enum CommandKind {
    Tag, // for debug purpose
    Include,
    Inline,
    Answer,
    Summarize,
    Derive,
    Repeat,
}

pub struct Parameters {
    pub parameters: serde_json::Map<String, serde_json::Value>,
    pub range: Range,
}

impl Parameters {
    pub fn new() -> Self {
        Parameters {
            parameters: serde_json::Map::new(),
            range: Range::null(),
        }
    }
}

pub struct Argument {
    pub value: String,
    pub range: Range,
}

pub struct Arguments {
    pub arguments: Vec<Argument>,
    pub range: Range,
}

pub struct Tag {
    pub command: CommandKind,
    pub parameters: Parameters,
    pub arguments: Arguments,
    pub range: Range,
}

#[derive(Debug, PartialEq)]
pub enum AnchorKind {
    Begin,
    End,
}

pub struct Anchor {
    pub command: CommandKind,
    pub uuid: Uuid,
    pub kind: AnchorKind,
    pub parameters: Parameters,
    pub arguments: Arguments,
    pub range: Range,
}

pub enum Content {
    Text(Text),
    Tag(Tag),
    Anchor(Anchor),
}

pub struct Document {
    pub content: Vec<Content>,
    pub range: Range,
}
