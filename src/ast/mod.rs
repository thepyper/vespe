use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

// 2. Definizione delle Strutture Dati Core

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position {
    pub offset: usize, // 0-based byte offset from the start of the document
    pub line: usize,   // 1-based line number
    pub column: usize, // 1-based column number (character index on the line)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParameterValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
}

pub type Parameters = HashMap<String, ParameterValue>;

// 3. Implementazione della Gestione degli Errori (`ParsingError`)

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ParsingError {
    #[error("Unexpected token: expected '{expected}', found '{found}' at {range:?}")]
    UnexpectedToken {
        expected: String,
        found: String,
        range: Range,
    },
    #[error("Invalid syntax: {message} at {range:?}")]
    InvalidSyntax { message: String, range: Range },
    #[error("Unexpected end of file: expected '{expected}' at {range:?}")]
    EndOfFileUnexpected { expected: String, range: Range },
    #[error("Invalid number format: '{value}' at {range:?}")]
    InvalidNumberFormat { value: String, range: Range },
    #[error("Unterminated string at {range:?}")]
    UnterminatedString { range: Range },
    #[error("Custom parsing error: {message} at {range:?}")]
    Custom { message: String, range: Range },
}

// Placeholder for other structs that will be defined later
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Include,
    Inline,
    Answer,
    Derive,
    Summarize,
    Set,
    Repeat,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tag {
    pub command: Command,
    pub parameters: Parameters,
    pub arguments: Vec<String>, // Simplified for now, will be `Argument` structs later
    pub range: Range,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    Begin,
    End,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Anchor {
    pub command: Command,
    pub uuid: Uuid,
    pub kind: Kind,
    pub parameters: Parameters,
    pub arguments: Vec<String>, // Simplified for now, will be `Argument` structs later
    pub range: Range,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Text {
    pub content: String,
    pub range: Range,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Tag(Tag),
    Anchor(Anchor),
    Text(Text),
}

pub struct Root {
    pub children: Vec<Node>,
    pub range: Range,
}

// Placeholder functions
pub fn parse(document: &str) -> Result<Root, ParsingError> {
    // TODO: Implement the main parsing logic
    let start_position = Position { offset: 0, line: 1, column: 1 };
    let end_position = Position { offset: document.len(), line: 1, column: document.len() + 1 }; // Placeholder
    let root_range = Range { start: start_position, end: end_position };

    Ok(Root {
        children: vec![],
        range: root_range,
    })
}

// Helper to advance position
fn advance_position(current_pos: Position, text: &str) -> Position {
    let mut new_pos = current_pos;
    for char_code in text.chars() {
        new_pos.offset += char_code.len_utf8();
        if char_code == '\n' {
            new_pos.line += 1;
            new_pos.column = 1;
        } else {
            new_pos.column += 1;
        }
    }
    new_pos
}

// Placeholder for parse_parameters
fn parse_parameters(document: &str, current_pos: Position) -> Result<(Parameters, Range, Position), ParsingError> {
    // For now, return empty parameters and advance position by 0
    let empty_params = HashMap::new();
    let range = Range { start: current_pos, end: current_pos };
    Ok((empty_params, range, current_pos))
}

// Placeholder for parse_argument
fn parse_argument(document: &str, current_pos: Position) -> Result<Option<(String, Range, Position)>, ParsingError> {
    Ok(None)
}

// Placeholder for parse_arguments
fn parse_arguments(document: &str, current_pos: Position) -> Result<(Vec<String>, Range, Position), ParsingError> {
    let empty_args = vec![];
    let range = Range { start: current_pos, end: current_pos };
    Ok((empty_args, range, current_pos))
}

// Placeholder for parse_tag
fn parse_tag(document: &str, current_pos: Position) -> Result<Option<(Tag, Position)>, ParsingError> {
    Ok(None)
}

// Placeholder for parse_anchor
fn parse_anchor(document: &str, current_pos: Position) -> Result<Option<(Anchor, Position)>, ParsingError> {
    Ok(None)
}

// Placeholder for parse_text
fn parse_text(document: &str, current_pos: Position) -> Result<Option<(Text, Position)>, ParsingError> {
    Ok(None)
}