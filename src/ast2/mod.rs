use thiserror::Error;
use crate::ast2::parser::Parser;
use crate::ast2::parsing_logic::parse_content;
use crate::ast2::types::{Content, Position, Range};

pub mod types;
pub mod parser;
pub mod parsing_logic;

pub struct Document {
    pub content: Vec<Content>,
    pub range: Range,
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

pub fn parse_document(document: &str) -> Result<Document> {
    let parser = Parser::new(document);
    let begin = parser.get_position();
    
    let (content, parser_after_content) = parse_content(parser)?;
    
    let end = parser_after_content.get_position();

    Ok(Document {
        content: content,
        range: Range { begin, end }, 
    })
}

#[cfg(test)]
#[path = "tests/test_parse_document.rs"]
mod test_parse_document;

