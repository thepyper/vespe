use std::str::Chars;
use uuid::Uuid;
use serde_json::json;
use thiserror::Error;
use anyhow::Result;

#[derive(Error, Debug)]
pub enum ParsingError {
    #[error("Unexpected token: expected {expected}, found {found} at {range:?}")]
    UnexpectedToken { expected: String, found: String, range: Range },
    #[error("Invalid syntax: {message} at {range:?}")]
    InvalidSyntax { message: String, range: Range },
    #[error("Unexpected end of file: expected {expected} at {range:?}")]
    EndOfFileUnexpected { expected: String, range: Range },
    #[error("Invalid number format: {value} at {range:?}")]
    InvalidNumberFormat { value: String, range: Range },
    #[error("Unterminated string at {range:?}")]
    UnterminatedString { range: Range },
    #[error("Custom error: {message} at {range:?}")]
    Custom { message: String, range: Range },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position {
    pub offset: usize,      /// 0-based character offset
    pub line: usize,        /// 1-based line
    pub column: usize, // 1-based column
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Range {
    pub begin: Position,
    pub end: Position,
}

pub struct Text {
    pub range: Range,
}

pub enum CommandKind {
    Tag,        // for debug purpose
    Include,
    Inline,
    Answer,
    Summarize,
    Derive,
    Repeat,
}

pub struct Parameters {
    pub parameters: serde_json::Value,
    pub range: Range,
}

pub struct Argument {
    pub value: String,
    pub range: Range,
}

pub struct Arguments {
    pub arguments: Vec<Argument>,
    pub range: Range,
}

pub struct Tag {
    pub command: CommandKind,
    pub parameters: Parameters,
    pub arguments: Arguments,
    pub range: Range,   
}

pub enum AnchorKind 
{
    Begin,
    End,
}

pub struct Anchor {
    pub command: CommandKind,
    pub uuid: Uuid,
    pub kind: AnchorKind,
    pub parameters: Parameters,
    pub arguments: Arguments,
    pub range: Range,
}

pub enum Content {
    Text(Text),
    Tag(Tag),
    Anchor(Anchor),
}

pub struct Document {
    pub content: Vec<Content>,
    pub range: Range,
}

pub struct Parser<'a> {
    _document: &'a str,
    position: Position,
    iterator: Chars<'a>,
}

#[derive(Debug, Clone)]
pub struct ParserStatus<'a> {
    pub position: Position,
    pub iterator: Chars<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(document: &'a str) -> Self {
        dbg!("Parser::new", document);
        Self {
            _document: document,
            position: Position { offset: 0, line: 1, column: 1 },
            iterator: document.chars(),
        }
    }
    pub fn get_position(&self) -> Position {
        dbg!("Parser::get_position", self.position);
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
    pub fn is_eol(&self) -> bool {
        self.remain().starts_with("\n")
    }
    pub fn is_begin_of_line(&self) -> bool {
        self.position.column == 1
    }
    pub fn peek(&self) -> Option<char> {
        let next_char = self.iterator.clone().next();
        dbg!("Parser::peek", next_char);
        next_char
    }
    pub fn consume_matching_string(&mut self, xs: &str) -> bool {
        dbg!("Parser::consume_matching_string", xs, self.position);
        if !self.remain().starts_with(xs) {
            dbg!("Parser::consume_matching_string no match");
            return false;
        }
        for _x in xs.chars() {
            self.advance();
        }
        dbg!("Parser::consume_matching_string matched");
        true
    }    
    pub fn consume_matching_char(&mut self, x: char) -> bool {
        dbg!("Parser::consume_matching_char", x, self.position);
        match self.remain().chars().next() {
            None => {
                dbg!("Parser::consume_matching_char EOD");
                return false;
            }
            Some(y) => {
                if x != y {
                    dbg!("Parser::consume_matching_char no match", y);
                    return false;
                }
                self.advance();
                dbg!("Parser::consume_matching_char matched", y);
                return true;
            }
        }
    }
        pub fn consume_char_if<F>(&mut self, filter: F) -> Option<char> 
        where F: FnOnce(char) -> bool,
        {
            dbg!("Parser::consume_char_if", self.position);
            match self.remain().chars().next() {
                None => {
                    dbg!("Parser::consume_char_if EOD");
                    return None;
                }
                Some(y) => {
                    if !filter(y) {
                        dbg!("Parser::consume_char_if filter failed", y);
                        return None;
                    }
                    self.advance();
                    dbg!("Parser::consume_char_if consumed", y);
                    return Some(y);
                }
            }
        }    pub fn consume_one_char_of(&mut self, xs: &str) -> Option<char> {
        dbg!("Parser::consume_one_char_of", xs, self.position);
        for x in xs.chars() {
            if self.consume_matching_char(x) {
                dbg!("Parser::consume_one_char_of matched", x);
                return Some(x);
            }
        }
        dbg!("Parser::consume_one_char_of no match");
        None
    }
    pub fn skip_many_of(&mut self, xs: &str) {
        dbg!("Parser::skip_many_of", xs, self.position);
        while self.consume_one_char_of(xs).is_some() {}
        dbg!("Parser::skip_many_of finished", self.position);
    }
    pub fn skip_many_whitespaces(&mut self) {
        dbg!("Parser::skip_many_whitespaces", self.position);
        self.skip_many_of(" \t\r");
        dbg!("Parser::skip_many_whitespaces finished", self.position);
    }
    pub fn skip_many_whitespaces_or_eol(&mut self) {
        dbg!("Parser::skip_many_whitespaces_or_eol", self.position);
        self.skip_many_of(" \t\r\n");
        dbg!("Parser::skip_many_whitespaces_or_eol finished", self.position);
    }
    pub fn consume_one_dec_digit(&mut self) -> Option<char> {
        dbg!("Parser::consume_one_dec_digit", self.position);
        self.consume_char_if(|x| x.is_ascii_digit())
    }
    pub fn consume_one_hex_digit(&mut self) -> Option<char> {
        dbg!("Parser::consume_one_hex_digit", self.position);
        self.consume_char_if(|x| x.is_ascii_hexdigit())
    }
    pub fn consume_one_alpha(&mut self) -> Option<char> {
        dbg!("Parser::consume_one_alpha", self.position);
        self.consume_char_if(|x| x.is_ascii_alphabetic())
    }
    pub fn consume_one_alnum(&mut self) -> Option<char> {
        dbg!("Parser::consume_one_alnum", self.position);
        self.consume_char_if(|x| x.is_ascii_alphanumeric())
    }
    pub fn consume_one_alpha_or_underscore(&mut self) -> Option<char> {
        dbg!("Parser::consume_one_alpha_or_underscore", self.position);
        self.consume_char_if(|x| x.is_ascii_alphabetic() || x == '_')
    }
    pub fn consume(&mut self) -> Option<char> {
        dbg!("Parser::consume", self.position);
        let current_char = self.peek()?;
        self.advance();
        Some(current_char)
    }

    pub fn consume_one_alnum_or_underscore(&mut self) -> Option<char> {
        dbg!("Parser::consume_one_alnum_or_underscore", self.position);
        self.consume_char_if(|x| x.is_ascii_alphanumeric() || x == '_')
    }
    pub fn advance(&mut self) -> Option<char> {
        let next_char = match self.iterator.next() {
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
        };
        dbg!("Parser::advance", self.position, next_char);
        next_char
    }
    pub fn store(&self) -> ParserStatus<'a> {
        dbg!("Parser::store", self.position);
        ParserStatus {
            position: self.position.clone(),
            iterator: self.iterator.clone(),
        }
    }
    pub fn load(&mut self, status: ParserStatus<'a>) {
        dbg!("Parser::load", &status);
        self.position = status.position;
        self.iterator = status.iterator;
    }
}

pub fn parse_document(document: &str) -> Result<Document> {
    dbg!("parse_document", document);
    let mut parser = Parser::new(document);
    let begin = parser.get_position();
    let content = parse_content(document, &mut parser)?;
    let end   = parser.get_position();

    Ok(Document {
        content: content,
        range: Range { begin, end },
    })
}

fn parse_content(_document: &str, parser: &mut Parser) -> Result<Vec<Content>> {
    dbg!("parse_content", parser.get_position());
    let mut contents = Vec::new();

    while !parser.is_eod() {
        dbg!("parse_content loop", parser.get_position());
        parser.skip_many_whitespaces_or_eol(); // Skip leading whitespace/newlines between nodes
        if parser.is_eod() { dbg!("parse_content eod"); break; }

        let begin_pos = parser.get_position();
        if let Some(tag) = _try_parse_tag(parser)? {
            dbg!("parse_content parsed tag", &tag);
            contents.push(Content::Tag(tag));            
        } else if let Some(anchor) = _try_parse_anchor(parser)? {
            dbg!("parse_content parsed anchor", &anchor);
            contents.push(Content::Anchor(anchor));
        } else if let Some(text) = _try_parse_text(parser)? {
            dbg!("parse_content parsed text", &text);
            contents.push(Content::Text(text));
        } else {
            dbg!("parse_content unparseable content");
            return Err(ParsingError::InvalidSyntax {
                message: "Unparseable content".to_string(),
                range: Range { begin: begin_pos, end: parser.get_position() },
            }.into());
        }
    }

    dbg!("parse_content returning", &contents);
    Ok(contents)
}

fn _try_parse_tag(parser: &mut Parser) -> Result<Option<Tag>> {
    dbg!("_try_parse_tag", parser.get_position());
    let status = parser.store();

    if let Some(x) = _try_parse_tag0(parser)? {
        dbg!("_try_parse_tag success", &x);
        return Ok(Some(x));
    }

    dbg!("_try_parse_tag failed, loading status");
    parser.load(status.clone());
    Ok(None)
}
 
fn _try_parse_tag0(parser: &mut Parser) -> Result<Option<Tag>> {
    dbg!("_try_parse_tag0", parser.get_position());
    let begin = parser.get_position();

    if !parser.consume_matching_char('@') {
        dbg!("_try_parse_tag0 no @");
        return Ok(None);
    }

    let command = match _try_parse_command_kind(parser)? {
        Some(c) => { dbg!("_try_parse_tag0 command", &c); c },
        None => {
            dbg!("_try_parse_tag0 no command kind");
            return Err(ParsingError::InvalidSyntax {
                message: "Expected command kind after @".to_string(),
                range: Range { begin, end: parser.get_position() },
            }.into());
        },
    };

    parser.skip_many_whitespaces();
    dbg!("_try_parse_tag0 after command and whitespace", parser.get_position());

    let parameters = _try_parse_parameters(parser)?.unwrap_or_else(|| {
        dbg!("_try_parse_tag0 no parameters");
        Parameters { parameters: json!({}), range: Range { begin: parser.get_position(), end: parser.get_position() } }
    });
    dbg!("_try_parse_tag0 parameters", &parameters);
    
    parser.skip_many_whitespaces();
    dbg!("_try_parse_tag0 after parameters and whitespace", parser.get_position());

    let arguments = _try_parse_arguments(parser)?.unwrap_or_else(|| {
        dbg!("_try_parse_tag0 no arguments");
        Arguments { arguments: Vec::new(), range: Range { begin: parser.get_position(), end: parser.get_position() } }
    });
    dbg!("_try_parse_tag0 arguments", &arguments);

    parser.skip_many_whitespaces();
    dbg!("_try_parse_tag0 after arguments and whitespace", parser.get_position());

    if !parser.is_eol() && !parser.is_eod() {
        dbg!("_try_parse_tag0 unexpected content after tag");
        return Err(ParsingError::InvalidSyntax {
            message: "Unexpected content after tag arguments".to_string(),
            range: Range { begin: parser.get_position(), end: parser.get_position() },
        }.into());
    }
    dbg!("_try_parse_tag0 consuming newline");
    parser.consume_matching_char('\n'); // Consume newline if present

    let end = parser.get_position();
    dbg!("_try_parse_tag0 end", end);

    Ok(Some(Tag {
        command,
        parameters,
        arguments,
        range: Range {
            begin, end 
        }
    }))
}

fn _try_parse_anchor(parser: &mut Parser) -> Result<Option<Anchor>> {
    dbg!("_try_parse_anchor", parser.get_position());
    let status = parser.store();

    if let Some(x) = _try_parse_anchor0(parser)? {
        dbg!("_try_parse_anchor success", &x);
        return Ok(Some(x));
    }

    dbg!("_try_parse_anchor failed, loading status");
    parser.load(status.clone());
    Ok(None)
}

fn _try_parse_anchor0(parser: &mut Parser) -> Result<Option<Anchor>> {
    dbg!("_try_parse_anchor0", parser.get_position());
    let begin = parser.get_position();

    if !parser.consume_matching_string("<!--") {
        dbg!("_try_parse_anchor0 no <!--");
        return Ok(None);
    }

    parser.skip_many_whitespaces();
    dbg!("_try_parse_anchor0 after <!-- and whitespace", parser.get_position());

    let command = match _try_parse_command_kind(parser)? {
        Some(c) => { dbg!("_try_parse_anchor0 command", &c); c },
        None => {
            dbg!("_try_parse_anchor0 no command kind");
            return Err(ParsingError::InvalidSyntax {
                message: "Expected command kind after <!--".to_string(),
                range: Range { begin, end: parser.get_position() },
            }.into());
        },
    };

    if !parser.consume_matching_char('-') {
        dbg!("_try_parse_anchor0 no hyphen");
        return Err(ParsingError::UnexpectedToken {
            expected: "-".to_string(),
            found: parser.peek().map_or("EOF".to_string(), |c| c.to_string()),
            range: Range { begin: parser.get_position(), end: parser.get_position() },
        }.into());
    }

    let uuid = match _try_parse_uuid(parser)? {
        Some(u) => { dbg!("_try_parse_anchor0 uuid", &u); u },
        None => {
            dbg!("_try_parse_anchor0 no uuid");
            return Err(ParsingError::InvalidSyntax {
                message: "Expected UUID after command kind and -".to_string(),
                range: Range { begin, end: parser.get_position() },
            }.into());
        },
    };

    if !parser.consume_matching_char(':') {
        dbg!("_try_parse_anchor0 no colon");
        return Err(ParsingError::UnexpectedToken {
            expected: ":".to_string(),
            found: parser.peek().map_or("EOF".to_string(), |c| c.to_string()),
            range: Range { begin: parser.get_position(), end: parser.get_position() },
        }.into());
    }

    let kind = match _try_parse_anchor_kind(parser)? {
        Some(k) => { dbg!("_try_parse_anchor0 kind", &k); k },
        None => {
            dbg!("_try_parse_anchor0 no anchor kind");
            return Err(ParsingError::InvalidSyntax {
                message: "Expected anchor kind after UUID and :".to_string(),
                range: Range { begin, end: parser.get_position() },
            }.into());
        },
    };

    parser.skip_many_whitespaces();
    dbg!("_try_parse_anchor0 after kind and whitespace", parser.get_position());

    let parameters = _try_parse_parameters(parser)?.unwrap_or_else(|| {
        dbg!("_try_parse_anchor0 no parameters");
        Parameters { parameters: json!({}), range: Range { begin: parser.get_position(), end: parser.get_position() } }
    });
    dbg!("_try_parse_anchor0 parameters", &parameters);
    
    parser.skip_many_whitespaces();
    dbg!("_try_parse_anchor0 after parameters and whitespace", parser.get_position());

    let arguments = _try_parse_arguments(parser)?.unwrap_or_else(|| {
        dbg!("_try_parse_anchor0 no arguments");
        Arguments { arguments: Vec::new(), range: Range { begin: parser.get_position(), end: parser.get_position() } }
    });
    dbg!("_try_parse_anchor0 arguments", &arguments);

    parser.skip_many_whitespaces_or_eol();
    dbg!("_try_parse_anchor0 after arguments and whitespace/eol", parser.get_position());

    if !parser.consume_matching_string("-->") {
        dbg!("_try_parse_anchor0 no closing -->");
        return Err(ParsingError::UnterminatedString {
            range: Range { begin, end: parser.get_position() },
        }.into());
    }
    dbg!("_try_parse_anchor0 consumed -->");

    parser.skip_many_whitespaces();
    dbg!("_try_parse_anchor0 after closing --> and whitespace", parser.get_position());

    if !parser.is_eol() && !parser.is_eod() {
        dbg!("_try_parse_anchor0 unexpected content after anchor");
        return Err(ParsingError::InvalidSyntax {
            message: "Unexpected content after anchor closing tag".to_string(),
            range: Range { begin: parser.get_position(), end: parser.get_position() },
        }.into());
    }
    dbg!("_try_parse_anchor0 consuming newline");
    parser.consume_matching_char('\n'); // Consume newline if present

    let end = parser.get_position();
    dbg!("_try_parse_anchor0 end", end);

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
    dbg!("_try_parse_command_kind", parser.get_position());
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
        if parser.consume_matching_string(name) {
            dbg!("_try_parse_command_kind matched", name);
            return Ok(Some(kind));
        }
    }

    dbg!("_try_parse_command_kind no match");
    Ok(None)
}

fn _try_parse_uuid(parser: &mut Parser) -> Result<Option<Uuid>> {
    dbg!("_try_parse_uuid", parser.get_position());
    let start_pos = parser.get_position();
    let mut uuid_str = String::new();

    // UUID format: 8-4-4-4-12 hex digits
    for i in 0..8 {
        if let Some(c) = parser.consume_one_hex_digit() {
            uuid_str.push(c);
        } else {
            dbg!("_try_parse_uuid failed at first 8 hex digits");
            parser.load(ParserStatus { position: start_pos, iterator: parser.iterator.clone() });
            return Ok(None);
        }
    }
    if !parser.consume_matching_char('-') { dbg!("_try_parse_uuid no first hyphen"); return Ok(None); }
    uuid_str.push('-');
    for i in 0..4 {
        if let Some(c) = parser.consume_one_hex_digit() {
            uuid_str.push(c);
        } else {
            dbg!("_try_parse_uuid failed at first 4 hex digits");
            parser.load(ParserStatus { position: start_pos, iterator: parser.iterator.clone() });
            return Ok(None);
        }
    }
    if !parser.consume_matching_char('-') { dbg!("_try_parse_uuid no second hyphen"); return Ok(None); }
    uuid_str.push('-');
    for i in 0..4 {
        if let Some(c) = parser.consume_one_hex_digit() {
            uuid_str.push(c);
        } else {
            dbg!("_try_parse_uuid failed at second 4 hex digits");
            parser.load(ParserStatus { position: start_pos, iterator: parser.iterator.clone() });
            return Ok(None);
        }
    }
    if !parser.consume_matching_char('-') { dbg!("_try_parse_uuid no third hyphen"); return Ok(None); }
    uuid_str.push('-');
    for i in 0..4 {
        if let Some(c) = parser.consume_one_hex_digit() {
            uuid_str.push(c);
        } else {
            dbg!("_try_parse_uuid failed at third 4 hex digits");
            parser.load(ParserStatus { position: start_pos, iterator: parser.iterator.clone() });
            return Ok(None);
        }
    }
    if !parser.consume_matching_char('-') { dbg!("_try_parse_uuid no fourth hyphen"); return Ok(None); }
    uuid_str.push('-');
    for i in 0..12 {
        if let Some(c) = parser.consume_one_hex_digit() {
            uuid_str.push(c);
        } else {
            dbg!("_try_parse_uuid failed at last 12 hex digits");
            parser.load(ParserStatus { position: start_pos, iterator: parser.iterator.clone() });
            return Ok(None);
        }
    }

    dbg!("_try_parse_uuid parsed string", &uuid_str);
    match Uuid::parse_str(&uuid_str) {
        Ok(uuid) => { dbg!("_try_parse_uuid success", &uuid); Ok(Some(uuid)) },
        Err(e) => { dbg!("_try_parse_uuid parse error", &e); Err(ParsingError::InvalidSyntax {
            message: format!("Invalid UUID format: {}", uuid_str),
            range: Range { begin: start_pos, end: parser.get_position() },
        }.into())},
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

fn _try_parse_anchor_kind(parser: &mut Parser) -> Result<Option<AnchorKind>> {
    dbg!("_try_parse_anchor_kind", parser.get_position());
    let tags_list = vec![
        ("begin", AnchorKind::Begin),
        ("end", AnchorKind::End),
    ];

    for (name, kind) in tags_list {
        if parser.consume_matching_string(name) {
            dbg!("_try_parse_anchor_kind matched", name);
            return Ok(Some(kind));
        }
    }

    dbg!("_try_parse_anchor_kind no match");
    Ok(None)
}

fn _try_parse_parameters(parser: &mut Parser) -> Result<Option<Parameters>> {
    dbg!("_try_parse_parameters", parser.get_position());
    let status = parser.store();

    if let Some(x) = _try_parse_parameters0(parser)? {
        dbg!("_try_parse_parameters success", &x);
        return Ok(Some(x));
    }

    dbg!("_try_parse_parameters failed, loading status");
    parser.load(status);
    Ok(None)
}

fn _try_parse_parameters0(parser: &mut Parser) -> Result<Option<Parameters>> {
    dbg!("_try_parse_parameters0", parser.get_position());
    let begin = parser.get_position();

    if !parser.consume_matching_char('{') {
        dbg!("_try_parse_parameters0 no opening brace");
        return Ok(None);
    } 

    let mut parameters = json!({});

    loop {
        dbg!("_try_parse_parameters0 loop", parser.get_position());
        parser.skip_many_whitespaces_or_eol();

        if parser.consume_matching_char('}') {
            dbg!("_try_parse_parameters0 closing brace");
            break;
        }
        
        let parameter_begin = parser.get_position();
        let (key, value) = match _try_parse_parameter(parser)? {
            Some(p) => { dbg!("_try_parse_parameters0 parsed parameter", &p); p },
            None => {
                dbg!("_try_parse_parameters0 no parameter entry");
                return Err(ParsingError::InvalidSyntax {
                    message: "Expected parameter entry".to_string(),
                    range: Range { begin: parameter_begin, end: parser.get_position() },
                }.into());
            },
        };

        parameters[key] = value;

        parser.skip_many_whitespaces_or_eol();

        if parser.consume_matching_char(',') {
            dbg!("_try_parse_parameters0 consumed comma");
            // Continue loop for next parameter
        } else if parser.peek() == Some('}') {
            dbg!("_try_parse_parameters0 peeked closing brace");
            // Closing brace will be consumed in the next iteration
        } else {
            dbg!("_try_parse_parameters0 unexpected token");
            return Err(ParsingError::UnexpectedToken {
                expected: ", or } ".to_string(),
                found: parser.peek().map_or("EOF".to_string(), |c| c.to_string()),
                range: Range { begin: parser.get_position(), end: parser.get_position() },
            }.into());
        }
    }

    let end = parser.get_position();
    dbg!("_try_parse_parameters0 end", end);

    Ok(Some(Parameters { parameters, range: Range { begin, end } }))
}

fn _try_parse_parameter(parser: &mut Parser) -> Result<Option<(String, serde_json::Value)>> {
    dbg!("_try_parse_parameter", parser.get_position());
    let begin = parser.get_position();

    let key = match _try_parse_identifier(parser)? {
        Some(k) => { dbg!("_try_parse_parameter key", &k); k },
        None => {
            dbg!("_try_parse_parameter no key");
            return Err(ParsingError::InvalidSyntax {
                message: "Expected parameter key".to_string(),
                range: Range { begin, end: parser.get_position() },
            }.into());
        },
    };

    parser.skip_many_whitespaces_or_eol();
    dbg!("_try_parse_parameter after key and whitespace/eol", parser.get_position());

    if !parser.consume_matching_char(':') {
        dbg!("_try_parse_parameter no colon");
        return Err(ParsingError::UnexpectedToken {
            expected: ":".to_string(),
            found: parser.peek().map_or("EOF".to_string(), |c| c.to_string()),
            range: Range { begin: parser.get_position(), end: parser.get_position() },
        }.into());
    }
    dbg!("_try_parse_parameter consumed colon");

    parser.skip_many_whitespaces_or_eol();
    dbg!("_try_parse_parameter after colon and whitespace/eol", parser.get_position());

    let value = match _try_parse_value(parser)? {
        Some(v) => { dbg!("_try_parse_parameter value", &v); v },
        None => {
            dbg!("_try_parse_parameter no value");
            return Err(ParsingError::InvalidSyntax {
                message: "Expected parameter value".to_string(),
                range: Range { begin, end: parser.get_position() },
            }.into());
        },
    };

    Ok(Some((key, value)))
}

fn _try_parse_identifier(parser: &mut Parser) -> Result<Option<String>> {
    dbg!("_try_parse_identifier", parser.get_position());
    let mut identifier = String::new();

    match parser.consume_one_alpha_or_underscore() {
        None => {
            dbg!("_try_parse_identifier no initial alpha/underscore");
            return Ok(None);
        }
        Some(x) => {
            identifier.push(x);
        }
    }
    
    loop {
        match parser.consume_one_alnum_or_underscore() {
            None => {
                dbg!("_try_parse_identifier end of alnum/underscore");
                break;
            }
            Some(x) => {
                identifier.push(x);
            }
        }
    }

    dbg!("_try_parse_identifier result", &identifier);
    Ok(Some(identifier))
}

fn _try_parse_value(parser: &mut Parser) -> Result<Option<serde_json::Value>> {
    dbg!("_try_parse_value", parser.get_position());
    if parser.peek() == Some('"') { // Check peek instead of consume to get the char
        parser.advance(); // Consume the opening quote
        dbg!("_try_parse_value parsing double quoted string");
        Ok(_try_parse_enclosed_value(parser, '"')?.map(|s| serde_json::Value::String(s)))
    } else if parser.peek() == Some('\'') { // Check peek instead of consume to get the char
        parser.advance(); // Consume the opening quote
        dbg!("_try_parse_value parsing single quoted string");
        Ok(_try_parse_enclosed_value(parser, '\'')?.map(|s| serde_json::Value::String(s)))
    } else {
        dbg!("_try_parse_value parsing nude value");
        _try_parse_nude_value(parser)
    }
}

fn _try_parse_enclosed_value(parser: &mut Parser, closure: char) -> Result<Option<String>> {
    dbg!("_try_parse_enclosed_value", parser.get_position(), closure);
    let mut value = String::new();
    let start_pos = parser.get_position();

    loop {
        dbg!("_try_parse_enclosed_value loop", parser.get_position());
        if parser.consume_matching_char(closure) {
            dbg!("_try_parse_enclosed_value consumed closure", &value);
            return Ok(Some(value));
        } else if parser.consume_matching_char('\\') {
            dbg!("_try_parse_enclosed_value consumed backslash");
            // Handle escape sequences
            if parser.consume_matching_char('n') {
                value.push('\n');
            } else if parser.consume_matching_char('r') {
                value.push('\r');
            } else if parser.consume_matching_char('t') {
                value.push('\t');
            } else if parser.consume_matching_char('\\') {
                value.push('\\');
            } else if parser.consume_matching_char('"') {
                value.push('"');
            } else if parser.consume_matching_char('\'') {
                value.push('\'');
            } else if parser.consume_matching_char(closure) { // Handle escaped closure character
                value.push(closure);
            }
            else {
                dbg!("_try_parse_enclosed_value invalid escape sequence");
                // Invalid escape sequence, just push the backslash and the next char
                value.push('\\');
                if let Some(c) = parser.advance() {
                    value.push(c);
                } else {
                    return Err(ParsingError::UnterminatedString {
                        range: Range { begin: start_pos, end: parser.get_position() },
                    }.into());
                }
            }
        } else {
            match parser.advance() {
                None => {
                    dbg!("_try_parse_enclosed_value EOD, unterminated string");
                    return Err(ParsingError::UnterminatedString {
                        range: Range { begin: start_pos, end: parser.get_position() },
                    }.into());
                }
                Some(x) => {
                    value.push(x);
                }
            }
        }
    }
}

fn _try_parse_nude_value(parser: &mut Parser) -> Result<Option<serde_json::Value>> {
    dbg!("_try_parse_nude_value", parser.get_position());
    let status = parser.store();

    if let Ok(Some(x)) = _try_parse_nude_integer(parser) {
        dbg!("_try_parse_nude_value parsed integer", x);
        return Ok(Some(json!(x)));
    }
    parser.load(status.clone());

    if let Ok(Some(x)) = _try_parse_nude_float(parser) {
        dbg!("_try_parse_nude_value parsed float", x);
        return Ok(Some(json!(x)));
    }
    parser.load(status.clone());

    if let Ok(Some(x)) = _try_parse_nude_bool(parser) {
        dbg!("_try_parse_nude_value parsed bool", x);
        return Ok(Some(json!(x)));
    }
    parser.load(status.clone());

    if let Ok(Some(x)) = _try_parse_nude_string(parser) {
        dbg!("_try_parse_nude_value parsed string", x);
        return Ok(Some(json!(x)));
    }
    parser.load(status.clone());

    dbg!("_try_parse_nude_value no match");
    Ok(None)
}

fn _try_parse_nude_integer(parser: &mut Parser) -> Result<Option<i64>> {
    dbg!("_try_parse_nude_integer", parser.get_position());
    let mut number = String::new();
    let start_pos = parser.get_position();

    loop {
        match parser.consume_one_dec_digit() {
            Some(x) => {
                number.push(x);
            }
            None => {
                break;
            }
        }
    }

    if number.is_empty() {
        dbg!("_try_parse_nude_integer empty");
        Ok(None)
    } else {
        dbg!("_try_parse_nude_integer parsing", &number);
        match i64::from_str_radix(&number, 10) {
            Ok(num) => { dbg!("_try_parse_nude_integer success", num); Ok(Some(num)) },
            Err(_) => { dbg!("_try_parse_nude_integer error"); Err(ParsingError::InvalidNumberFormat {
                value: number,
                range: Range { begin: start_pos, end: parser.get_position() },
            }.into())},
        }
    }
}

fn _try_parse_nude_float(parser: &mut Parser) -> Result<Option<f64>> {
    dbg!("_try_parse_nude_float", parser.get_position());
    let start_pos = parser.get_position();
    let mut number = String::new();
    let mut has_decimal = false;

    // Handle leading '.' for floats like ".5"
    if parser.peek() == Some('.') {
        if let Some(c) = parser.consume() {
            number.push(c);
            has_decimal = true;
        }
    }

    // Consume leading digits
    while let Some(x) = parser.consume_one_dec_digit() {
        number.push(x);
    }

    // Consume optional decimal point and subsequent digits
    if !has_decimal && parser.consume_matching_char('.') {
        has_decimal = true;
        number.push('.');
        while let Some(x) = parser.consume_one_dec_digit() {
            number.push(x);
        }
    }

    if number.is_empty() || (number == "." && has_decimal) {
        dbg!("_try_parse_nude_float empty or just dot");
        parser.load(ParserStatus { position: start_pos, iterator: parser.iterator.clone() }); // Rewind if nothing was parsed
        return Ok(None);
    }

    if has_decimal {
        dbg!("_try_parse_nude_float parsing float", &number);
        match number.parse::<f64>() {
            Ok(f) => { dbg!("_try_parse_nude_float success", f); Ok(Some(f)) },
            Err(_) => { dbg!("_try_parse_nude_float error"); Err(ParsingError::InvalidNumberFormat {
                value: number,
                range: Range { begin: start_pos, end: parser.get_position() },
            }.into())},
        }
    } else {
        dbg!("_try_parse_nude_float not a float (no decimal)");
        parser.load(ParserStatus { position: start_pos, iterator: parser.iterator.clone() }); // Rewind if it was just an integer
        Ok(None) // Not a float if no decimal was found
    }
}

fn _try_parse_nude_bool(parser: &mut Parser) -> Result<Option<bool>> {
    dbg!("_try_parse_nude_bool", parser.get_position());
    if parser.consume_matching_string("true") {
        dbg!("_try_parse_nude_bool parsed true");
        return Ok(Some(true));
    } else if parser.consume_matching_string("false") {
        dbg!("_try_parse_nude_bool parsed false");
        return Ok(Some(false));
    } else {
        dbg!("_try_parse_nude_bool no match");
        return Ok(None);
    }
}

fn _try_parse_nude_string(parser: &mut Parser) -> Result<Option<String>> {
    dbg!("_try_parse_nude_string", parser.get_position());
    let mut s = String::new();
    let start_pos = parser.get_position();

    loop {
        let current_char = parser.peek();
        match current_char {
            Some(c) if c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '/' || c == '-' => {
                parser.advance();
                s.push(c);
            },
            _ => break,
        }
    }

    if s.is_empty() {
        dbg!("_try_parse_nude_string empty");
        Ok(None)
    } else {
        dbg!("_try_parse_nude_string result", &s);
        Ok(Some(s))
    }
}

fn _try_parse_argument(parser: &mut Parser) -> Result<Option<Argument>> {
    dbg!("_try_parse_argument", parser.get_position());
    let begin = parser.get_position();
    let status = parser.store();

    let value_json_result = if parser.peek() == Some('"') {
        parser.advance(); // Consume the opening quote
        dbg!("_try_parse_argument parsing double quoted string");
        Ok(_try_parse_enclosed_value(parser, '"')?.map(|s| serde_json::Value::String(s)))
    } else if parser.peek() == Some('\'') {
        parser.advance(); // Consume the opening quote
        dbg!("_try_parse_argument parsing single quoted string");
        Ok(_try_parse_enclosed_value(parser, '\'')?.map(|s| serde_json::Value::String(s)))
    } else {
        dbg!("_try_parse_argument parsing nude value");
        _try_parse_nude_value(parser)
    };

    let value_json = match value_json_result {
        Ok(Some(v)) => Some(v),
        Ok(None) => {
            dbg!("_try_parse_argument no value parsed, loading status");
            parser.load(status);
            return Ok(None);
        },
        Err(e) => { dbg!("_try_parse_argument error", &e); return Err(e); },
    };

    if let Some(json_value) = value_json {
        let end = parser.get_position();
        // Convert serde_json::Value back to String for Argument.value
        let value_str = match json_value {
            serde_json::Value::String(s) => s,
            _ => json_value.to_string(), // Convert other types to string representation
        };
        dbg!("_try_parse_argument success", &value_str);
        Ok(Some(Argument {
            value: value_str,
            range: Range { begin, end },
        }))
    } else {
        dbg!("_try_parse_argument final no value, loading status");
        parser.load(status);
        Ok(None)
    }
}

fn _try_parse_arguments(parser: &mut Parser) -> Result<Option<Arguments>> {
    dbg!("_try_parse_arguments", parser.get_position());
    let begin = parser.get_position();
    let mut args = Vec::new();

    loop {
        dbg!("_try_parse_arguments loop", parser.get_position());
        parser.skip_many_whitespaces();
        if parser.is_eol() || parser.is_eod() {
            dbg!("_try_parse_arguments EOL or EOD");
            break;
        }

        let status = parser.store();
        if let Some(arg) = _try_parse_argument(parser)? {
            dbg!("_try_parse_arguments parsed arg", &arg);
            args.push(arg);
        } else {
            dbg!("_try_parse_arguments no arg parsed, loading status");
            parser.load(status);
            break;
        }
    }

    if args.is_empty() {
        dbg!("_try_parse_arguments empty");
        Ok(None)
    } else {
        let end = parser.get_position();
        dbg!("_try_parse_arguments result", &args);
        Ok(Some(Arguments {
            arguments: args,
            range: Range { begin, end },
        }))
    }
}

fn _try_parse_text(parser: &mut Parser) -> Result<Option<Text>> {
    dbg!("_try_parse_text", parser.get_position());
    let begin = parser.get_position();
    let mut content_len = 0;

    loop {
        dbg!("_try_parse_text loop", parser.get_position());
        let current_status = parser.store();
        if parser.is_eod() || parser.remain().starts_with(" @") || parser.remain().starts_with("<!--") {
            dbg!("_try_parse_text EOD or start of tag/anchor");
            parser.load(current_status);
            break;
        }

        match parser.advance() {
            None => {
                dbg!("_try_parse_text EOD during advance");
                break;
            }
            Some(_) => {
                content_len += 1;
            }
        }
    }

    if content_len == 0 {
        dbg!("_try_parse_text empty");
        Ok(None)
    } else {
        let end = parser.get_position();
        dbg!("_try_parse_text result", Range {begin, end });
        Ok(Some(Text {
            range: Range {begin, end }
        }))
    }
}
