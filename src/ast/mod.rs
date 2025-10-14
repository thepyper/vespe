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
    #[error("Unexpected token: expected '{{expected}}', found '{{found}}' at {{range:?}}")]
    UnexpectedToken {
        expected: String,
        found: String,
        range: Range,
    },
    #[error("Invalid syntax: {{message}} at {{range:?}}")]
    InvalidSyntax {
        message: String,
        range: Range,
    },
    #[error("Unexpected end of file: expected '{{expected}}' at {{range:?}}")]
    EndOfFileUnexpected {
        expected: String,
        range: Range,
    },
    #[error("Invalid number format: '{{value}}' at {{range:?}}")]
    InvalidNumberFormat {
        value: String,
        range: Range,
    },
    #[error("Unterminated string at {{range:?}}")]
    UnterminatedString {
        range: Range,
    },
    #[error("Custom parsing error: {{message}} at {{range:?}}")]
    Custom {
        message: String,
        range: Range,
    },
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
                        '"' => value.push('"'),
                        '\\' => value.push('\\'),
                        _ => return Err(ParsingError::InvalidSyntax {
                            message: format!("Invalid escape sequence: {}", c),
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

pub fn parse(document: &str) -> Result<Root, ParsingError> {
    dbg!("Entering parse function");
    let mut parser = Parser::new(document);
    let start_position = parser.current_pos;
    let mut children = Vec::new();

    while parser.peek().is_some() {
        dbg!("Parse loop iteration", &parser.current_pos);
        let current_offset = parser.current_pos.offset;
        parser.skip_whitespace(); // Skip whitespace between nodes
        if parser.current_pos.offset == document.len() {
            // Reached end of document after skipping whitespace
            break;
        }

        match parse_node(&mut parser)? {
            Some(node) => {
                dbg!("Node parsed successfully", &node);
                children.push(node);
            },
            None => {
                // If no node was parsed, and we haven't advanced, it's an infinite loop or unexpected content
                if parser.current_pos.offset == current_offset {
                    dbg!("Parser stuck or no node parsed, breaking loop");
                    break; // Break to avoid infinite loop if no progress is made
                }
            }
        }
    }

    let end_position = parser.current_pos;
    let root_range = Range { start: start_position, end: end_position };

    dbg!("Exiting parse function");
    Ok(Root {
        children,
        range: root_range,
    })
}

fn parse_node(parser: &mut Parser) -> Result<Option<Node>, ParsingError> {
    dbg!("Entering parse_node", &parser.current_pos);
    let start_pos = parser.current_pos;

    dbg!("Attempting to parse tag");
    if let Some(tag) = parse_tag(parser)? {
        dbg!("Tag parsed successfully");
        return Ok(Some(Node::Tag(tag)));
    }

    // Reset position if tag parsing failed without consuming anything
    parser.current_pos = start_pos;
    dbg!("Attempting to parse anchor");
    if let Some(anchor) = parse_anchor(parser)? {
        dbg!("Anchor parsed successfully");
        return Ok(Some(Node::Anchor(anchor)));
    }

    // Reset position if anchor parsing failed without consuming anything
    parser.current_pos = start_pos;
    dbg!("Attempting to parse text");
    if let Some(text) = parse_text(parser)? {
        dbg!("Text parsed successfully");
        return Ok(Some(Node::Text(text)));
    }

    dbg!("No node parsed, returning None");
    Ok(None)
}

fn parse_parameters(parser: &mut Parser) -> Result<(Parameters, Range), ParsingError> {
    let start_pos = parser.current_pos;
    parser.skip_whitespace();

    if parser.peek() != Some('{') {
        let end_pos = parser.current_pos;
        return Ok((HashMap::new(), Range { start: start_pos, end: end_pos }));
    }

    parser.consume(); // Consume '{'
    let mut params = HashMap::new();

    loop {
        parser.skip_whitespace();
        let key_start_pos = parser.current_pos;

        let key = match parser.peek() {
            Some('\'') | Some('"') => {
                let (k, _) = parser.parse_quoted_string(parser.peek().unwrap())?;
                k
            },
            Some(c) if c.is_alphanumeric() || c == '_' => {
                let (k, _) = parser.parse_unquoted_identifier().ok_or_else(|| ParsingError::InvalidSyntax {
                    message: "Expected parameter key".to_string(),
                    range: Range { start: key_start_pos, end: parser.current_pos },
                })?;
                k
            },
            Some('}') => break, // End of parameters
            _ => return Err(ParsingError::InvalidSyntax {
                message: "Expected parameter key or '}'".to_string(),
                range: Range { start: key_start_pos, end: parser.current_pos },
            }),
        };

        parser.skip_whitespace();
        if parser.consume() != Some(':') {
            return Err(ParsingError::UnexpectedToken {
                expected: ":".to_string(),
                found: parser.peek().map_or("EOF".to_string(), |c| c.to_string()),
                range: Range { start: parser.current_pos, end: parser.current_pos },
            });
        }

        parser.skip_whitespace();
        let value_start_pos = parser.current_pos;
        let value_and_range = match parser.peek() {
            Some('\'') | Some('"') => {
                let (s, r) = parser.parse_quoted_string(parser.peek().unwrap())?;
                (ParameterValue::String(s), r)
            },
            Some(c) if c.is_ascii_digit() || c == '-' => {
                let (v, r) = parser.parse_number()?.ok_or_else(|| ParsingError::InvalidSyntax {
                    message: "Expected number value".to_string(),
                    range: Range { start: value_start_pos, end: parser.current_pos },
                })?;
                (v, r)
            },
            Some(c) if c.is_alphabetic() => {
                let current_slice = parser.current_slice();
                if current_slice.starts_with("true") || current_slice.starts_with("false") {
                    let (v, r) = parser.parse_boolean().ok_or_else(|| ParsingError::InvalidSyntax {
                        message: "Expected boolean value".to_string(),
                        range: Range { start: value_start_pos, end: parser.current_pos },
                    })?;
                    (v, r)
                } else {
                    let (s, r) = parser.parse_unquoted_identifier().ok_or_else(|| ParsingError::InvalidSyntax {
                        message: "Expected string value".to_string(),
                        range: Range { start: value_start_pos, end: parser.current_pos },
                    })?;
                    (ParameterValue::String(s), r)
                }
            },
            _ => return Err(ParsingError::InvalidSyntax {
                message: "Expected parameter value".to_string(),
                range: Range { start: value_start_pos, end: parser.current_pos },
            }),
        };
        params.insert(key, value_and_range.0);

        parser.skip_whitespace();
        match parser.peek() {
            Some(',') => {
                parser.consume();
            },
            Some('}') => break,
            _ => return Err(ParsingError::UnexpectedToken {
                expected: ",".to_string(),
                found: parser.peek().map_or("EOF".to_string(), |c| c.to_string()),
                range: Range { start: parser.current_pos, end: parser.current_pos },
            }),
        }
    }

    parser.consume(); // Consume '}'
    let end_pos = parser.current_pos;
    Ok((params, Range { start: start_pos, end: end_pos }))
}

fn parse_argument(parser: &mut Parser) -> Result<Option<(String, Range)>, ParsingError> {
    parser.skip_whitespace();
    let start_pos = parser.current_pos;

    let arg_value = match parser.peek() {
        Some('\'') | Some('"') => {
            let (s, r) = parser.parse_quoted_string(parser.peek().unwrap())?;
            Some((s, r))
        },
        Some(c) if !c.is_whitespace() && c != '{' && c != '}' && c != ',' && c != '-' && c != '<' => {
            let word_start_pos = parser.current_pos;
            let word = parser.take_while(|c| !c.is_whitespace() && c != '{' && c != '}' && c != ',' && c != '-' && c != '<');
            if word.is_empty() {
                None
            } else {
                let end_pos = parser.current_pos;
                Some((word.to_string(), Range { start: word_start_pos, end: end_pos }))
            }
        },
        _ => None,
    };

    Ok(arg_value)
}

pub fn parse_arguments(parser: &mut Parser) -> Result<(Vec<String>, Range), ParsingError> {
    let start_pos = parser.current_pos;
    let mut args = Vec::new();

    loop {
        parser.skip_whitespace();
        let current_line_start_offset = parser.document[..parser.current_pos.offset].rfind('\n').map_or(0, |i| i + 1);
        let current_line_slice = &parser.document[current_line_start_offset..];

        // Check for start of a new tag or anchor, or end of line/file
        if (current_line_slice.starts_with("@") && parser.current_pos.column == 1) || (current_line_slice.starts_with("<!--") && parser.current_pos.column == 1) || parser.peek().is_none() {
            break;
        }

        match parse_argument(parser)? {
            Some((arg, _)) => {
                args.push(arg);
            },
            None => {
                // No more arguments found, or an empty string was parsed (which shouldn't happen with current parse_argument logic)
                break;
            },
        }
    }

    let end_pos = parser.current_pos;
    Ok((args, Range { start: start_pos, end: end_pos }))
}

pub fn parse_tag(parser: &mut Parser) -> Result<Option<Tag>, ParsingError> {
    dbg!("Entering parse_tag", &parser.current_pos);
    let start_pos = parser.current_pos;

    // Tags must start at the beginning of a line and begin with '@'
    if start_pos.column != 1 || parser.peek() != Some('@') {
        dbg!("parse_tag: Not a tag or not at column 1");
        return Ok(None);
    }

    let line_start_offset = parser.document[..start_pos.offset].rfind('\n').map_or(0, |i| i + 1);
    let line_slice = &parser.document[line_start_offset..];

    let tag_regex = regex::Regex::new(r"^@([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();

    if let Some(captures) = tag_regex.captures(line_slice) {
        dbg!("parse_tag: Tag regex matched");
        let command_str = captures.get(1).unwrap().as_str();
        let command = match command_str {
            "include" => Command::Include,
            "inline" => Command::Inline,
            "answer" => Command::Answer,
            "derive" => Command::Derive,
            "summarize" => Command::Summarize,
            "set" => Command::Set,
            "repeat" => Command::Repeat,
            _ => return Err(ParsingError::InvalidSyntax {
                message: format!("Unknown command: {}", command_str),
                range: Range { start: start_pos, end: parser.current_pos },
            }),
        };

        // Advance parser past the command
        parser.advance_position_by_str(captures.get(0).unwrap().as_str());

        let (parameters, _) = parse_parameters(parser)?;
        let (arguments, _) = parse_arguments(parser)?;

        let end_pos = parser.current_pos;
        dbg!("parse_tag: Successfully parsed tag");
        Ok(Some(Tag {
            command,
            parameters,
            arguments,
            range: Range { start: start_pos, end: end_pos },
        }))
    } else {
        dbg!("parse_tag: Tag regex did not match");
        Ok(None)
    }
}

pub fn parse_anchor(parser: &mut Parser) -> Result<Option<Anchor>, ParsingError> {
    dbg!("Entering parse_anchor", &parser.current_pos);
    let start_pos = parser.current_pos;

    // Anchors must start at the beginning of a line and begin with "<!--"
    if start_pos.column != 1 || !parser.remaining_slice().starts_with("<!--") {
        dbg!("parse_anchor: Not an anchor or not at column 1");
        return Ok(None);
    }

    let line_start_offset = parser.document[..start_pos.offset].rfind('\n').map_or(0, |i| i + 1);
    let line_slice = &parser.document[line_start_offset..];

    let anchor_regex = regex::Regex::new(r"^<!--\s*([a-zA-Z_][a-zA-Z0-9_]*)-([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}):(begin|end)").unwrap();

    if let Some(captures) = anchor_regex.captures(line_slice) {
        dbg!("parse_anchor: Anchor regex matched");
        let full_match = captures.get(0).unwrap().as_str();
        let command_str = captures.get(1).unwrap().as_str();
        let uuid_str = captures.get(2).unwrap().as_str();
        let kind_str = captures.get(3).unwrap().as_str();

        let command = match command_str {
            "include" => Command::Include,
            "inline" => Command::Inline,
            "answer" => Command::Answer,
            "derive" => Command::Derive,
            "summarize" => Command::Summarize,
            "set" => Command::Set,
            "repeat" => Command::Repeat,
            _ => return Err(ParsingError::InvalidSyntax {
                message: format!("Unknown command in anchor: {}", command_str),
                range: Range { start: start_pos, end: parser.current_pos },
            }),
        };

        let uuid = Uuid::parse_str(uuid_str).map_err(|e| ParsingError::InvalidSyntax {
            message: format!("Invalid UUID in anchor: {}", e),
            range: Range { start: start_pos, end: parser.current_pos },
        })?;

        let kind = match kind_str {
            "begin" => Kind::Begin,
            "end" => Kind::End,
            _ => unreachable!(), // Regex ensures this won't happen
        };

        // Advance parser past the initial anchor part
        parser.advance_position_by_str(full_match);

        let (parameters, _) = parse_parameters(parser)?;
        let (arguments, _) = parse_arguments(parser)?;

        parser.skip_whitespace();

        if !parser.remaining_slice().starts_with("-->") {
            dbg!("parse_anchor: Missing closing '-->'");
            return Err(ParsingError::UnterminatedString {
                range: Range { start: start_pos, end: parser.current_pos },
            });
        }
        parser.advance_position_by_str("-->");

        let end_pos = parser.current_pos;
        dbg!("parse_anchor: Successfully parsed anchor");
        Ok(Some(Anchor {
            command,
            uuid,
            kind,
            parameters,
            arguments,
            range: Range { start: start_pos, end: end_pos },
        }))
    } else {
        dbg!("parse_anchor: Anchor regex did not match");
        Ok(None)
    }
}

pub fn parse_text(parser: &mut Parser) -> Result<Option<Text>, ParsingError> {
    dbg!("Entering parse_text", &parser.current_pos);
    let start_pos = parser.current_pos;
    let mut content = String::new();

    loop {
        let remaining = parser.remaining_slice();
        if remaining.is_empty() {
            dbg!("parse_text: Remaining slice is empty, breaking loop");
            break;
        }

        // Check for start of a new tag or anchor
        if (remaining.starts_with("@") && parser.current_pos.column == 1) || (remaining.starts_with("<!--") && parser.current_pos.column == 1) {
            dbg!("parse_text: Found start of tag or anchor, breaking loop");
            break;
        }

        if let Some(c) = parser.consume() {
            content.push(c);
        } else {
            dbg!("parse_text: No more characters to consume, breaking loop");
            break;
        }
    }

    if content.trim().is_empty() {
        dbg!("parse_text: Content is whitespace only, returning None");
        Ok(None)
    } else {
        let end_pos = parser.current_pos;
        dbg!("parse_text: Successfully parsed text", &content);
        Ok(Some(Text {
            content,
            range: Range { start: start_pos, end: end_pos },
        }))
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

#[cfg(test)]
#[path = "debug_printer.rs"]
mod debug_printer;
