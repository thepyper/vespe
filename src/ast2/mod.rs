use std::str::Chars;
use uuid::Uuid;
use serde_json::{json, Value};
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ParsingError {
    #[error("Unexpected token: expected {expected}, found {found} at {range:?}")]
    UnexpectedToken {
        expected: String,
        found: String,
        range: Range,
    },
    #[error("Invalid syntax: {message} at {range:?}")]
    InvalidSyntax {
        message: String,
        range: Range,
    },
    #[error("Unexpected end of file: expected {expected} at {range:?}")]
    EndOfFileUnexpected {
        expected: String,
        range: Range,
    },
    #[error("Invalid number format: {value} at {range:?}")]
    InvalidNumberFormat {
        value: String,
        range: Range,
    },
    #[error("Unterminated string at {range:?}")]
    UnterminatedString {
        range: Range,
    },
    #[error("Custom parsing error: {message} at {range:?}")]
    Custom {
        message: String,
        range: Range,
    },
}

pub type Result<T> = std::result::Result<T, ParsingError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position {
    pub offset: usize,      /// 0-based character offset
    pub line: usize,        /// 1-based line
    pub column: usize,      /// 1-based column
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Range {
    pub begin: Position,
    pub end: Position,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Text {
    pub range: Range,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandKind {
    Tag,        // for debug purpose
    Include,
    Inline,
    Answer,
    Summarize,
    Derive,
    Repeat,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameters {
    pub parameters: serde_json::Value,
    pub range: Range,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Argument {
    pub value: String,
    pub range: Range,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Arguments {
    pub arguments: Vec<Argument>,
    pub range: Range,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tag {
    pub command: CommandKind,
    pub parameters: Option<Parameters>,
    pub arguments: Option<Arguments>,
    pub range: Range,   
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnchorKind 
{
    Begin,
    End,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Anchor {
    pub command: CommandKind,
    pub uuid: Uuid,
    pub kind: AnchorKind,
    pub parameters: Option<Parameters>,
    pub arguments: Option<Arguments>,
    pub range: Range,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Content {
    Text(Text),
    Tag(Tag),
    Anchor(Anchor),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub content: Vec<Content>,
}

pub struct Parser<'a> {
    document: &'a str,
    position: Position,
    iterator: Chars<'a>,
}

#[derive(Debug, Clone)]
pub struct ParserStatus<'a> {
    pub position: Position,
    pub iterator: Chars<'a>,
}

impl <'a> Parser<'a> {
    pub fn new(document: &'a str) -> Self {
        Self {
            document,
            position: Position { offset: 0, line: 1, column: 1 },
            iterator: document.chars(),
        }
    }
    pub fn get_position(&self) -> Position {
        self.position.clone()
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
    pub fn is_begin_of_line(&self) -> bool {
        self.position.column == 1
    }
    pub fn consume(&mut self, xs: &str) -> bool {
        if !self.remain().starts_with(xs) {
            return false;
        }
        for x in xs.chars() {
            self.advance();
        }
        true
    }    
    pub fn skip_one_of(&mut self, xs: &str) -> bool {
        let status = self.store();
        for x in xs.chars() {
            match self.advance() {
                None => {
                    return false;
                }
                Some(y) => {
                    if y == x {
                        return true;
                    }
                }
            }
            self.load(&status);
        }
        false
    }
    pub fn skip_many_of(&mut self, xs: &str) {
        while self.skip_one_of(xs) {}
    }
    pub fn skip_many_whitespaces(&mut self) {
        self.skip_many_of(" \t\r");
    }
    pub fn skip_many_whitespaces_or_eol(&mut self) {
        self.skip_many_of(" \t\r\n");
    }
    pub fn advance(&mut self) -> Option<char> {
        match self.iterator.next() {
            None => None,
            Some(c) => {
                self.position.offset += 1;
                match c {
                    c if c = '\n' => {
                        self.position.line += 1;
                        self.position.column = 1;
                    }
                    _ => {
                        self.position.column += 1;
                    }
                }
                Some(c)                
            }
        }
    }
    pub fn store(&self) -> ParserStatus {
        ParserStatus {
            position: self.position.clone(),
            iterator: self.iterator.clone(),
        }
    }
    pub fn load(&mut self, status: &ParserStatus) {
        self.position = status.position;
        self.iterator = status.iterator;
    }
}

pub fn parse_document(document: &str) -> Result<Document> {

    let mut parser = Parser::new(document);
    let content = parse_content(&mut parser)?;

    Ok(Document {
        content,
    })
}

fn parse_content(parser: &mut Parser) -> Result<Vec<Content>> {

    let mut contents = Vec::new();

    while !parser.is_eod() {
        if let Some(tag) = try_parse_tag(document, parser)? {
            contents.push(Tag(tag));            
        } else if let Some(anchor) = try_parse_anchor(document, parser)? {
            contents.push(Anchor(anchor));
        } else if let Some(text) = try_parse_text(document, parser)? {
            contents.push(Text(text));
        } else {
            // TODO parse error
        }
    }

    Ok(contents)
}

fn try_parse_tag(document: &str, parser: &mut Parser) -> Result<Option<Tag>> {

    let status = parser.store();

    match _try_parse_tag(parser)? {
        None => {
            parser.load(status);
            None           
        }
        Some(x) => Some(x)
    }    
} 

fn _try_parse_tag(parser: &mut Parser) -> Result<Option<Tag>> {

    let begin = parser.get_position();

    if !parser.consume("@") {
        return Ok(None);
    }

    let command = _try_parse_command_kind(parser)?;
    if command.is_none() {
        return Ok(None);
    }

    parser.skip_many_whitespaces();

    let parameters = _try_parse_parameters(document, parser)?;
    
    parser.skip_many_whitespaces();

    let arguments = _try_parse_arguments(document, parser)?;

    parser.skip_many_whitespaces();

    if !parser.consume("\n") {
        // TODO errore, text dopo arguments e prima di fine linea!?
    }

    let end = parser.get_position();

    Ok(Some(Tag {
        command,
        parameters,
        arguments,
        range: Range {
            begin, end 
        }
    }))
}

fn try_parse_anchor(document: &str, parser: &mut Parser) -> Result<Option<Anchor>> {

    let status = parser.store();

    match _try_parse_anchor(parser)? {
        None => {
            parser.load(status);
            None           
        }
        Some(x) => Some(x)
    }    
}

fn _try_parse_anchor(parser: &mut Parser) -> Result<Option<Anchor>> {

    let begin = parser.get_position();

    if !parser.consume("<!--") {
        return Ok(None);
    }

    parser.skip_many_whitespaces();

    let command = _try_parse_command_kind(parser)?;
    if command.is_none() {
        return Ok(None);
    }

    if !parser.consume("-") {
        // TODO parsing error anchor, manca trattino prima di uuid
    }

    let uuid = _try_parse_uuid(document, parser)?;
    if uuid.is_none() {
        // TODO parsing error anchor, manca uuid
    }

    if !parser.consume(":") {
        // TODO parsing error anchor, manca :
    }

    let kind = _try_parse_anchor_kind(parser)?;

    parser.skip_many_whitespaces();

    let parameters = _try_parse_parameters(document, parser)?;
    
    parser.skip_many_whitespaces();

    let arguments = _try_parse_arguments(document, parser)?;

    parser.skip_many_whitespaces_or_eol();

    if !parser.consume("-->") {
        // TODO errore, ancora non chiusa
    }

    parser.skip_many_whitespaces();

    if !parser.consume("\n") {
        // TODO errore, text dopo arguments e prima di fine linea!?
    }

    let end = parser.get_position();

    Ok(Some(Anchor {
        command,
        uuid,
        kind,
        parameters,
        arguments,
        range: Range {
            begin, end 
        }
    }))
}

fn _try_parse_command_kind(parser: &mut Parser) -> Result<Option<CommandKind>> {

    let tags_list = vec![
        ("tag", CommandKind::Tag),
        ("include", CommandKind::Include),
        ("inline", CommandKind::Inline),
        ("answer", CommandKind::Answer),
        ("summarize", CommandKind::Summarize),
        ("derive", CommandKind::Derive),
        ("repeat", CommandKind::Repeat),
    ];

    let status = parser.store();
    for (name, kind) in tags_list {
        if !parser.consume(name) {
            parser.load(status);
        } else {
            return Some(kind);
        }
    }

    kind
}

fn _try_parse_anchor_kind(parser: &mut Parser) -> Result<Option<AnchorKind>> {

    let tags_list = vec![
        ("begin", AnchorKind::Begin),
        ("end", AnchorKind::End),
    ];

    let status = parser.store();
    for (name, kind) in tags_list {
        if !parser.consume(name) {
            parser.load(status);
        } else {
            return Some(kind);
        }
    }

    None
}

fn _try_parse_parameters(parser: &mut Parser) -> Result<Option<Parameters>> {

    let status = parser.store();

    if let Some(x) = _try_parse_parameters0(parser)? {
        return Some(x);
    }

    parser.load(status);
    None
}

fn _try_parse_parameters0(parser: &mut Parser) -> Result<Option<Parameters>> {

    let begin = parser.get_position();

    if !parser.consume("{") {
        return Ok(None);
    } 

    let mut parameters = json!({});

    while !parser.is_eod() {

        parser.skip_many_whitespaces_or_eol();

        if parser.consume("}") {
            break;
        } 
        
        let parameter = _try_parse_parameter(parser)?;

        if parameter.is_none() {
            // TODO errore, parametro non parsed!?!?
        }

        // TODO add parameter to serde_json object

    }

    let end = parser.get_position();

    Ok(Parameters { parameters, range: Range { begin, end }})
}

