use uuid::Uuid;

use std::collections::HashMap;
    pub offset: usize, // 0-based byte offset from the start of the document
    pub line: usize,   // 1-based line number
    pub column: usize, // 1-based column number (character index on the line)
}

pub struct Range
{
    pub begin: Position,  // 0-based offset
    pub end: Position,  // 0-based offset
}

struct Root
{
    children: Vec<Node>,
    range: Range,
}

struct Text
{
    content: String,
    range: Range,
}

enum Command
{
    Include,
    Inline,
    Answer,
    Derive,
    Summarize,
    Set,
    Repeat,
}

struct TagOpening
{
    command: Command,
}

struct Tag
{
    opening: TagOpening,
    parameters: Parameters,
    parameters_range: Range,
    arguments: Arguments,
    arguments_range: Range,
}

enum Kind
{
    Begin,
    End,
}

struct AnchorOpening
{
    command: Command,
    uuid: Uuid,
    kind: Kind,
}

struct Anchor
{
    opening: AnchorOpening,
    parameters: Parameters,
    parameters_range: Range,
    arguments: Arguments,
    arguments_range: Range,
    range: Range,
}

enum Node
{
    Root(Root),
    Tag(Tag),
    Anchor(Anchor),
    Text(Text),
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParsingError {
    #[error("Unexpected end of input at {0:?}")]
    UnexpectedEndOfInput(Position),
    #[error("Expected character '{expected}' but found '{found}' at {position:?}")]
    ExpectedChar { expected: char, found: char, position: Position },
    #[error("Invalid escape sequence '\{sequence}' at {position:?}")]
    InvalidEscapeSequence { sequence: char, position: Position },
    #[error("Unclosed string literal starting at {0:?}")]
    UnclosedStringLiteral(Position),
    #[error("Invalid parameter format at {0:?}: {1}")]
    InvalidParameterFormat(Position, String),
    #[error("Parsing not advanced from {0:?}")]
    ParsingNotAdvanced(Position),
    #[error("Generic parsing error at {0:?}: {1}")]
    Generic(Position, String),
}

pub fn parse(document: &str) -> Result<Root, ParsingError> 
{
    let begin_position = Position { offset: 0, line: 1, column: 1 };

    let (children, range) = parse_many_nodes(document, begin_position)?;

    Ok(Root{
        children,
        range,
    })

}

fn parse_many_nodes(document: &str, begin: Position) -> Result<(Vec<Node>, Range), ParsingError>
{
    let mut nodes = Vec::new();

    let end_offset = document.len();
    let mut current_position = begin;

    while current_position.offset < end_offset {

        let (node, range) = parse_node(document, current_position)?;
        nodes.push(node);

        if range.begin.offset == range.end.offset {
            // Parsing did not advance, this indicates an infinite loop if not handled.
            return Err(ParsingError::ParsingNotAdvanced(current_position));
        }

        current_position = range.end;
    }

    Ok((nodes, Range { begin, end: current_position }))
}

fn parse_node(document: &str, begin: Position) -> Result<(Node, Range), ParsingError>
{
    if let Some((tag, range)) = parse_tag(document, begin)? {
        Ok((Node::Tag(tag), range))
    } else if let Some((anchor, range)) = parse_anchor(document, begin)? {
        Ok((Node::Anchor(anchor), range))
    } else if let Some((text, range)) = parse_text(document, begin)? {
        Ok((Node::Text(text), range))
    } else {
        Err(ParsingError::ParsingNotAdvanced(begin))
    }
}

fn parse_text(document: &str, begin: Position) -> Result<Option<(Text, Range)>, ParsingError>
{
    let mut current_position = begin;
    let mut text_content = String::new();

    loop {
        let remaining_doc = document.get(current_position.offset..).unwrap_or("");

        // Check for start of a tag
        let tag_re = Regex::new(r"^@([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
        if tag_re.is_match(remaining_doc) && current_position.column == 1 {
            break;
        }

        // Check for start of an anchor
        let anchor_re = Regex::new(r"^<!--\s*([a-zA-Z_][a-zA-Z0-9_]*)-([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}):(begin|end)\s*").unwrap();
        if anchor_re.is_match(remaining_doc) && current_position.column == 1 {
            break;
        }

        let Some(ch) = document.chars().nth(current_position.offset) else {
            break; // End of document
        };

        text_content.push(ch);
        current_position = advance_position(current_position, ch);
    }

    if text_content.is_empty() {
        Ok(None)
    } else {
        Ok(Some((Text { content: text_content, range: Range { begin, end: current_position } }, Range { begin, end: current_position })))}

use regex::Regex;

fn parse_tag(document: &str, begin: Position) -> Result<Option<(Tag, Range)>, ParsingError>
{
    if begin.column != 1 {
        return Ok(None);
    }

    let mut current_position = begin;

    let tag_re = Regex::new(r"^@([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
    let Some(captures) = tag_re.captures(document.get(current_position.offset..).unwrap_or("")) else {
        return Ok(None);
    };

    let full_match = captures.get(0).unwrap();
    let command_str = captures.get(1).unwrap().as_str();

    let command = match command_str {
        "include" => Command::Include,
        "inline" => Command::Inline,
        "answer" => Command::Answer,
        "derive" => Command::Derive,
        "summarize" => Command::Summarize,
        "set" => Command::Set,
        "repeat" => Command::Repeat,
        _ => return Ok(None), // Unknown command
    };

    for _ in 0..full_match.len() {
        current_position = advance_position(current_position, document.chars().nth(current_position.offset).unwrap());
    }

    let tag_opening = TagOpening {
        command,
    };

    // Parse parameters
    let (parameters, parameters_range) = if let Some((p, r)) = parse_parameters(document, current_position)? {
        (p, r)
    } else {
        (HashMap::new(), Range { begin: current_position, end: current_position })
    };
    current_position = parameters_range.end;

    // Parse arguments
    let (arguments_struct, arguments_range) = if let Some((a, r)) = parse_arguments(document, current_position)? {
        (a, r)
    } else {
        (Arguments { children: Vec::new() }, Range { begin: current_position, end: current_position })
    };
    current_position = arguments_range.end;

    Ok(Some((Tag {
        opening: tag_opening,
        parameters,
        parameters_range,
        arguments: arguments_struct,
        arguments_range,
    }, Range { begin, end: current_position })))

fn parse_anchor(document: &str, begin: Position) -> Result<Option<(Anchor, Range)>, ParsingError>
{
    if begin.column != 1 {
        return Ok(None);
    }

    let mut current_position = begin;

    let anchor_re = Regex::new(r"^<!--\s*([a-zA-Z_][a-zA-Z0-9_]*)-([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}):(begin|end)\s*").unwrap();
    let Some(captures) = anchor_re.captures(document.get(current_position.offset..).unwrap_or("")) else {
        return Ok(None);
    };

    let full_match = captures.get(0).unwrap();
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
        _ => return Ok(None), // Unknown command
    };

    let uuid = Uuid::parse_str(uuid_str).map_err(|e| ParsingError::Generic(current_position, format!("Invalid UUID: {}", e)))?;

    let kind = match kind_str {
        "begin" => Kind::Begin,
        "end" => Kind::End,
        _ => return Ok(None), // Should not happen due to regex
    };

    for _ in 0..full_match.len() {
        current_position = advance_position(current_position, document.chars().nth(current_position.offset).unwrap());
    }

    let anchor_opening = AnchorOpening {
        command,
        uuid,
        kind,
    };

    // Parse parameters
    let (parameters, parameters_range) = if let Some((p, r)) = parse_parameters(document, current_position)? {
        (p, r)
    } else {
        (HashMap::new(), Range { begin: current_position, end: current_position })
    };
    current_position = parameters_range.end;

    // Parse arguments
    let (arguments_struct, arguments_range) = if let Some((a, r)) = parse_arguments(document, current_position)? {
        (a, r)
    } else {
        (Arguments { children: Vec::new() }, Range { begin: current_position, end: current_position })
    };
    current_position = arguments_range.end;

    // Expect closing '-->'
    current_position = skip_whitespace(document, current_position);
    let Some(closing_chars) = document.get(current_position.offset..) else {
        return Err(ParsingError::UnexpectedEndOfInput(current_position));
    };
    if !closing_chars.starts_with("-->") {
        return Err(ParsingError::ExpectedChar { expected: '-', found: closing_chars.chars().next().unwrap_or('\0'), position: current_position });
    }

    for _ in 0..3 { // Advance 3 characters for "-->"
        current_position = advance_position(current_position, document.chars().nth(current_position.offset).unwrap());
    }

    Ok(Some((Anchor {
        opening: anchor_opening,
        parameters,
        parameters_range,
        arguments: arguments_struct,
        arguments_range,
    }, Range { begin, end: current_position })))
/// Advances the given position by the provided character.
fn advance_position(mut position: Position, ch: char) -> Position {
    position.offset += ch.len_utf8();
    if ch == '\n' {
        position.line += 1;
        position.column = 1;
    } else {
        position.column += 1;
    }
    position
}

/// Skips whitespace characters and updates the position.
fn skip_whitespace(document: &str, mut position: Position) -> Position {
    while let Some(ch) = document.chars().nth(position.offset) {
        if ch.is_whitespace() {
            position = advance_position(position, ch);
        } else {
            break;
        }
    }
    position
}

fn parse_parameters(document: &str, begin: Position) -> Result<Option<(Parameters, Range)>, ParsingError>
{
    let mut current_position = begin;

    // Check for opening brace
    let Some(first_char) = document.chars().nth(current_position.offset) else { return Ok(None); };
    if first_char != '{' {
        return Ok(None);
    }
    current_position = advance_position(current_position, first_char);

    let mut parameters = HashMap::new();

/// Parses a parameter value (string, number, boolean).
fn parse_parameter_value(document: &str, mut position: Position) -> Result<(ParameterValue, Position), ParsingError> {
    let value_start_pos = position;
    let Some(first_char) = document.chars().nth(position.offset) else {
        return Err(ParsingError::UnexpectedEndOfInput(position));
    };

    if first_char == ''' || first_char == '"' {
        // Quoted string
        let quote_char = first_char;
        position = advance_position(position, quote_char);
        let mut value = String::new();
        while let Some(ch) = document.chars().nth(position.offset) {
            if ch == quote_char {
                position = advance_position(position, ch);
                return Ok((ParameterValue::String(value), position));
            } else if ch == '\\' {
                position = advance_position(position, ch);
                let Some(escaped_char) = document.chars().nth(position.offset) else {
                    return Err(ParsingError::UnexpectedEndOfInput(position));
                };
                value.push(escaped_char);
                position = advance_position(position, escaped_char);
            } else {
                value.push(ch);
                position = advance_position(position, ch);
            }
        }
        return Err(ParsingError::UnclosedStringLiteral(value_start_pos));
    } else if first_char.is_ascii_digit() || first_char == '-' {
        // Number (integer or float)
        let mut num_str = String::new();
        let mut is_float = false;
        while let Some(ch) = document.chars().nth(position.offset) {
            if ch.is_ascii_digit() {
                num_str.push(ch);
                position = advance_position(position, ch);
            } else if ch == '.' && !is_float {
                num_str.push(ch);
                is_float = true;
                position = advance_position(position, ch);
            } else if ch == '-' && num_str.is_empty() {
                num_str.push(ch);
                position = advance_position(position, ch);
            } else {
                break;
            }
        }
        if is_float {
            if let Ok(f) = num_str.parse::<f64>() {
                return Ok((ParameterValue::Float(f), position));
            } else {
                return Err(ParsingError::InvalidParameterFormat(value_start_pos, format!("Invalid float: {}", num_str)));
            }
        } else {
            if let Ok(i) = num_str.parse::<i64>() {
                return Ok((ParameterValue::Integer(i), position));
            } else {
                return Err(ParsingError::InvalidParameterFormat(value_start_pos, format!("Invalid integer: {}", num_str)));
            }
        }
    } else if document.get(position.offset..).map_or(false, |s| s.starts_with("true")) {
        position = advance_position(position, 't');
        position = advance_position(position, 'r');
        position = advance_position(position, 'u');
        position = advance_position(position, 'e');
        return Ok((ParameterValue::Boolean(true), position));
    } else if document.get(position.offset..).map_or(false, |s| s.starts_with("false")) {
        position = advance_position(position, 'f');
        position = advance_position(position, 'a');
        position = advance_position(position, 'l');
        position = advance_position(position, 's');
        position = advance_position(position, 'e');
        return Ok((ParameterValue::Boolean(false), position));
    } else {
        // Unquoted string
        let mut value = String::new();
        while let Some(ch) = document.chars().nth(position.offset) {
            if ch.is_whitespace() || ch == ',' || ch == '}' {
                break;
            } else {
                value.push(ch);
                position = advance_position(position, ch);
            }
        }
        if value.is_empty() {
            return Err(ParsingError::InvalidParameterFormat(value_start_pos, "Empty value".to_string()));
        }
        return Ok((ParameterValue::String(value), position));
    }
}

    loop {
        current_position = skip_whitespace(document, current_position);

        // Check for closing brace or end of input
        let Some(ch) = document.chars().nth(current_position.offset) else {
            return Err(ParsingError::UnexpectedEndOfInput(current_position));
        };
        if ch == '}' {
            break; // End of parameters
        }

        // Parse key
        let key_start_pos = current_position;
        let mut key = String::new();
        let mut in_quote = None; // None, Some('''), Some('"')

        if ch == ''' || ch == '"' {
            in_quote = Some(ch);
            current_position = advance_position(current_position, ch);
        }

        while let Some(key_char) = document.chars().nth(current_position.offset) {
            if let Some(quote_char) = in_quote {
                if key_char == quote_char {
                    current_position = advance_position(current_position, key_char);
                    break;
                } else if key_char == '\\' {
                    // Handle escape sequence
                    current_position = advance_position(current_position, key_char);
                    let Some(escaped_char) = document.chars().nth(current_position.offset) else {
                        return Err(ParsingError::UnexpectedEndOfInput(current_position));
                    };
                    key.push(escaped_char);
                    current_position = advance_position(current_position, escaped_char);
                } else {
                    key.push(key_char);
                    current_position = advance_position(current_position, key_char);
                }
            } else { // Not in quote
                if key_char.is_whitespace() || key_char == ':' || key_char == ',' || key_char == '}' {
                    break;
                } else {
                    key.push(key_char);
                    current_position = advance_position(current_position, key_char);
                }
            }
        }

        if in_quote.is_some() && document.chars().nth(current_position.offset - 1) != in_quote {
            return Err(ParsingError::UnclosedStringLiteral(key_start_pos));
        }

        // Expect colon
        current_position = skip_whitespace(document, current_position);
        let Some(colon_char) = document.chars().nth(current_position.offset) else {
            return Err(ParsingError::UnexpectedEndOfInput(current_position));
        };
        if colon_char != ':' {
            return Err(ParsingError::ExpectedChar { expected: ':', found: colon_char, position: current_position });
        }
        current_position = advance_position(current_position, colon_char);

        current_position = skip_whitespace(document, current_position);
        let (value, next_pos) = parse_parameter_value(document, current_position)?;
        current_position = next_pos;
        parameters.insert(key, value);

        current_position = skip_whitespace(document, current_position);
        let Some(separator_char) = document.chars().nth(current_position.offset) else {
            return Err(ParsingError::UnexpectedEndOfInput(current_position));
        };

        if separator_char == ',' {
            current_position = advance_position(current_position, separator_char);
        } else if separator_char == '}' {
            break; // End of parameters
        } else {
            return Err(ParsingError::ExpectedChar { expected: ',' , found: separator_char, position: current_position });
        }

    }

    // Expect closing brace
    let Some(last_char) = document.chars().nth(current_position.offset) else { 
        return Err(ParsingError::UnexpectedEndOfInput(current_position));
    };
    if last_char != '}' {
        return Err(ParsingError::ExpectedChar { expected: '}', found: last_char, position: current_position });
    }
    current_position = advance_position(current_position, last_char);

    Ok(Some((parameters, Range { begin, end: current_position })))
}

fn parse_arguments(document: &str, begin: Position) -> Result<Option<(Arguments, Range)>, ParsingError>
{
    let mut current_position = begin;
    let mut arguments_vec = Vec::new();

    loop {
        let Some(ch) = document.chars().nth(current_position.offset) else {
            break; // End of document
        };

        if ch == '}' || ch == '\n' {
            break; // End of arguments for this context
        }

        if let Some((argument, range)) = parse_argument(document, current_position)? {
            arguments_vec.push(argument);
            current_position = range.end;
        } else {
            break; // No more arguments found
        }
    }

    if arguments_vec.is_empty() {
        Ok(None)
    } else {
        Ok(Some((Arguments { children: arguments_vec }, Range { begin, end: current_position })))
fn parse_argument(document: &str, begin: Position) -> Result<Option<(Argument, Range)>, ParsingError>
{
    let mut current_position = begin;
    current_position = skip_whitespace(document, current_position);

    let Some(first_char) = document.chars().nth(current_position.offset) else {
        return Ok(None);
    };

    let mut arg_str = String::new();
    let mut in_quote = None; // None, Some('''), Some('"')

    if first_char == ''' || first_char == '"' {
        in_quote = Some(first_char);
        current_position = advance_position(current_position, first_char);
    }

    while let Some(ch) = document.chars().nth(current_position.offset) {
        if let Some(quote_char) = in_quote {
            if ch == quote_char {
                current_position = advance_position(current_position, ch);
                break;
            } else if ch == '\\' {
                current_position = advance_position(current_position, ch);
                let Some(escaped_char) = document.chars().nth(current_position.offset) else {
                    return Err(ParsingError::UnexpectedEndOfInput(current_position));
                };
                arg_str.push(escaped_char);
                current_position = advance_position(current_position, escaped_char);
            } else {
                arg_str.push(ch);
                current_position = advance_position(current_position, ch);
            }
        } else { // Not in quote
            if ch.is_whitespace() || ch == '}' { // '}' is a special case for arguments after parameters
                break;
            } else {
                arg_str.push(ch);
                current_position = advance_position(current_position, ch);
            }
        }
    }

    if in_quote.is_some() && document.chars().nth(current_position.offset - 1) != in_quote {
        return Err(ParsingError::UnclosedStringLiteral(begin));
    }

    if arg_str.is_empty() && in_quote.is_none() {
        return Ok(None);
    }

    Ok(Some((Argument { }, Range { begin, end: current_position })))


