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
    // ASSERT: begin.column == 1 altrimenti errore, partenza tag e' SEMPRE ad inizio linea

    // TODO: parse di un tag, fatto da:
    // 1) parse di    @<nome-tag> 
    // 2) call di parse_parameters che fa parsing di {} oggetto JSON (possibile che non ci sia, allora parameters e' un oggetto vuoto {})
    // 3) call di parse_arguments che fa il parsing del resto della linea dove e' finito il JSON con }, e separa le words in diversi argument; gestire ', e " per accorpare
    // ritornare struttura Tag, completa di calcolo del Range che comprende tutto il Tag compreso fine-linea
    Ok(None) // Placeholder
}

fn parse_anchor(document: &str, begin: usize) -> Result<Option<Anchor>, ParsingError>
{
    // ASSERT: begin.column == 1 altrimenti errore, partenza anchor e' SEMPRE ad inizio linea

    // TODO: parse di una anchor, fatto da:
    // 1) parse di <!-- <nome-tag>-<uuid>:<kind> 
    // 2) call di parse_parameters, come in parse_tag 
    // 3) call di parse_arguments, come in parse_tag
    // 4) parse di -->
    // ritornare struttura Anchor, completa di calcolo del Range che comprende tutto il Tag compreso fine-linea
    Ok(None) // Placeholder
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
    // TODO: Implement parse_text
    Ok(None) // Placeholder
}