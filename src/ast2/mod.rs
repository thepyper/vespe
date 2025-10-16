use std::str::Chars;
use uuid::Uuid;
use serde_json::{json, Value};
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
    pub column: usize,      /// 1-based column
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
    document: &'a str,
    position: Position,
    iterator: Chars<'a>,
}

pub struct ParserStatus<'a> {
    position: Position,
    iterator: Chars<'a>,
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
    pub fn is_eol(&self) -> bool {
        self.remain().starts_with("\n")
    }
    pub fn is_begin_of_line(&self) -> bool {
        self.position.column == 1
    }
    pub fn consume_matching_string(&mut self, xs: &str) -> bool {
        if !self.remain().starts_with(xs) {
            return false;
        }
        for x in xs.chars() {
            self.advance();
        }
        true
    }    
    pub fn consume_matching_char(&mut self, x: char) -> bool {
        match self.remain().chars().next() {
            None => {
                return false;
            }
            Some(y) => {
                if x != y {
                    return false;
                }
                self.advance();
                return true;
            }
        }
    }
        pub fn consume_char_if<F>(&mut self, filter: F) -> Option<char>
        where F: FnOnce(char) -> bool,
        {
            match self.remain().chars().next() {
                None => {
                    return None;
                }
                Some(y) => {
                    if !filter(y) {
                        return None;
                    }
                    self.advance();
                    return Some(y);
                }
            }
        }    pub fn consume_one_char_of(&mut self, xs: &str) -> Option<char> {
        for x in xs.chars() {
            if self.consume_matching_char(x) {
                return Some(x);
            }
        }
        None
    }
    pub fn skip_many_of(&mut self, xs: &str) {
        while self.consume_one_char_of(xs) {}
    }
    pub fn skip_many_whitespaces(&mut self) {
        self.skip_many_of(" \t\r");
    }
    pub fn skip_many_whitespaces_or_eol(&mut self) {
        self.skip_many_of(" \t\r\n");
    }
    pub fn consume_one_dec_digit(&mut self) -> Option<char> {
        self.consume_char_if(|x| x.is_ascii_digit())
    }
    pub fn consume_one_hex_digit(&mut self) -> Option<char> {
        self.consume_char_if(|x| x.is_ascii_hexdigit())
    }
    pub fn consume_one_alpha(&mut self) -> Option<char> {
        self.consume_char_if(|x| x.is_ascii_alphabetic())
    }
    pub fn consume_one_alnum(&mut self) -> Option<char> {
        self.consume_char_if(|x| x.is_ascii_alphanumeric())
    }
    pub fn consume_one_alpha_or_underscore(&mut self) -> Option<char> {
        self.consume_char_if(|x| x.is_ascii_alphabetic() || x == '_')
    }
    pub fn consume_one_alnum_or_underscore(&mut self) -> Option<char> {
        self.consume_char_if(|x| x.is_ascii_alphanumeric() || x == '_')
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

fn parse_document(document: &str) -> Result<Document> {

    let mut parser = Parser::new(document);
    let begin = parser.get_position();
    let content = parse_content(document, &mut parser)?;
    let end   = parser.get_position();

    Document {
        content: content,
        range: Range { begin, end },
    }
}

fn parse_content(document: &str, parser: &mut Parser) -> Result<Vec<Content>> {

    let mut contents = Vec::new();

    while !parser.is_eod() {
        if let Some(tag) = _try_parse_tag(document, parser)? {
            contents.push(Tag(tag));            
        } else if let Some(anchor) = _try_parse_anchor(document, parser)? {
            contents.push(Anchor(anchor));
        } else if let Some(text) = _try_parse_text(document, parser)? {
            contents.push(Text(text));
        } else {
            // TODO parse error
        }
    }

    Ok(contents)
}

fn _try_parse_tag(document: &str, parser: &mut Parser) -> Result<Option<Tag>> {

    let status = parser.store();

    if let Some(x) = _try_parse_tag0(document, parser)? {
        return Some(x);
    }

    parser.load(status);
    None
} 

fn _try_parse_tag0(document: &str, parser: &mut Parser) -> Result<Option<Tag>> {

    let begin = parser.get_position();

    if !parser.consume_matching_char('@') {
        return Ok(None);
    }

    let command = _try_parse_command_kind(document, parser)?;
    if command.is_none() {
        return Ok(None);
    }

    parser.skip_many_whitespaces();

    let parameters = _try_parse_parameters(document, parser)?;
    
    parser.skip_many_whitespaces();

    let arguments = _try_parse_arguments(document, parser)?;

    parser.skip_many_whitespaces();

    if !parser.consume_matching_char('\n') {
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

fn _try_parse_anchor(document: &str, parser: &mut Parser) -> Result<Option<Anchor>> {

    let status = parser.store();

    if let Some(x) = _try_parse_anchor0(document, parser)? {
        return Some(x);
    }

    parser.load(status);
    None
}

fn _try_parse_anchor0(document: &str, parser: &mut Parser) -> Result<Option<Anchor>> {

    let begin = parser.get_position();

    if !parser.consume_matching_string("<!--") {
        return Ok(None);
    }

    parser.skip_many_whitespaces();

    let command = _try_parse_command_kind(document, parser)?;
    if command.is_none() {
        return Ok(None);
    }

    if !parser.consume_matching_char('-') {
        // TODO parsing error anchor, manca trattino prima di uuid
    }

    let uuid = _try_parse_uuid(document, parser)?;
    if uuid.is_none() {
        // TODO parsing error anchor, manca uuid
    }

    if !parser.consume_matching_char(':') {
        // TODO parsing error anchor, manca :
    }

    let kind = _try_parse_anchor_kind(document, parser)?;

    parser.skip_many_whitespaces();

    let parameters = _try_parse_parameters(document, parser)?;
    
    parser.skip_many_whitespaces();

    let arguments = _try_parse_arguments(document, parser)?;

    parser.skip_many_whitespaces_or_eol();

    if !parser.consume_matching_string("-->") {
        // TODO errore, ancora non chiusa
    }

    parser.skip_many_whitespaces();

    if !parser.consume_matching_char('\n') {
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

fn _try_parse_command_kind(document: &str, parser: &mut Parser) -> Result<Option<CommandKind>> {

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
            return Some(kind);
        }
    }

    None
}

fn _try_parse_anchor_kind(document: &str, parser: &mut Parser) -> Result<Option<AnchorKind>> {

    let tags_list = vec![
        ("begin", AnchorKind::Begin),
        ("end", AnchorKind::End),
    ];

    for (name, kind) in tags_list {
        if parser.consume_matching_string(name) {
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

    if !parser.consume_matching_char("{") {
        return Ok(None);
    } 

    let mut parameters = json!({});

    loop {

        parser.skip_many_whitespaces_or_eol();

        if parser.consume_matching_char("}") {
            break;
        }
        
        let parameter_begin = parser.get_position();
        let (key, value) = match _try_parse_parameter(parser)? {
            Some(p) => p,
            None => return Err(ParsingError::InvalidSyntax {
                message: "Expected parameter entry".to_string(),
                range: Range { begin: parameter_begin, end: parser.get_position() },
            }.into()),
        };

        parameters[key] = value;

        parser.skip_many_whitespaces_or_eol();

        if parser.consume_matching_char(",") {
            // Continue loop for next parameter
        } else if parser.peek() == Some('}') {
            // Closing brace will be consumed in the next iteration
        } else {
            return Err(ParsingError::UnexpectedToken {
                expected: ", or } ".to_string(),
                found: parser.peek().map_or("EOF".to_string(), |c| c.to_string()),
                range: Range { begin: parser.get_position(), end: parser.get_position() },
            }.into());
        }
    }

    let end = parser.get_position();

    Ok(Some(Parameters { parameters, range: Range { begin, end } }))
}

fn _try_parse_parameter(parser: &mut Parser) -> Result<Option<(String, serde_json::Value)>> {

    let begin = parser.get_position();

    let key = match _try_parse_identifier(parser)? {
        Some(k) => k,
        None => return Err(ParsingError::InvalidSyntax {
            message: "Expected parameter key".to_string(),
            range: Range { begin, end: parser.get_position() },
        }.into()),
    };

    parser.skip_many_whitespaces_or_eol();

    if !parser.consume_matching_char(":") {
        return Err(ParsingError::UnexpectedToken {
            expected: ":".to_string(),
            found: parser.peek().map_or("EOF".to_string(), |c| c.to_string()),
            range: Range { begin, end: parser.get_position() },
        }.into());
    }

    parser.skip_many_whitespaces_or_eol();

    let value = match _try_parse_value(parser)? {
        Some(v) => v,
        None => return Err(ParsingError::InvalidSyntax {
            message: "Expected parameter value".to_string(),
            range: Range { begin, end: parser.get_position() },
        }.into()),
    };

    Ok(Some((key, value)))
}

fn _try_parse_identifier(parser: &mut Parser) -> Result<Option<String>> {

    let mut identifier = String::new();

    match parser.consume_one_alpha_or_underscore() {
        None => {
            return None;
        }
        Some(x) => {
            identifier.push(x);
        }
    }
    
    loop {
        match parser.consume_one_alnum_or_underscore() {
            None => {
                break;
            }
            Some(x) => {
                identifier.push(x);
            }
        }
    }

    Ok(Some(identifier))
}

fn _try_parse_value(parser: &mut Parser) -> Result<Option<serde_json::Value>> {
    if parser.consume_matching_char('"') {
        _try_parse_enclosed_value(parser, "\"")
    } else if parser.consume_matching_char('\'') {
        _try_parse_enclosed_value(parser, "\'")
    } else {
        _try_parse_nude_value(parser)
    }
}

fn _try_parse_enclosed_value(parser: &mut Parser, closure: &str) -> Result<Option<serde_json::Value>> {

    let mut value = String::new();
    let start_pos = parser.get_position();

    loop {
        if parser.consume_matching_string(closure) {
            return Ok(Some(json!(value)));
        } else if parser.consume_matching_string("\\") {
            // Handle escape sequences
            if parser.consume_matching_char('n') {
                value.push('\n');
            } else if parser.consume_matching_char('r') {
                value.push('\r');
            } else if parser.consume_matching_char('t') {
                value.push('\t');
            } else if parser.consume_matching_char('t') {
                value.push('\t');
            } else if parser.consume_matching_char('\\') {
                value.push('\\');
            } else if parser.consume_matching_char('"') {
                value.push('"');
            } else if parser.consume_matching_char('\'') {
                value.push('\'');
            } else {
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
    let status = parser.store();

    if let Ok(Some(x)) = _try_parse_nude_integer(parser) {
        return Ok(Some(json!(x)));
    }
    parser.load(&status);

    if let Ok(Some(x)) = _try_parse_nude_float(parser) {
        return Ok(Some(json!(x)));
    }
    parser.load(&status);

    if let Ok(Some(x)) = _try_parse_nude_bool(parser) {
        return Ok(Some(json!(x)));
    }
    parser.load(&status);

    if let Ok(Some(x)) = _try_parse_nude_string(parser) {
        return Ok(Some(json!(x)));
    }
    parser.load(&status);

    Ok(None)
}

fn _try_parse_nude_integer(parser: &mut Parser) -> Result<Option<i64>> {

    let mut number = String::new();

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
        return Ok(None);
    } else {
        return Ok(Some(i64::from_str_radix(&number, 10)));
    }
}

fn _try_parse_nude_float(parser: &mut Parser) -> Result<Option<f64>> {
    let start_pos = parser.get_position();
    let mut number = String::new();
    let mut has_decimal = false;

    // Handle leading '.' for floats like ".5"
    if parser.peek() == Some('.') {
        if let Some(c) = parser.consume_matching_char('.') {
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
        parser.load(&ParserStatus { position: start_pos, iterator: parser.iterator.clone() }); // Rewind if nothing was parsed
        return Ok(None);
    }

    if has_decimal {
        match number.parse::<f64>() {
            Ok(f) => Ok(Some(f)),
            Err(_) => Err(ParsingError::InvalidNumberFormat {
                value: number,
                range: Range { begin: start_pos, end: parser.get_position() },
            }.into()),
        }
    } else {
        parser.load(&ParserStatus { position: start_pos, iterator: parser.iterator.clone() }); // Rewind if it was just an integer
        Ok(None) // Not a float if no decimal was found
    }
}

fn _try_parse_nude_bool(parser: &mut Parser) -> Result<Option<bool>> {

    if parser.consume_matching_string("true") {
        return Ok(Some(true));
    } else if parser.consume_matching_string("false") {
        return Ok(Some(false));
    } else {
        return Ok(None);
    }
}

fn _try_parse_nude_string(parser: &mut Parser) -> Result<Option<String>> {
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
        Ok(None)
    } else {
        Ok(Some(s))
    }
}

fn _try_parse_argument(parser: &mut Parser) -> Result<Option<Argument>> {
    let begin = parser.get_position();
    let status = parser.store();

    let value_json_result = if parser.peek() == Some('"') {
        parser.advance(); // Consume the opening quote
        _try_parse_enclosed_value(parser, "\"")
    } else if parser.peek() == Some('\'') {
        parser.advance(); // Consume the opening quote
        _try_parse_enclosed_value(parser, "\'")
    } else {
        _try_parse_nude_value(parser)
    };

    let value_json = match value_json_result {
        Ok(Some(v)) => Some(v),
        Ok(None) => {
            parser.load(&status);
            return Ok(None);
        },
        Err(e) => return Err(e),
    };

    if let Some(json_value) = value_json {
        let end = parser.get_position();
        // Convert serde_json::Value back to String for Argument.value
        let value_str = match json_value {
            serde_json::Value::String(s) => s,
            _ => json_value.to_string(), // Convert other types to string representation
        };
        Ok(Some(Argument {
            value: value_str,
            range: Range { begin, end },
        }))
    } else {
        parser.load(&status);
        Ok(None)
    }
}

fn _try_parse_arguments(parser: &mut Parser) -> Result<Option<Arguments>> {
    let begin = parser.get_position();
    let mut args = Vec::new();

    loop {
        parser.skip_many_whitespaces();
        if parser.is_eol() || parser.is_eod() {
            break;
        }

        let status = parser.store();
        if let Some(arg) = _try_parse_argument(parser)? {
            args.push(arg);
        } else {
            parser.load(&status);
            break;
        }
    }

    if args.is_empty() {
        Ok(None)
    } else {
        let end = parser.get_position();
        Ok(Some(Arguments {
            arguments: args,
            range: Range { begin, end },
        }))
    }
}

fn _try_parse_text(parser: &mut Parser) -> Result<Option<Text>> {

    let begin = parser.get_position();

    let mut content = String::new();

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
        range: Range {begin, end }
    }))
}