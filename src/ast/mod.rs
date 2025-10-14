struct Range
{
    begin: usize,  // 0-based offset
    end: usize,  // 0-based offset
}

struct Root
{
    children: Vec<Node>,
    range: Range,
}

struct Text
{
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
    range: Range,
}

struct Parameters
{
    parameters: serde_json::Value,
    range: Range,
}

struct Argument
{
    range: Range,
}

struct Arguments
{
    children: Vec<Argument>,
    range: Range,
}

struct Tag
{
    opening: TagOpening,
    parameters: Parameters,
    arguments: Arguments,
    range: Range,
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
    range: Range,
}

struct Anchor
{
    opening: AnchorOpening,
    parameters: Parameters,
    arguments: Arguments,
    range: Range,
}

enum Node
{
    Root(Root),
    Tag(Tag),
    Anchor(Anchor),
    Text(Text),
}

use uuid::Uuid;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParsingError {
    #[error("Unexpected token: {0}")]
    UnexpectedToken(String),
    #[error("Invalid UUID: {0}")]
    InvalidUuid(#[from] uuid::Error),
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Parsing not advanced at position {0}")]
    ParsingNotAdvanced(usize),
    #[error("End of document reached unexpectedly")]
    EndOfDocument,
}


pub fn parse(document: &str) -> Result<Root, ParsingError> 
{
    let begin = 0usize;

    let (children, range) = parse_many_nodes(document, begin)?;

    Ok(Root{
        children,
        range,
    })

}

fn parse_many_nodes(document: &str, begin: usize) -> Result<Vec<Node>, ParsingError>
{
    let mut nodes = Vec::new();

    let end_offset = document.len();
    let mut position = begin;

    while position < end_offset {

        let (node, range) = parse_node(document, position)?;
        nodes.push(node);

        position = range.end;
    }

    Ok(nodes)
}

fn parse_node(document: &str, begin: usize) -> Result<Node, ParsingError>
{
    if let Some(tag) = parse_tag(document, begin)? {
        Ok(Node::Tag(tag))
    } else if let Some(anchor) = parse_anchor(document, begin)? {
        Ok(Node::Anchor(anchor))
    } else if let Some(text) = parse_text(document, begin)? {
        Ok(Node::Text(text))
    } else {
        Err(ParsingError::ParsingNotAdvanced(begin))
    }
}

fn parse_tag(document: &str, begin: usize) -> Result<Option<Tag>, ParsingError>
{
    let s = &document[begin..];

    // Check for begin.column == 1
    let line_start = document[..begin].rfind('\n').map_or(0, |i| i + 1);
    if begin != line_start {
        return Ok(None); // Tag must start at the beginning of a line
    }

    if !s.starts_with("@") {
        return Ok(None);
    }

    let mut chars = s.char_indices().peekable();
    chars.next(); // Consume '@'

    let command_start = begin + 1;
    let mut command_end = command_start;

    while let Some((i, c)) = chars.peek() {
        if c.is_alphanumeric() || *c == '-' || *c == '_' {
            command_end = begin + i + c.len_utf8();
            chars.next();
        } else {
            break;
        }
    }

    if command_start == command_end {
        return Err(ParsingError::UnexpectedToken("Expected command name after '@'".to_string()));
    }

    let command_str = &document[command_start..command_end];
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

    let opening = TagOpening {
        command,
        range: Range { begin, end: command_end },
    };

    let parameters = parse_parameters(document, command_end)?;
    let arguments = parse_arguments(document, parameters.range.end)?;

    let end = arguments.range.end;

    Ok(Some(Tag {
        opening,
        parameters,
        arguments,
        range: Range { begin, end },
    }))
}

fn parse_anchor(document: &str, begin: usize) -> Result<Option<Anchor>, ParsingError>
{
    let s = &document[begin..];

    // Check for begin.column == 1
    let line_start = document[..begin].rfind('\n').map_or(0, |i| i + 1);
    if begin != line_start {
        return Ok(None); // Anchor must start at the beginning of a line
    }

    if !s.starts_with("<!-- ") {
        return Ok(None);
    }

    let mut chars = s.char_indices().peekable();
    let mut current_pos = begin;

    // Consume "<!-- "
    for _ in 0.."<!-- ".len() {
        chars.next();
        current_pos += 1;
    }

    // Parse command
    let command_start = current_pos;
    let mut command_end = command_start;
    while let Some((i, c)) = chars.peek() {
        if c.is_alphanumeric() || *c == '-' || *c == '_' {
            command_end = begin + i + c.len_utf8();
            chars.next();
        } else {
            break;
        }
    }
    let command_str = &document[command_start..command_end];
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

    // Consume '-'
    if chars.next().map(|(_, c)| c) != Some('-') {
        return Ok(None);
    }
    current_pos = command_end + 1;

    // Parse UUID
    let uuid_start = current_pos;
    let mut uuid_end = uuid_start;
    while let Some((i, c)) = chars.peek() {
        if c.is_alphanumeric() || *c == '-' {
            uuid_end = begin + i + c.len_utf8();
            chars.next();
        } else {
            break;
        }
    }
    let uuid_str = &document[uuid_start..uuid_end];
    let uuid = Uuid::parse_str(uuid_str)?;
    current_pos = uuid_end;

    // Consume ':'
    if chars.next().map(|(_, c)| c) != Some(':') {
        return Ok(None);
    }
    current_pos += 1;

    // Parse Kind
    let kind_start = current_pos;
    let mut kind_end = kind_start;
    while let Some((i, c)) = chars.peek() {
        if c.is_alphanumeric() {
            kind_end = begin + i + c.len_utf8();
            chars.next();
        } else {
            break;
        }
    }
    let kind_str = &document[kind_start..kind_end];
    let kind = match kind_str {
        "Begin" => Kind::Begin,
        "End" => Kind::End,
        _ => return Ok(None), // Unknown kind
    };
    current_pos = kind_end;

    let opening = AnchorOpening {
        command,
        uuid,
        kind,
        range: Range { begin, end: current_pos },
    };

    let parameters = parse_parameters(document, current_pos)?;
    let arguments = parse_arguments(document, parameters.range.end)?;

    let mut end = arguments.range.end;

    // Consume " -->"
    let closing_sequence = " -->";
    if document[end..].starts_with(closing_sequence) {
        end += closing_sequence.len();
    } else {
        return Err(ParsingError::UnexpectedToken("Expected ' -->'".to_string()));
    }

    Ok(Some(Anchor {
        opening,
        parameters,
        arguments,
        range: Range { begin, end },
    }))
}

fn parse_parameters(document: &str, begin: usize) -> Result<Parameters, ParsingError>
{
    let s = &document[begin..];
    if !s.starts_with('{') {
        return Ok(Parameters {
            parameters: serde_json::Value::Object(serde_json::Map::new()),
            range: Range { begin, end: begin },
        });
    }

    let mut brace_count = 0;
    let mut end = begin;
    let mut found_closing_brace = false;

    for (i, c) in s.char_indices() {
        if c == '{' {
            brace_count += 1;
        } else if c == '}' {
            brace_count -= 1;
        }

        if brace_count == 0 && c == '}' {
            end = begin + i + 1;
            found_closing_brace = true;
            break;
        }
    }

    if !found_closing_brace {
        return Err(ParsingError::UnexpectedToken("Unmatched opening brace for parameters".to_string()));
    }

    let json_str = &document[begin..end];
    let parameters: serde_json::Value = serde_json::from_str(json_str)?;

    Ok(Parameters {
        parameters,
        range: Range { begin, end },
    })
}

fn parse_arguments(document: &str, begin: usize) -> Result<Arguments, ParsingError>
{
    let mut arguments = Vec::new();

    let end_of_line = document[begin..].find('\n').map_or(document.len(), |i| begin + i);
    let mut position = begin;

    while position < end_of_line {
        // Skip whitespace
        let remaining = &document[position..];
        if let Some(first_char) = remaining.chars().next() {
            if first_char.is_whitespace() {
                position += first_char.len_utf8();
                continue;
            }
        }

        // Check for end of arguments (e.g., closing brace of parameters)
        if document[position..].starts_with("}") {
            break;
        }

        let argument = parse_argument(document, position)?;
        position = argument.range.end;
        arguments.push(argument);
    }

    Ok(Arguments{
        children: arguments,
        range: Range{begin, end: position},
    })
}

fn parse_argument(document: &str, begin: usize) -> Result<Argument, ParsingError>
{
    let s = &document[begin..];
    let mut end = begin;

    if s.is_empty() {
        return Err(ParsingError::EndOfDocument);
    }

    let first_char = s.chars().next().unwrap();

    if first_char == '\'' || first_char == '"' {
        // Quoted string
        let quote_char = first_char;
        let mut chars = s.char_indices().peekable();
        chars.next(); // Consume the opening quote

        end = begin + 1; // Start after the opening quote

        while let Some((i, c)) = chars.next() {
            if c == '\\' {
                // Handle escape sequence
                chars.next(); // Consume the escaped character
                end = begin + i + 2;
            } else if c == quote_char {
                // Closing quote
                end = begin + i + 1;
                break;
            } else {
                end = begin + i + 1;
            }
        }

        if document.chars().nth(end - 1) != Some(quote_char) {
            return Err(ParsingError::UnexpectedToken(format!("Unclosed string literal starting at position {}", begin)));
        }

    } else {
        // Single word or token
        for (i, c) in s.char_indices() {
            if c.is_whitespace() || c == '}' || c == ',' {
                end = begin + i;
                break;
            } else {
                end = begin + i + 1;
            }
        }
    }

    if end == begin {
        return Err(ParsingError::ParsingNotAdvanced(begin));
    }

    Ok(Argument {
        range: Range { begin, end },
    })
}

fn parse_text(document: &str, begin: usize) -> Result<Option<Text>, ParsingError>
{
    let mut end = begin;
    let mut found_content = false;

    for (i, c) in document[begin..].char_indices() {
        let current_pos = begin + i;
        // Check if the current position starts a tag or an anchor
        if document[current_pos..].starts_with("@") || document[current_pos..].starts_with("<!-- ") {
            break;
        }
        end = current_pos + c.len_utf8();
        found_content = true;
    }

    if found_content {
        Ok(Some(Text {
            range: Range { begin, end },
        }))
    } else {
        Ok(None)
    }
}