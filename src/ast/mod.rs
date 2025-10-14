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

pub struct Parser<'a> {
    document: &'a str,
    pub current_pos: Position,
}

impl<'a> Parser<'a> {
    pub fn new(document: &'a str) -> Self {
        Self {
            document,
            current_pos: Position { offset: 0, line: 1, column: 1 },
        }
    }

    pub fn peek(&self) -> Option<char> {
        self.document[self.current_pos.offset..].chars().next()
    }

    pub fn consume(&mut self) -> Option<char> {
        let mut chars = self.document[self.current_pos.offset..].chars();
        if let Some(c) = chars.next() {
            self.advance_position_by_char(c);
            Some(c)
        } else {
            None
        }
    }

    pub fn advance_position_by_char(&mut self, c: char) {
        self.current_pos.offset += c.len_utf8();
        if c == '\n' {
            self.current_pos.line += 1;
            self.current_pos.column = 1;
        } else {
            self.current_pos.column += 1;
        }
    }

    pub fn advance_position_by_str(&mut self, s: &str) {
        for c in s.chars() {
            self.advance_position_by_char(c);
        }
    }

    pub fn take_while<F>(&mut self, predicate: F) -> &'a str
    where
        F: Fn(char) -> bool,
    {
        let start_offset = self.current_pos.offset;
        while let Some(c) = self.peek() {
            if predicate(c) {
                self.consume();
            } else {
                break;
            }
        }
        &self.document[start_offset..self.current_pos.offset]
    }

    pub fn take_until<F>(&mut self, predicate: F) -> &'a str
    where
        F: Fn(char) -> bool,
    {
        let start_offset = self.current_pos.offset;
        while let Some(c) = self.peek() {
            if !predicate(c) {
                self.consume();
            } else {
                break;
            }
        }
        &self.document[start_offset..self.current_pos.offset]
    }

    pub fn take_until_and_consume<F>(&mut self, predicate: F) -> &'a str
    where
        F: Fn(char) -> bool,
    {
        let start_offset = self.current_pos.offset;
        while let Some(c) = self.peek() {
            if !predicate(c) {
                self.consume();
            } else {
                self.consume(); // Consume the character that satisfies the predicate
                break;
            }
        }
        &self.document[start_offset..self.current_pos.offset]
    }

    pub fn take_exact(&self, length: usize) -> Option<&'a str> {
        if self.current_pos.offset + length <= self.document.len() {
            Some(&self.document[self.current_pos.offset..self.current_pos.offset + length])
        } else {
            None
        }
    }

    pub fn take_exact_and_consume(&mut self, length: usize) -> Option<&'a str> {
        if self.current_pos.offset + length <= self.document.len() {
            let s = &self.document[self.current_pos.offset..self.current_pos.offset + length];
            self.advance_position_by_str(s);
            Some(s)
        } else {
            None
        }
    }

    pub fn skip_whitespace(&mut self) {
        self.take_while(|c| c.is_whitespace());
    }

    pub fn current_slice(&self) -> &'a str {
        &self.document[self.current_pos.offset..]
    }

    pub fn remaining_slice(&self) -> &'a str {
        &self.document[self.current_pos.offset..]
    }

    pub fn parse_quoted_string(&mut self, quote_char: char) -> Result<(String, Range), ParsingError> {
        let start_pos = self.current_pos;
        self.consume(); // Consume the opening quote

        let mut value = String::new();
        let mut escaped = false;

        loop {
            let current_char_pos = self.current_pos;
            match self.consume() {
                Some(c) if escaped => {
                    match c {
                        'n' => value.push('\n'),
                        'r' => value.push('\r'),
                        't' => value.push('\t'),
                        '\'' => value.push('\''),
                        '"' => value.push('\"'),
                        '\\' => value.push('\\'),
                        _ => return Err(ParsingError::InvalidSyntax {
                            message: format!("Invalid escape sequence: \\{}", c),
                            range: Range { start: current_char_pos, end: self.current_pos },
                        }),
                    }
                    escaped = false;
                },
                Some('\\') => {
                    escaped = true;
                },
                Some(c) if c == quote_char => {
                    let end_pos = self.current_pos;
                    return Ok((value, Range { start: start_pos, end: end_pos }));
                },
                Some(c) => {
                    value.push(c);
                },
                None => {
                    return Err(ParsingError::UnterminatedString {
                        range: Range { start: start_pos, end: self.current_pos },
                    });
                },
            }
        }
    }

    pub fn parse_unquoted_identifier(&mut self) -> Option<(String, Range)> {
        let start_pos = self.current_pos;
        let identifier_str = self.take_while(|c| c.is_alphanumeric() || c == '_');

        if identifier_str.is_empty() {
            None
        } else {
            let end_pos = self.current_pos;
            Some((identifier_str.to_string(), Range { start: start_pos, end: end_pos }))
        }
    }

    pub fn parse_number(&mut self) -> Result<Option<(ParameterValue, Range)>, ParsingError> {
        let start_pos = self.current_pos;
        let num_str = self.take_while(|c| c.is_ascii_digit() || c == '.' || c == '-');

        if num_str.is_empty() {
            return Ok(None);
        }

        let end_pos = self.current_pos;
        let range = Range { start: start_pos, end: end_pos };

        if num_str.contains('.') {
            match num_str.parse::<f64>() {
                Ok(f) => Ok(Some((ParameterValue::Float(f), range))),
                Err(_) => Err(ParsingError::InvalidNumberFormat { value: num_str.to_string(), range }),
            }
        } else {
            match num_str.parse::<i64>() {
                Ok(i) => Ok(Some((ParameterValue::Integer(i), range))),
                Err(_) => Err(ParsingError::InvalidNumberFormat { value: num_str.to_string(), range }),
            }
        }
    }

    pub fn parse_boolean(&mut self) -> Option<(ParameterValue, Range)> {
        let start_pos = self.current_pos;
        let bool_str = self.take_while(|c| c.is_alphabetic());

        let end_pos = self.current_pos;
        let range = Range { start: start_pos, end: end_pos };

        match bool_str {
            "true" => Some((ParameterValue::Boolean(true), range)),
            "false" => Some((ParameterValue::Boolean(false), range)),
            _ => None,
        }
    }
}

// Placeholder functions
pub fn parse(document: &str) -> Result<Root, ParsingError> {
    let mut parser = Parser::new(document);
    let start_position = parser.current_pos;

    // TODO: Implement the main parsing logic

    let end_position = Position { offset: document.len(), line: 1, column: document.len() + 1 }; // Placeholder
    let root_range = Range { start: start_position, end: end_position };

    Ok(Root {
        children: vec![],
        range: root_range,
    })
}

// Placeholder for parse_parameters
fn parse_parameters(parser: &mut Parser) -> Result<(Parameters, Range), ParsingError> {
    // For now, return empty parameters and advance position by 0
    let empty_params = HashMap::new();
    let range = Range { start: parser.current_pos, end: parser.current_pos };
    Ok((empty_params, range))
}

// Placeholder for parse_argument
fn parse_argument(parser: &mut Parser) -> Result<Option<(String, Range)>, ParsingError> {
    Ok(None)
}

// Placeholder for parse_arguments
fn parse_arguments(parser: &mut Parser) -> Result<(Vec<String>, Range), ParsingError> {
    let empty_args = vec![];
    let range = Range { start: parser.current_pos, end: parser.current_pos };
    Ok((empty_args, range))
}

// Placeholder for parse_tag
fn parse_tag(parser: &mut Parser) -> Result<Option<Tag>, ParsingError> {
    Ok(None)
}

// Placeholder for parse_anchor
fn parse_anchor(parser: &mut Parser) -> Result<Option<Anchor>, ParsingError> {
    Ok(None)
}

// Placeholder for parse_text
fn parse_text(parser: &mut Parser) -> Result<Option<Text>, ParsingError> {
    Ok(None)
}