
struct Position {
    offset: usize,      /// 0-based character offset
    line: usize,        /// 1-based line
    column: usize,      /// 1-based column
}

struct Range {
    begin: Position,
    end: Position,
}

struct Text {
    range: Range,
}

enum CommandKind {
    Tag,        // for debug purpose
    Include,
    Inline,
    Answer,
    Summarize,
    Derive,
    Repeat,
}

struct Parameters {
    parameters: serde_json::Value,
    range: Range,
}

struct Argument {
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

enum AnchorKind 
{
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
    pub fn consume_char_if<F>(&mut self, filter: F) -> bool 
    where F: FnOnce() -> bool,
    {
        match self.remain().chars().next() {
            None => {
                return false;
            }
            Some(y) => {
                if !F(y) {
                    return false;
                }
                self.advance();
                return true;
            }
        }
    }
    pub fn consume_one_char_of(&mut self, xs: &str) -> Option<char> {
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
    pub fn consume_one_dec_digit(&muf self) -> Option<char> {
        self.consume_char_if(|x| x.is_digit(10)); 
    }
    pub fn consume_one_hex_digit(&muf self) -> Option<char> {
        self.consume_char_if(|x| x.is_digit(16)); 
    }
    pub fn consume_one_alpha(&muf self) -> Option<char> {
        self.consume_one_char_of(|x| x.is_alphabetic()); 
    }
    pub fn consume_one_alnum(&muf self) -> Option<char> {
        self.consume_one_char_of(|x| x.is_alphanumeric()); 
    }
    pub fn consume_one_alpha_or_underscore(&muf self) -> Option<char> {
        self.consume_one_char_of(|x| x.is_alphabetic() | (x == '_')); 
    }
    pub fn consume_one_alnum_or_underscore(&muf self) -> Option<char> {
        self.consume_one_char_of(|x| x.is_alphanumeric() | (x == '_')); 
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

    while !parser.is_eod() {

        parser.skip_many_whitespaces_or_eol();

        if parser.consume_matching_char("}") {
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

fn _try_parse_parameter(parser: &mut Parser) -> Result<Option<(String, serde_json::Value)>> {

    let begin = parser.get_position();

    let key = _try_parse_identifier(parser)?;
    if key.is_none() {
        // TODO errore parsing key
    }    

    parser.skip_many_whitespaces_or_eol();

    if !parser.consume_matching_char(":") {
        // TODO errore parsing :
    } 

    parser.skip_many_whitespaces_or_eol();

    let value = _try_parse_value(parser)?;
    if value.is_none() {
        // TODO errore parsing value
    }

    let end = parser.get_position();

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

