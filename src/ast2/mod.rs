use clap::builder::Str;
use serde_json::json;
use std::str::Chars;
use std::str::FromStr;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Copy)]
struct Position {
    /// 0-based character offset
    offset: usize,
    /// 1-based line
    line: usize,
    /// 1-based column
    column: usize,
}

impl Position {
    fn null() -> Self {
        Position {
            offset: 0,
            line: 0,
            column: 0,
        }
    }
    fn is_valid(&self) -> bool {
        (self.line > 0) && (self.column > 0)
    }
}

#[derive(Debug, Clone, Copy)]
struct Range {
    begin: Position,
    end: Position,
}

impl Range {
    fn null() -> Self {
        Range {
            begin: Position::null(),
            end: Position::null(),
        }
    }
    fn is_valid(&self) -> bool {
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
}

pub type Result<T> = std::result::Result<T, Ast2Error>;

struct Text {
    range: Range,
}

enum CommandKind {
    Tag, // for debug purpose
    Include,
    Inline,
    Answer,
    Summarize,
    Derive,
    Repeat,
}

struct Parameters {
    parameters: serde_json::Map<String, serde_json::Value>,
    range: Range,
}

impl Parameters {
    fn new() -> Self {
        Parameters {
            parameters: serde_json::Map::new(),
            range: Range::null(),
        }
    }
}

struct Argument {
    value: String,
    range: Range,
}

struct Arguments {
    arguments: Vec<Argument>,
    range: Range,
}

struct Tag {
    command: CommandKind,
    parameters: Parameters,
    arguments: Arguments,
    range: Range,
}

enum AnchorKind {
    Begin,
    End,
}

struct Anchor {
    command: CommandKind,
    uuid: Uuid,
    kind: AnchorKind,
    parameters: Parameters,
    arguments: Arguments,
    range: Range,
}

enum Content {
    Text(Text),
    Tag(Tag),
    Anchor(Anchor),
}

struct Document {
    content: Vec<Content>,
    range: Range,
}

#[derive(Clone)]
pub struct Parser<'a> {
    document: &'a str,
    position: Position,
    iterator: Chars<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(document: &'a str) -> Self {
        Self {
            document,
            position: Position {
                offset: 0,
                line: 1,
                column: 1,
            },
            iterator: document.chars(),
        }
    }
    pub fn get_position(&self) -> Position {
        self.position.clone()
    }
    pub fn get_checkpoint(&self) -> Position {
        self.position.clone()
    }
    pub fn restore_checkpoint(&mut self, checkpoint: Position) {
        self.position = checkpoint;
        self.iterator = self.document[self.position.offset..].chars();
    }
    pub fn get_offset(&self) -> usize {
        self.position.offset
    }
    pub fn remain(&self) -> &'a str {
        self.iterator.as_str()
    }
    pub fn is_eod(&self) -> bool {
        self.remain().is_empty()
    }
    pub fn is_eol(&self) -> bool {
        self.remain().starts_with("\n")
    }
    pub fn is_begin_of_line(&self) -> bool {
        self.position.column == 1
    }
    pub fn consume_matching_string(&mut self, xs: &str) -> Option<String> {
        if !self.remain().starts_with(xs) {
            None
        } else {
            for x in xs.chars() {
                self.advance();
            }
            Some(xs.into())
        }
    }
    pub fn consume_matching_char(&mut self, x: char) -> Option<char> {
        self.consume_char_if(|y| x == y)
    }
    pub fn consume_char_if<F>(&mut self, filter: F) -> Option<char>
    where
        F: FnOnce(char) -> bool,
    {
        match self.remain().chars().next() {
            None => None,
            Some(y) => {
                if !filter(y) {
                    None
                } else {
                    self.advance()
                }
            }
        }
    }
    pub fn consume_many_if<F>(&mut self, filter: F) -> Option<String>
    where
        F: Fn(char) -> bool,
    {
        let mut xs = String::new();
        loop {
            match self.consume_char_if(|c| filter(c)) {
                None => {
                    break;
                }
                Some(x) => xs.push(x),
            }
        }
        if xs.is_empty() {
            None
        } else {
            Some(xs)
        }
    }
    fn consume_many_of(&mut self, xs: &str) -> Option<String> {
        self.consume_many_if(|y| xs.contains(y))
    }
    pub fn skip_many_whitespaces(&mut self) {
        let _ = self.consume_many_of(" \t\r");
    }
    pub fn skip_many_whitespaces_or_eol(&mut self) {
        let _ = self.consume_many_of(" \t\r\n");
    }
    pub fn advance(&mut self) -> Option<char> {
        match self.iterator.next() {
            None => None,
            Some(c) => {
                self.position.offset += 1;
                if c == '\n' {
                    self.position.line += 1;
                    self.position.column = 1;
                } else {
                    self.position.column += 1;
                }
                Some(c)
            }
        }
    }
}

fn parse_document(document: &str) -> Result<Document> {
    let mut parser = Parser::new(document);
    let begin = parser.get_position();
    let (content, end) = parse_content(&mut parser)?;

    Ok(Document {
        content: content,
        range: Range { begin, end }, 
    })
}

fn parse_content<'a>(parser: &'a mut Parser<'a>) -> Result<(Vec<Content>, Position)> {
    let mut contents = Vec::new();

    loop {
        let result = {
            if parser.is_eod() {
                break;
            }

            let mut parsed_something = false;
            let mut current_result: Option<Content> = None;

            // Attempt to parse a tag
            match _try_parse_tag(parser) {
                Ok(Some(tag)) => {
                    current_result = Some(Content::Tag(tag));
                    parsed_something = true;
                }
                Ok(None) => { /* No tag found, try next */ }
                Err(e) => return Err(e),
            }

            if !parsed_something {
                // Attempt to parse an anchor
                match _try_parse_anchor(parser) {
                    Ok(Some(anchor)) => {
                        current_result = Some(Content::Anchor(anchor));
                        parsed_something = true;
                    }
                    Ok(None) => { /* No anchor found, try next */ }
                    Err(e) => return Err(e),
                }
            }

            if !parsed_something {
                // Attempt to parse text
                match _try_parse_text(parser) {
                    Ok(Some(text)) => {
                        current_result = Some(Content::Text(text));
                        parsed_something = true;
                    }
                    Ok(None) => { /* No text found, this should ideally not happen if other parsers failed */ }
                    Err(e) => return Err(e),
                }
            }
            
            if !parsed_something {
                // If none of the above parsed anything, it's an error
                return Err(Ast2Error::ParsingError {
                    position: parser.get_position(),
                    message: "Unable to parse content".to_string(),
                });
            }
            current_result
        };

        if let Some(content) = result {
            contents.push(content);
        }
    }

    let end = parser.get_position();
    Ok((contents, end))
}

fn _try_parse_tag<'a>(parser: &'a mut Parser<'a>) -> Result<Option<Tag>> {
    let checkpoint = parser.get_checkpoint();
    match _try_parse_tag0(parser) {
        Ok(Some(tag)) => Ok(Some(tag)),
        Ok(None) => {
            parser.restore_checkpoint(checkpoint);
            Ok(None)
        },
        Err(e) => {
            parser.restore_checkpoint(checkpoint);
            Err(e)
        },
    }
}

fn _try_parse_tag0<'a>(parser: &'a mut Parser<'a>) -> Result<Option<Tag>> {
    let begin = parser.get_position();

    if parser.consume_matching_char('@').is_none() {
        return Ok(None);
    }

    let command = _try_parse_command_kind(parser)?;
    let command = match command {
        Some(c) => c,
        None => return Ok(None),
    };

    parser.skip_many_whitespaces();

    let parameters = _try_parse_parameters(parser)?;
    let parameters = match parameters {
        Some(p) => p,
        None => Parameters::new(),
    };

    parser.skip_many_whitespaces();

    let arguments = _try_parse_arguments(parser)?;
    let arguments = match arguments {
        Some(a) => a,
        None => Arguments {
            arguments: Vec::new(),
            range: Range::null(),
        },
    };

    parser.skip_many_whitespaces();

    // Consuma EOL se c'e', altrimenti siamo a fine documento
    parser.consume_matching_char('\n');

    let end = parser.get_position();

    Ok(Some(Tag {
        command,
        parameters,
        arguments,
        range: Range { begin, end },
    }))
}

fn _try_parse_anchor<'a>(parser: &'a mut Parser<'a>) -> Result<Option<Anchor>> {
    let checkpoint = parser.get_checkpoint();
    match _try_parse_anchor0(parser) {
        Ok(Some(anchor)) => Ok(Some(anchor)),
        Ok(None) => {
            parser.restore_checkpoint(checkpoint);
            Ok(None)
        },
        Err(e) => {
            parser.restore_checkpoint(checkpoint);
            Err(e)
        },
    }
}

fn _try_parse_anchor0<'a>(parser: &'a mut Parser<'a>) -> Result<Option<Anchor>> {
    let begin = parser.get_position();

    if parser.consume_matching_string("<!--").is_none() {
        return Ok(None);
    }

    parser.skip_many_whitespaces();

    let command = _try_parse_command_kind(parser)?;
    let command = match command {
        Some(c) => c,
        None => return Ok(None),
    };

    if parser.consume_matching_char('-').is_none() {
        return Err(Ast2Error::ParsingError {
            position: parser.get_position(),
            message: "Expected '-' before UUID in anchor".to_string(),
        });
    }

    let uuid = _try_parse_uuid(parser)?;
    let uuid = match uuid {
        Some(u) => u,
        None => {
            return Err(Ast2Error::InvalidUuid {
                position: parser.get_position(),
            })
        }
    };

    if parser.consume_matching_char(':').is_none() {
        return Err(Ast2Error::ParsingError {
            position: parser.get_position(),
            message: "Expected ':' after UUID in anchor".to_string(),
        });
    }

    let kind = _try_parse_anchor_kind(parser)?;
    let kind = match kind {
        Some(k) => k,
        None => {
            return Err(Ast2Error::InvalidAnchorKind {
                position: parser.get_position(),
            })
        }
    };

    parser.skip_many_whitespaces();

    let parameters = match _try_parse_parameters(parser)? {
        Some(x) => x,
        None => Parameters::new(),
    };

    parser.skip_many_whitespaces();

    let arguments = _try_parse_arguments(parser)?;
    let arguments = match arguments {
        Some(a) => a,
        None => Arguments {
            arguments: Vec::new(),
            range: Range::null(),
        },
    };

    parser.skip_many_whitespaces_or_eol();

    if parser.consume_matching_string("-->").is_none() {
        return Err(Ast2Error::UnclosedString {
            position: parser.get_position(),
        });
    }

    parser.skip_many_whitespaces();

    parser.consume_matching_char('\n');

    let end = parser.get_position();

    Ok(Some(Anchor {
        command,
        uuid,
        kind,
        parameters,
        arguments,
        range: Range { begin, end },
    }))
}

fn _try_parse_command_kind<'a>(parser: &'a mut Parser<'a>) -> Result<Option<CommandKind>> {
    let tags_list = vec![
        ("tag", CommandKind::Tag),
        ("include", CommandKind::Include),
        ("inline", CommandKind::Inline),
        ("answer", CommandKind::Answer),
        ("summarize", CommandKind::Summarize),
        ("derive", CommandKind::Derive),
        ("repeat", CommandKind::Repeat),
    ];

    for (name, kind) in tags_list {
        if parser.consume_matching_string(name).is_some() {
            return Ok(Some(kind));
        }
    }

    Ok(None)
}

fn _try_parse_anchor_kind<'a>(parser: &'a mut Parser<'a>) -> Result<Option<AnchorKind>> {
    let tags_list = vec![("begin", AnchorKind::Begin), ("end", AnchorKind::End)];

    for (name, kind) in tags_list {
        if parser.consume_matching_string(name).is_some() {
            return Ok(Some(kind));
        }
    }

    Ok(None)
}

fn _try_parse_parameters<'a>(parser: &'a mut Parser<'a>) -> Result<Option<Parameters>> {
    let checkpoint = parser.get_checkpoint();
    match _try_parse_parameters0(parser) {
        Ok(Some(parameters)) => Ok(Some(parameters)),
        Ok(None) => {
            parser.restore_checkpoint(checkpoint);
            Ok(None)
        },
        Err(e) => {
            parser.restore_checkpoint(checkpoint);
            Err(e)
        },
    }
}

fn _try_parse_parameters0<'a>(parser: &'a mut Parser<'a>) -> Result<Option<Parameters>> {
    let begin = parser.get_position();

    if parser.consume_matching_char('{').is_none() {
        return Ok(None);
    }

    parser.skip_many_whitespaces_or_eol();

    if parser.consume_matching_char('}').is_some() {
        return Ok(Some(Parameters {
            parameters: serde_json::Map::new(),
            range: Range {
                begin,
                end: parser.get_position(),
            },
        }));
    }

    let mut parameters_map = serde_json::Map::new();
    let mut first_param = true;

    loop {
        if !first_param {
            if parser.consume_matching_char(',').is_none() {
                return Err(Ast2Error::MissingCommaInParameters {
                    position: parser.get_position(),
                });
            }
            parser.skip_many_whitespaces_or_eol();
        }

        let parameter = _try_parse_parameter(parser)?;
        let (key, value) = match parameter {
            Some(p) => p,
            None => {
                return Err(Ast2Error::ParameterNotParsed {
                    position: parser.get_position(),
                })
            }
        };
        parameters_map.insert(key, value);

        parser.skip_many_whitespaces_or_eol();

        if parser.consume_matching_char('}').is_some() {
            break;
        }
        first_param = false;
    }

    let end = parser.get_position();

    Ok(Some(Parameters {
        parameters: parameters_map,
        range: Range { begin, end },
    }))
}

fn _try_parse_parameter<'a>(
    parser: &'a mut Parser<'a>,
) -> Result<Option<(String, serde_json::Value)>> {
    let begin = parser.get_position();

    let key = _try_parse_identifier(parser)?;
    let key = match key {
        Some(k) => k,
        None => {
            return Err(Ast2Error::MissingParameterKey {
                position: parser.get_position(),
            })
        }
    };

    parser.skip_many_whitespaces_or_eol();

    if parser.consume_matching_char(':').is_none() {
        return Err(Ast2Error::MissingParameterColon {
            position: parser.get_position(),
        });
    }

    parser.skip_many_whitespaces_or_eol();

    let value = _try_parse_value(parser)?;
    let value = match value {
        Some(v) => v,
        None => {
            return Err(Ast2Error::MissingParameterValue {
                position: parser.get_position(),
            })
        }
    };

    let end = parser.get_position();

    Ok(Some((key, value)))
}

fn _try_parse_arguments<'a>(parser: &'a mut Parser<'a>) -> Result<Option<Arguments>> {
    let checkpoint = parser.get_checkpoint();
    match _try_parse_arguments0(parser) {
        Ok(Some(arguments)) => Ok(Some(arguments)),
        Ok(None) => {
            parser.restore_checkpoint(checkpoint);
            Ok(None)
        },
        Err(e) => {
            parser.restore_checkpoint(checkpoint);
            Err(e)
        },
    }
}

fn _try_parse_arguments0<'a>(parser: &'a mut Parser<'a>) -> Result<Option<Arguments>> {
    let begin = parser.get_position();

    let mut arguments = Vec::new();

    loop {
        parser.skip_many_whitespaces();

        // Check for anchor end
        if parser.remain().starts_with("-->") {
            break;
        }

        match _try_parse_argument(parser)? {
            Some(x) => arguments.push(x),
            None => break,
        }
    }

    if arguments.is_empty() {
        return Ok(None);
    }

    let end = parser.get_position();

    Ok(Some(Arguments {
        arguments,
        range: Range { begin, end },
    }))
}

fn _try_parse_argument<'a>(parser: &'a mut Parser<'a>) -> Result<Option<Argument>> {
    let begin = parser.get_position();

    let value = if let Some(x) = _try_parse_enclosed_string(parser, "\'")? {
        Some(x)
    } else if let Some(x) = _try_parse_enclosed_string(parser, "\"")? {
        Some(x)
    } else if let Some(x) = _try_parse_nude_string(parser)? {
        Some(x)
    } else {
        None
    };

    let end = parser.get_position();

    Ok(value.map(|value| Argument { value, range: Range {begin, end}}))
}

fn _try_parse_identifier<'a>(parser: &'a mut Parser<'a>) -> Result<Option<String>> {
    let mut identifier = String::new();

    match parser.consume_char_if(|c| c.is_alphabetic() || c == '_') {
        Some(x) => {
            identifier.push(x);
            match parser.consume_many_if(|c| c.is_alphanumeric() || c == '_') {
                Some(x) => {
                    identifier.push_str(&x);
                }
                None => {}
            }
        }
        None => {}
    }

    if identifier.is_empty() {
        return Ok(None);
    } else {
        return Ok(Some(identifier));
    }
}

fn _try_parse_value<'a>(parser: &'a mut Parser<'a>) -> Result<Option<serde_json::Value>> {
    match parser.consume_matching_char('"') {
        Some(_) => _try_parse_enclosed_value(parser, "\""),
        None => match parser.consume_matching_char('\'') {
            Some(_) => _try_parse_enclosed_value(parser, "\'"),
            None => _try_parse_nude_value(parser)
        }
    }
}

fn _try_parse_enclosed_value<'a>(
    parser: &'a mut Parser<'a>,
    closure: &str,
) -> Result<Option<serde_json::Value>> {
    _try_parse_enclosed_string(parser, closure).and_then(|x| match x {
        Some(s) => Ok(Some(serde_json::Value::String(s))),
        None => Ok(Some(serde_json::Value::Null)),
    })
}

fn _try_parse_enclosed_string<'a>(
    parser: &'a mut Parser<'a>,
    closure: &str,
) -> Result<Option<String>> {
    let begin_pos = parser.get_position();
    let mut value = String::new();

    loop {
        if parser.consume_matching_string("\\\"").is_some() {
            value.push('\"');
        } else if parser.consume_matching_string("\\\'").is_some() {
            value.push('\'');
        } else if parser.consume_matching_string("\\n").is_some() {
            value.push('\n');
        } else if parser.consume_matching_string("\\r").is_some() {
            value.push('\r');
        } else if parser.consume_matching_string("\\t").is_some() {
            value.push('\t');
        } else if parser.consume_matching_string("\\\\").is_some() {
            value.push('\\');
        } else if parser.consume_matching_string(closure).is_some() {
            return Ok(Some(value));
        } else {
            match parser.advance() {
                None => {
                    return Err(Ast2Error::UnclosedString {
                        position: begin_pos,
                    });
                }
                Some(x) => {
                    value.push(x);
                }
            }
        }
    }
}

fn _try_parse_nude_value<'a>(parser: &'a mut Parser<'a>) -> Result<Option<serde_json::Value>> {
    if let Some(x) = _try_parse_nude_integer(parser)? {
        return Ok(Some(json!(x)));
    } 
    if let Some(x) = _try_parse_nude_float(parser)? {
        return Ok(Some(json!(x)));
    } 
    if let Some(x) = _try_parse_nude_bool(parser)? {
        return Ok(Some(json!(x)));
    } 
    if let Some(x) = _try_parse_nude_string(parser)? {
        return Ok(Some(json!(x)));
    } 
    return Err(Ast2Error::MalformedValue {
        position: parser.get_position(),
    });
}

fn _try_parse_nude_integer<'a>(parser: &'a mut Parser<'a>) -> Result<Option<i64>> {
    let number_str_option = parser.consume_many_if(|x| x.is_digit(10));

    match number_str_option {
        Some(number_str) => match i64::from_str_radix(&number_str, 10) {
            Ok(num) => Ok(Some(num)),
            Err(e) => Err(Ast2Error::ParseIntError(e)),
        },
        None => Ok(None),
    }
}

fn _try_parse_nude_float<'a>(parser: &'a mut Parser<'a>) -> Result<Option<f64>> {
    let mut number_str = String::new();

    let integer_part = parser.consume_many_if(|x| x.is_digit(10));
    if let Some(s) = integer_part {
        number_str.push_str(&s);
    }

    if parser.consume_matching_char('.').is_some() {
        number_str.push('.');
        let fractional_part = parser.consume_many_if(|x| x.is_digit(10));
        if let Some(s) = fractional_part {
            number_str.push_str(&s);
        } else {
            number_str.push('0');
        }
    } 

    if number_str.is_empty() {
        Ok(None)
    } else {
        match f64::from_str(&number_str) {
            Ok(num) => Ok(Some(num)),
            Err(e) => Err(Ast2Error::ParseFloatError(e)),
        }
    }
}

fn _try_parse_nude_bool<'a>(parser: &'a mut Parser<'a>) -> Result<Option<bool>> {
    if parser.consume_matching_string("true").is_some() {
        return Ok(Some(true));
    } else if parser.consume_matching_string("false").is_some() {
        return Ok(Some(false));
    } else {
        return Ok(None);
    }
}

fn _try_parse_nude_string<'a>(parser: &'a mut Parser<'a>) -> Result<Option<String>> {
    let xs = parser.consume_many_if(|x| x.is_alphanumeric() || x == '/' || x == '.' || x == '_');
    if xs.is_none() {
        return Ok(None);
    } else {
        return Ok(xs);
    }
}

fn _try_parse_uuid<'a>(parser: &'a mut Parser<'a>) -> Result<Option<Uuid>> {
    let start_pos = parser.get_position();
    let uuid_str_option = parser.consume_many_if(|c| c.is_ascii_hexdigit() || c == '-');

    match uuid_str_option {
        Some(uuid_str) => match Uuid::parse_str(&uuid_str) {
            Ok(uuid) => Ok(Some(uuid)),
            Err(_) => Err(Ast2Error::InvalidUuid {
                position: start_pos,
            }),
        },
        None => Ok(None),
    }
}

fn _try_parse_text<'a>(parser: &'a mut Parser<'a>) -> Result<Option<Text>> {
    let begin = parser.get_position();

    let mut content = String::new();
    
    /*  Note: This functions stops on newline on purpose, to avoid
        checking conditions that occur on begin of the line here,
        but return to the upper parser loop which will
        continue checking those. It's acceptable that Text
        becomes broken into single-lines, it's not a problem. */
    loop {
        match parser.advance() {
            None => {
                break;
            }
            Some('\n') => {
                content.push('\n');
                break;
            }
            Some(x) => {
                content.push(x);
            }
        }
    }

    let end = parser.get_position();

    Ok(Some(Text {
        range: Range { begin, end },
    }))
}
