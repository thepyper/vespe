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
                            message: format!("Invalid escape sequence: \\{{}}", c),
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
    let mut parser = Parser::new(document);
    let start_position = parser.current_pos;
    let mut children = Vec::new();

    while parser.peek().is_some() {
        let current_offset = parser.current_pos.offset;
        if let Some(node) = parse_node(&mut parser)? {
            children.push(node);
        } else {
            // If no node was parsed, and we haven't advanced, it's an infinite loop or unexpected content
            if parser.current_pos.offset == current_offset {
                return Err(ParsingError::Custom {
                    message: "Parser stuck: unable to parse content at current position".to_string(),
                    range: Range { start: parser.current_pos, end: parser.current_pos },
                });
            }
        }
    }

    let end_position = parser.current_pos;
    let root_range = Range { start: start_position, end: end_position };

    Ok(Root {
        children,
        range: root_range,
    })
}

fn parse_node(parser: &mut Parser) -> Result<Option<Node>, ParsingError> {
    parser.skip_whitespace();
    let start_pos = parser.current_pos;

    if let Some(tag) = parse_tag(parser)? {
        return Ok(Some(Node::Tag(tag)));
    }

    // Reset position if tag parsing failed without consuming anything
    parser.current_pos = start_pos;

    if let Some(anchor) = parse_anchor(parser)? {
        return Ok(Some(Node::Anchor(anchor)));
    }

    // Reset position if anchor parsing failed without consuming anything
    parser.current_pos = start_pos;

    if let Some(text) = parse_text(parser)? {
        return Ok(Some(Node::Text(text)));
    }

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
    let start_pos = parser.current_pos;

    // Tags must start at the beginning of a line and begin with '@'
    if start_pos.column != 1 || parser.peek() != Some('@') {
        return Ok(None);
    }

    let line_start_offset = parser.document[..start_pos.offset].rfind('\n').map_or(0, |i| i + 1);
    let line_slice = &parser.document[line_start_offset..];

    let tag_regex = regex::Regex::new(r"^@([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();

    if let Some(captures) = tag_regex.captures(line_slice) {
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
                message: format!("Unknown command: {{}}", command_str),
                range: Range { start: start_pos, end: parser.current_pos },
            }),
        };

        // Advance parser past the command
        parser.advance_position_by_str(captures.get(0).unwrap().as_str());

        let (parameters, _) = parse_parameters(parser)?;
        let (arguments, _) = parse_arguments(parser)?;

        let end_pos = parser.current_pos;
        Ok(Some(Tag {
            command,
            parameters,
            arguments,
            range: Range { start: start_pos, end: end_pos },
        }))
    } else {
        Ok(None)
    }
}

pub fn parse_anchor(parser: &mut Parser) -> Result<Option<Anchor>, ParsingError> {
    let start_pos = parser.current_pos;

    // Anchors must start at the beginning of a line and begin with "<!--"
    if start_pos.column != 1 || !parser.remaining_slice().starts_with("<!--") {
        return Ok(None);
    }

    let line_start_offset = parser.document[..start_pos.offset].rfind('\n').map_or(0, |i| i + 1);
    let line_slice = &parser.document[line_start_offset..];

    let anchor_regex = regex::Regex::new(r"^<!--\s*([a-zA-Z_][a-zA-Z0-9_]*)-([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}):(begin|end)").unwrap();

    if let Some(captures) = anchor_regex.captures(line_slice) {
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
                message: format!("Unknown command in anchor: {{}}", command_str),
                range: Range { start: start_pos, end: parser.current_pos },
            }),
        };

        let uuid = Uuid::parse_str(uuid_str).map_err(|e| ParsingError::InvalidSyntax {
            message: format!("Invalid UUID in anchor: {{}}", e),
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
            return Err(ParsingError::UnterminatedString {
                range: Range { start: start_pos, end: parser.current_pos },
            });
        }
        parser.advance_position_by_str("-->");

        let end_pos = parser.current_pos;
        Ok(Some(Anchor {
            command,
            uuid,
            kind,
            parameters,
            arguments,
            range: Range { start: start_pos, end: end_pos },
        }))
    } else {
        Ok(None)
    }
}

pub fn parse_text(parser: &mut Parser) -> Result<Option<Text>, ParsingError> {
    let start_pos = parser.current_pos;
    let mut content = String::new();

    loop {
        let remaining = parser.remaining_slice();
        if remaining.is_empty() {
            break;
        }

        // Check for start of a new tag or anchor
        if (remaining.starts_with("@") && parser.current_pos.column == 1) || (remaining.starts_with("<!--") && parser.current_pos.column == 1) {
            break;
        }

        if let Some(c) = parser.consume() {
            content.push(c);
        } else {
            break;
        }
    }

    if content.is_empty() {
        Ok(None)
    } else {
        let end_pos = parser.current_pos;
        Ok(Some(Text {
            content,
            range: Range { start: start_pos, end: end_pos },
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_pos(offset: usize, line: usize, column: usize) -> Position {
        Position { offset, line, column }
    }

    fn create_range(start_offset: usize, start_line: usize, start_column: usize, end_offset: usize, end_line: usize, end_column: usize) -> Range {
        Range {
            start: create_pos(start_offset, start_line, start_column),
            end: create_pos(end_offset, end_line, end_column),
        }
    }

    #[test]
    fn test_parser_new() {
        let document = "hello";
        let parser = Parser::new(document);
        assert_eq!(parser.current_pos, create_pos(0, 1, 1));
        assert_eq!(parser.document, "hello");
    }

    #[test]
    fn test_parser_peek_consume_advance() {
        let mut parser = Parser::new("abc\n123");
        assert_eq!(parser.peek(), Some('a'));
        assert_eq!(parser.consume(), Some('a'));
        assert_eq!(parser.current_pos, create_pos(1, 1, 2));

        assert_eq!(parser.peek(), Some('b'));
        assert_eq!(parser.consume(), Some('b'));
        assert_eq!(parser.current_pos, create_pos(2, 1, 3));

        assert_eq!(parser.peek(), Some('c'));
        assert_eq!(parser.consume(), Some('c'));
        assert_eq!(parser.current_pos, create_pos(3, 1, 4));

        assert_eq!(parser.peek(), Some('\n'));
        assert_eq!(parser.consume(), Some('\n'));
        assert_eq!(parser.current_pos, create_pos(4, 2, 1)); // New line

        assert_eq!(parser.peek(), Some('1'));
        assert_eq!(parser.consume(), Some('1'));
        assert_eq!(parser.current_pos, create_pos(5, 2, 2));

        parser.advance_position_by_str("23");
        assert_eq!(parser.current_pos, create_pos(7, 2, 4));

        assert_eq!(parser.peek(), None);
        assert_eq!(parser.consume(), None);
    }

    #[test]
    fn test_parser_take_while() {
        let mut parser = Parser::new("  hello world");
        parser.skip_whitespace();
        assert_eq!(parser.current_pos, create_pos(2, 1, 3));

        let word = parser.take_while(|c| c.is_alphabetic());
        assert_eq!(word, "hello");
        assert_eq!(parser.current_pos, create_pos(7, 1, 8));
    }

    #[test]
    fn test_parser_parse_quoted_string_double_quotes() {
        let mut parser = Parser::new("\"hello world\"");
        let (s, r) = parser.parse_quoted_string('"').unwrap();
        assert_eq!(s, "hello world");
        assert_eq!(r, create_range(0, 1, 1, 13, 1, 14));
    }

    #[test]
    fn test_parser_parse_quoted_string_single_quotes() {
        let mut parser = Parser::new("'hello world'");
        let (s, r) = parser.parse_quoted_string('\'').unwrap();
        assert_eq!(s, "hello world");
        assert_eq!(r, create_range(0, 1, 1, 13, 1, 14));
    }

    #[test]
    fn test_parser_parse_quoted_string_with_escapes() {
        let mut parser = Parser::new("\"hello\\nworld\\\"\"");
        let (s, r) = parser.parse_quoted_string('"').unwrap();
        assert_eq!(s, "hello\nworld\"");
        assert_eq!(r, create_range(0, 1, 1, 16, 1, 17));
    }

    #[test]
    fn test_parser_parse_quoted_string_unterminated() {
        let mut parser = Parser::new("\"hello world");
        let err = parser.parse_quoted_string('"').unwrap_err();
        assert_eq!(err, ParsingError::UnterminatedString { range: create_range(0, 1, 1, 12, 1, 13) });
    }

    #[test]
    fn test_parser_parse_unquoted_identifier() {
        let mut parser = Parser::new("my_identifier123 rest");
        let (s, r) = parser.parse_unquoted_identifier().unwrap();
        assert_eq!(s, "my_identifier123");
        assert_eq!(r, create_range(0, 1, 1, 16, 1, 17));
        assert_eq!(parser.remaining_slice(), " rest");
    }

    #[test]
    fn test_parser_parse_number_integer() {
        let mut parser = Parser::new("12345 rest");
        let (val, r) = parser.parse_number().unwrap().unwrap();
        assert_eq!(val, ParameterValue::Integer(12345));
        assert_eq!(r, create_range(0, 1, 1, 5, 1, 6));
        assert_eq!(parser.remaining_slice(), " rest");
    }

    #[test]
    fn test_parser_parse_number_float() {
        let mut parser = Parser::new("123.45 rest");
        let (val, r) = parser.parse_number().unwrap().unwrap();
        assert_eq!(val, ParameterValue::Float(123.45));
        assert_eq!(r, create_range(0, 1, 1, 6, 1, 7));
        assert_eq!(parser.remaining_slice(), " rest");
    }

    #[test]
    fn test_parser_parse_number_negative() {
        let mut parser = Parser::new("-123 rest");
        let (val, r) = parser.parse_number().unwrap().unwrap();
        assert_eq!(val, ParameterValue::Integer(-123));
        assert_eq!(r, create_range(0, 1, 1, 4, 1, 5));
        assert_eq!(parser.remaining_slice(), " rest");
    }

    #[test]
    fn test_parser_parse_boolean_true() {
        let mut parser = Parser::new("true rest");
        let (val, r) = parser.parse_boolean().unwrap();
        assert_eq!(val, ParameterValue::Boolean(true));
        assert_eq!(r, create_range(0, 1, 1, 4, 1, 5));
        assert_eq!(parser.remaining_slice(), " rest");
    }

    #[test]
    fn test_parser_parse_boolean_false() {
        let mut parser = Parser::new("false rest");
        let (val, r) = parser.parse_boolean().unwrap();
        assert_eq!(val, ParameterValue::Boolean(false));
        assert_eq!(r, create_range(0, 1, 1, 5, 1, 6));
        assert_eq!(parser.remaining_slice(), " rest");
    }

    #[test]
    fn test_parse_parameters_empty() {
        let mut parser = Parser::new("{}");
        let (params, r) = parse_parameters(&mut parser).unwrap();
        assert!(params.is_empty());
        assert_eq!(r, create_range(0, 1, 1, 2, 1, 3));
    }

    #[test]
    fn test_parse_parameters_single_unquoted() {
        let mut parser = Parser::new("{key: value}");
        let (params, r) = parse_parameters(&mut parser).unwrap();
        assert_eq!(params.len(), 1);
        assert_eq!(params["key"], ParameterValue::String("value".to_string()));
        assert_eq!(r, create_range(0, 1, 1, 12, 1, 13));
    }

    #[test]
    fn test_parse_parameters_multiple_quoted() {
        let mut parser = Parser::new("{\"key1\": \"value1\", 'key2': 123, key3: true}");
        let (params, r) = parse_parameters(&mut parser).unwrap();
        assert_eq!(params.len(), 3);
        assert_eq!(params["key1"], ParameterValue::String("value1".to_string()));
        assert_eq!(params["key2"], ParameterValue::Integer(123));
        assert_eq!(params["key3"], ParameterValue::Boolean(true));
        assert_eq!(r, create_range(0, 1, 1, 44, 1, 45));
    }

    #[test]
    fn test_parse_parameters_invalid_syntax() {
        let mut parser = Parser::new("{key: }");
        let err = parse_parameters(&mut parser).unwrap_err();
        assert!(matches!(err, ParsingError::InvalidSyntax { .. }));
    }

    #[test]
    fn test_parse_argument_quoted() {
        let mut parser = Parser::new("\"arg1\" rest");
        let (arg, r) = parse_argument(&mut parser).unwrap().unwrap();
        assert_eq!(arg, "arg1");
        assert_eq!(r, create_range(0, 1, 1, 6, 1, 7));
        assert_eq!(parser.remaining_slice(), " rest");
    }

    #[test]
    fn test_parse_argument_unquoted() {
        let mut parser = Parser::new("arg1 rest");
        let (arg, r) = parse_argument(&mut parser).unwrap().unwrap();
        assert_eq!(arg, "arg1");
        assert_eq!(r, create_range(0, 1, 1, 4, 1, 5));
        assert_eq!(parser.remaining_slice(), " rest");
    }

    #[test]
    fn test_parse_arguments_multiple() {
        let mut parser = Parser::new("arg1 \"arg2 with spaces\" arg3");
        let (args, r) = parse_arguments(&mut parser).unwrap();
        assert_eq!(args, vec!["arg1", "arg2 with spaces", "arg3"]);
        assert_eq!(r, create_range(0, 1, 1, 29, 1, 30));
    }

    #[test]
    fn test_parse_arguments_empty() {
        let mut parser = Parser::new("");
        let (args, r) = parse_arguments(&mut parser).unwrap();
        assert!(args.is_empty());
        assert_eq!(r, create_range(0, 1, 1, 0, 1, 1));
    }

    #[test]
    fn test_parse_tag_simple() {
        let mut parser = Parser::new("@include arg1 arg2");
        let tag = parse_tag(&mut parser).unwrap().unwrap();
        assert_eq!(tag.command, Command::Include);
        assert!(tag.parameters.is_empty());
        assert_eq!(tag.arguments, vec!["arg1", "arg2"]);
        assert_eq!(tag.range, create_range(0, 1, 1, 18, 1, 19));
    }

    #[test]
    fn test_parse_tag_with_parameters() {
        let mut parser = Parser::new("@answer {key: \"value\"} arg1");
        let tag = parse_tag(&mut parser).unwrap().unwrap();
        assert_eq!(tag.command, Command::Answer);
        assert_eq!(tag.parameters.len(), 1);
        assert_eq!(tag.parameters["key"], ParameterValue::String("value".to_string()));
        assert_eq!(tag.arguments, vec!["arg1"]);
        assert_eq!(tag.range, create_range(0, 1, 1, 28, 1, 29));
    }

    #[test]
    fn test_parse_tag_not_at_column_1() {
        let mut parser = Parser::new(" @include arg1");
        let tag = parse_tag(&mut parser).unwrap();
        assert!(tag.is_none());
    }

    #[test]
    fn test_parse_anchor_begin() {
        let uuid = Uuid::new_v4();
        let document = format!("<!-- include-{{}}:begin --> arg1", uuid);
        let mut parser = Parser::new(&document);
        let anchor = parse_anchor(&mut parser).unwrap().unwrap();
        assert_eq!(anchor.command, Command::Include);
        assert_eq!(anchor.uuid, uuid);
        assert_eq!(anchor.kind, Kind::Begin);
        assert!(anchor.parameters.is_empty());
        assert_eq!(anchor.arguments, vec!["arg1"]);
        assert_eq!(anchor.range, create_range(0, 1, 1, document.len(), 1, document.len() + 1));
    }

    #[test]
    fn test_parse_anchor_end_with_parameters() {
        let uuid = Uuid::new_v4();
        let document = format!("<!-- answer-{{}}:end --> {{status: \"ok\"}}", uuid);
        let mut parser = Parser::new(&document);
        let anchor = parse_anchor(&mut parser).unwrap().unwrap();
        assert_eq!(anchor.command, Command::Answer);
        assert_eq!(anchor.uuid, uuid);
        assert_eq!(anchor.kind, Kind::End);
        assert_eq!(anchor.parameters.len(), 1);
        assert_eq!(anchor.parameters["status"], ParameterValue::String("ok".to_string()));
        assert!(anchor.arguments.is_empty());
        assert_eq!(anchor.range, create_range(0, 1, 1, document.len(), 1, document.len() + 1));
    }

    #[test]
    fn test_parse_anchor_unterminated() {
        let uuid = Uuid::new_v4();
        let document = format!("<!-- include-{{}}:begin", uuid);
        let mut parser = Parser::new(&document);
        let err = parse_anchor(&mut parser).unwrap_err();
        assert!(matches!(err, ParsingError::UnterminatedString { .. }));
    }

    #[test]
    fn test_parse_text_simple() {
        let mut parser = Parser::new("This is some text.");
        let text = parse_text(&mut parser).unwrap().unwrap();
        assert_eq!(text.content, "This is some text.");
        assert_eq!(text.range, create_range(0, 1, 1, 18, 1, 19));
    }

    #[test]
    fn test_parse_text_until_tag() {
        let mut parser = Parser::new("Text before tag.\n@include arg");
        let text = parse_text(&mut parser).unwrap().unwrap();
        assert_eq!(text.content, "Text before tag.\n");
        assert_eq!(text.range, create_range(0, 1, 1, 18, 2, 1));
        assert_eq!(parser.remaining_slice(), "@include arg");
    }

    #[test]
    fn test_parse_text_until_anchor() {
        let uuid = Uuid::new_v4();
        let document = format!("Text before anchor.\n<!-- include-{{}}:begin -->", uuid);
        let mut parser = Parser::new(&document);
        let text = parse_text(&mut parser).unwrap().unwrap();
        assert_eq!(text.content, "Text before anchor.\n");
        assert_eq!(text.range, create_range(0, 1, 1, 21, 2, 1));
        assert!(parser.remaining_slice().starts_with("<!--"));
    }

    #[test]
    fn test_parse_node_tag() {
        let mut parser = Parser::new("@include arg");
        let node = parse_node(&mut parser).unwrap().unwrap();
        assert!(matches!(node, Node::Tag(_)));
    }

    #[test]
    fn test_parse_node_anchor() {
        let uuid = Uuid::new_v4();
        let document = format!("<!-- include-{{}}:begin -->", uuid);
        let mut parser = Parser::new(&document);
        let node = parse_node(&mut parser).unwrap().unwrap();
        assert!(matches!(node, Node::Anchor(_)));
    }

    #[test]
    fn test_parse_node_text() {
        let mut parser = Parser::new("Just some text.");
        let node = parse_node(&mut parser).unwrap().unwrap();
        assert!(matches!(node, Node::Text(_)));
    }

    #[test]
    fn test_parse_mixed_content() {
        let uuid1 = Uuid::new_v4();
        let uuid2 = Uuid::new_v4();
        let document = format!("Some initial text.\n@include file.md arg1\n<!-- derive-{{}}:begin -->\nMore text here.\n<!-- derive-{{}}:end -->\nFinal text.", uuid1, uuid2);

        let root = parse(&document).unwrap();
        assert_eq!(root.children.len(), 5);

        // Node 1: Text
        if let Node::Text(text) = &root.children[0] {
            assert_eq!(text.content, "Some initial text.\n");
        } else {
            panic!("Expected Text node");
        }

        // Node 2: Tag
        if let Node::Tag(tag) = &root.children[1] {
            assert_eq!(tag.command, Command::Include);
            assert_eq!(tag.arguments, vec!["file.md", "arg1"]);
        } else {
            panic!("Expected Tag node");
        }

        // Node 3: Anchor (begin)
        if let Node::Anchor(anchor) = &root.children[2] {
            assert_eq!(anchor.command, Command::Derive);
            assert_eq!(anchor.uuid, uuid1);
            assert_eq!(anchor.kind, Kind::Begin);
        } else {
            panic!("Expected Anchor node");
        }

        // Node 4: Text
        if let Node::Text(text) = &root.children[3] {
            assert_eq!(text.content, "More text here.\n");
        } else {
            panic!("Expected Text node");
        }

        // Node 5: Anchor (end)
        if let Node::Anchor(anchor) = &root.children[4] {
            assert_eq!(anchor.command, Command::Derive);
            assert_eq!(anchor.uuid, uuid2);
            assert_eq!(anchor.kind, Kind::End);
        } else {
            panic!("Expected Anchor node");
        }
    }

    #[test]
    fn test_parse_empty_document() {
        let document = "";
        let root = parse(document).unwrap();
        assert!(root.children.is_empty());
        assert_eq!(root.range, create_range(0, 1, 1, 0, 1, 1));
    }

    #[test]
    fn test_parse_only_whitespace() {
        let document = "   \n\n";
        let root = parse(document).unwrap();
        assert_eq!(root.children.len(), 1);
        if let Node::Text(text) = &root.children[0] {
            assert_eq!(text.content, "   \n\n");
        } else {
            panic!("Expected Text node");
        }
    }
}
