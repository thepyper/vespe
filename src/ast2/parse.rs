use serde_json::json;
use std::collections::HashMap;
use std::str::Chars;
use std::str::FromStr;
use uuid::Uuid;

use super::{
    Anchor, AnchorKind, Argument, Arguments, Ast2Error, CommandKind, Content, Document,
    JsonPlusEntity, JsonPlusObject, Parameters, Position, Range, Result, Tag, Text,
};

/// A stateful parser that consumes a string slice and tracks the current position.
///
/// The parser is designed to be mostly immutable, where parsing functions take a parser
/// and return a new, advanced parser state. This functional approach makes the parsing
/// logic easier to reason about and test.
#[derive(Debug, Clone)]
pub(crate) struct Parser<'a> {
    position: Position,
    iterator: Chars<'a>,
}

impl<'a> Parser<'a> {
    /// Creates a new `Parser` for the given document string.
    pub fn new(document: &'a str) -> Self {
        Self {
            position: Position {
                offset: 0,
                line: 1,
                column: 1,
            },
            iterator: document.chars(),
        }
    }

    pub fn advance_immutable(&self) -> Option<(char, Parser<'a>)> {
        let mut new_parser = self.clone();
        if let Some(char) = new_parser.advance() {
            Some((char, new_parser))
        } else {
            None
        }
    }

    pub fn consume_char_if_immutable<F>(&self, filter: F) -> Option<(char, Parser<'a>)>
    where
        F: FnOnce(char) -> bool,
    {
        let mut new_parser = self.clone();
        match new_parser.consume_char_if(filter) {
            Some(c) => Some((c, new_parser)),
            None => None,
        }
    }

    pub fn consume_matching_char_immutable(&self, x: char) -> Option<Parser<'a>> {
        let mut new_parser = self.clone();
        match new_parser.consume_matching_char(x) {
            Some(_) => Some(new_parser),
            None => None,
        }
    }

    pub fn consume_matching_string_immutable(&self, xs: &str) -> Option<Parser<'a>> {
        let mut new_parser = self.clone();
        match new_parser.consume_matching_string(xs) {
            Some(_) => Some(new_parser),
            None => None,
        }
    }

    pub fn consume_many_if_immutable<F>(&self, filter: F) -> (String, Parser<'a>)
    where
        F: Fn(char) -> bool,
    {
        let mut new_parser = self.clone();
        let result = new_parser.consume_many_if(filter).unwrap_or_default();
        (result, new_parser)
    }

    pub fn skip_many_whitespaces_immutable(&self) -> Parser<'a> {
        let mut new_parser = self.clone();
        new_parser.skip_many_whitespaces();
        new_parser
    }

    pub fn skip_many_whitespaces_or_eol_immutable(&self) -> Parser<'a> {
        let mut new_parser = self.clone();
        new_parser.skip_many_whitespaces_or_eol();
        new_parser
    }

    pub fn get_position(&self) -> Position {
        self.position.clone()
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
    pub fn consume_matching_string(&mut self, xs: &str) -> Option<String> {
        if !self.remain().starts_with(xs) {
            None
        } else {
            for _x in xs.chars() {
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

/// Parses a string slice into a `Document` AST.
///
/// This is the main entry point for the parser.
///
/// # Arguments
///
/// * `document` - A string slice representing the source document to be parsed.
///
/// # Returns
///
/// A `Result` containing the parsed `Document` or an `Ast2Error` on failure.
pub fn parse_document(document: &str) -> Result<Document> {
    let parser = Parser::new(document);
    let begin = parser.get_position();
    let (content, parser_after_content) = parse_content(parser)?;
    let end = parser_after_content.get_position();

    Ok(Document {
        content: content,
        range: Range { begin, end },
    })
}

pub(crate) fn parse_content<'doc>(parser: Parser<'doc>) -> Result<(Vec<Content>, Parser<'doc>)> {
    let mut contents = Vec::new();
    let mut p_current = parser; // Takes ownership

    // The core parsing loop. It processes the document line by line.
    // Each line must start at column 1 and is attempted to be parsed as a Tag,
    // an Anchor, or plain Text, in that order of precedence.
    // If none of these match, and the line is not empty, it's a parsing error.
    while !p_current.is_eod() {
        // Subroutines must always stop at end-of-line, otherwise we have a problem
        if !p_current.is_begin_of_line() {
            return Err(Ast2Error::ExpectedBeginOfLine {
                position: p_current.get_position(),
            });
        }

        if let Some((tag, p_next)) = _try_parse_tag(&p_current)? {
            contents.push(Content::Tag(tag));
            p_current = p_next;
            continue;
        }

        if let Some((anchor, p_next)) = _try_parse_anchor(&p_current)? {
            contents.push(Content::Anchor(anchor));
            p_current = p_next;
            continue;
        }

        if let Some((text, p_next)) = _try_parse_text(&p_current)? {
            contents.push(Content::Text(text));
            p_current = p_next;
            continue;
        }

        // If nothing matches, we have a problem.
        return Err(Ast2Error::ParsingError {
            position: p_current.get_position(),
            message: "Unable to parse content".to_string(),
        });
    }

    Ok((contents, p_current)) // Return the final state
}

pub(crate) fn _try_parse_tag<'doc>(parser: &Parser<'doc>) -> Result<Option<(Tag, Parser<'doc>)>> {
    let begin = parser.get_position();

    // Tags can be indented
    let p0 = parser.skip_many_whitespaces_immutable();

    // Must start with '@'
    let p1 = match p0.consume_matching_char_immutable('@') {
        Some(p) => p,
        None => return Ok(None),
    };

    // Then a command
    let (command, p2) = match _try_parse_command_kind(&p1)? {
        Some((c, p)) => (c, p),
        None => return Ok(None), // Not a valid tag if no command
    };

    let p3 = p2.skip_many_whitespaces_immutable();

    // Then optional parameters
    let (parameters, p4) = match _try_parse_parameters(&p3)? {
        Some((p, p_next)) => (p, p_next),
        None => (Parameters::new(), p2.clone()), // No parameters found, use default and continue from p2
    };

    let p5 = p4.skip_many_whitespaces_immutable();

    // Then optional arguments
    let (arguments, p6) = match _try_parse_arguments(&p5)? {
        Some((a, p_next)) => (a, p_next),
        None => (
            Arguments {
                arguments: Vec::new(),
                range: Range::null(),
            },
            p4.clone(),
        ), // No arguments found, use default and continue from p4
    };

    let p7 = p6.skip_many_whitespaces_immutable();

    // Consume EOL if it's there, but don't fail if it's not (e.g. end of file)
    let p8 = p7.consume_matching_char_immutable('\n').unwrap_or(p7);

    let end = p8.get_position();

    let tag = Tag {
        command,
        parameters,
        arguments,
        range: Range { begin, end },
    };

    Ok(Some((tag, p8)))
}

pub(crate) fn _try_parse_anchor<'doc>(
    parser: &Parser<'doc>,
) -> Result<Option<(Anchor, Parser<'doc>)>> {
    let begin = parser.get_position();

    // Anchors can be indented
    let p0 = parser.skip_many_whitespaces_immutable();

    let p1 = match p0.consume_matching_string_immutable("<!--") {
        Some(p) => p,
        None => return Ok(None),
    };

    let p2 = p1.skip_many_whitespaces_immutable();

    let (command, p3) = match _try_parse_command_kind(&p2)? {
        Some((c, p)) => (c, p),
        None => return Ok(None), // Not a valid anchor if no command
    };

    let p4 = match p3.consume_matching_char_immutable('-') {
        Some(p) => p,
        None => {
            return Err(Ast2Error::ParsingError {
                position: p3.get_position(),
                message: "Expected '-' before UUID in anchor".to_string(),
            });
        }
    };

    let (uuid, p5) = match _try_parse_uuid(&p4)? {
        Some((u, p)) => (u, p),
        None => {
            return Err(Ast2Error::InvalidUuid {
                position: p4.get_position(),
            })
        }
    };

    let p6 = match p5.consume_matching_char_immutable(':') {
        Some(p) => p,
        None => {
            return Err(Ast2Error::ParsingError {
                position: p5.get_position(),
                message: "Expected ':' after UUID in anchor".to_string(),
            });
        }
    };

    let (kind, p7) = match _try_parse_anchor_kind(&p6)? {
        Some((k, p)) => (k, p),
        None => {
            return Err(Ast2Error::InvalidAnchorKind {
                position: p6.get_position(),
            })
        }
    };

    let p8 = p7.skip_many_whitespaces_immutable();

    let (parameters, p9) = match _try_parse_parameters(&p8)? {
        Some((p, p_next)) => (p, p_next),
        None => (Parameters::new(), p8.clone()),
    };

    let p10 = p9.skip_many_whitespaces_immutable();

    let (arguments, p11) = match _try_parse_arguments(&p10)? {
        Some((a, p_next)) => (a, p_next),
        None => (
            Arguments {
                arguments: Vec::new(),
                range: Range::null(),
            },
            p10.clone(),
        ),
    };

    let p12 = p11.skip_many_whitespaces_or_eol_immutable();

    let p13 = match p12.consume_matching_string_immutable("-->") {
        Some(p) => p,
        None => {
            return Err(Ast2Error::UnclosedString {
                // Using UnclosedString for a missing -->
                position: p12.get_position(),
            });
        }
    };

    let p14 = p13.skip_many_whitespaces_immutable();

    // Consume EOL if it's there
    let p15 = p14.consume_matching_char_immutable('\n').unwrap_or(p14);

    let end = p15.get_position();

    let anchor = Anchor {
        command,
        uuid,
        kind,
        parameters,
        arguments,
        range: Range { begin, end },
    };

    Ok(Some((anchor, p15)))
}

pub(crate) fn _try_parse_command_kind<'doc>(
    parser: &Parser<'doc>,
) -> Result<Option<(CommandKind, Parser<'doc>)>> {
    let tags_list = vec![
        ("tag", CommandKind::Tag),
        ("include", CommandKind::Include),
        ("inline", CommandKind::Inline),
        ("answer", CommandKind::Answer),
        ("decide", CommandKind::Decide),
        ("choose", CommandKind::Choose),
        ("derive", CommandKind::Derive),
        ("repeat", CommandKind::Repeat),
        ("set", CommandKind::Set),
    ];

    for (name, kind) in tags_list {
        if let Some(new_parser) = parser.consume_matching_string_immutable(name) {
            return Ok(Some((kind, new_parser)));
        }
    }

    Ok(None)
}

pub(crate) fn _try_parse_anchor_kind<'doc>(
    parser: &Parser<'doc>,
) -> Result<Option<(AnchorKind, Parser<'doc>)>> {
    let tags_list = vec![("begin", AnchorKind::Begin), ("end", AnchorKind::End)];

    for (name, kind) in tags_list {
        if let Some(new_parser) = parser.consume_matching_string_immutable(name) {
            return Ok(Some((kind, new_parser)));
        }
    }

    Ok(None)
}

pub(crate) fn _try_parse_parameters<'doc>(
    parser: &Parser<'doc>,
) -> Result<Option<(Parameters, Parser<'doc>)>> {
    let begin = parser.get_position();

    if let Some((json_object, parser)) = _try_parse_jsonplus_object(parser)? {
        let end = parser.get_position();
        return Ok(Some((
            Parameters {
                parameters: json_object,
                range: Range { begin, end },
            },
            parser,
        )));
    } else {
        return Ok(None);
    }
}

pub(crate) fn _try_parse_parameter<'doc>(
    parser: &Parser<'doc>,
) -> Result<Option<((String, serde_json::Value), Parser<'doc>)>> {
    let p_initial = parser.skip_many_whitespaces_or_eol_immutable();

    let (key, p1) = match _try_parse_identifier(&p_initial)? {
        Some((k, p)) => (k, p),
        None => return Ok(None), // Not an error, just didn't find an identifier
    };

    let p2 = p1.skip_many_whitespaces_or_eol_immutable();

    let p3 = match p2.consume_matching_char_immutable('=') {
        Some(p) => p,
        None => return Ok(Some(((key, json!(true)), p2))), // No colon, so not a parameter. Treat as if key = true, continue from p2.
    };

    let p4 = p3.skip_many_whitespaces_or_eol_immutable();

    let (value, p5) = match _try_parse_value(&p4) {
        Ok(Some((v, p))) => (v, p),
        Ok(None) => {
            // Here, a key and colon were found, so a value is expected.
            // This IS a syntax error.
            return Err(Ast2Error::MissingParameterValue {
                position: p4.get_position(),
            });
        }
        Err(e) => return Err(e),
    };

    Ok(Some(((key, value), p5)))
}

pub(crate) fn _try_parse_arguments<'doc>(
    parser: &Parser<'doc>,
) -> Result<Option<(Arguments, Parser<'doc>)>> {
    let begin = parser.get_position();
    let mut p_current = parser.clone();
    let mut arguments = Vec::new();

    loop {
        let p_current_after_whitespaces = p_current.skip_many_whitespaces_immutable();

        // Check for anchor end, a special case for arguments
        if p_current_after_whitespaces.remain().starts_with("-->") {
            break;
        }

        match _try_parse_argument(&p_current_after_whitespaces)? {
            Some((arg, p_next)) => {
                arguments.push(arg);
                p_current = p_next;
            }
            None => break, // No more arguments to parse
        }
    }

    if arguments.is_empty() {
        return Ok(None);
    }

    let end = p_current.get_position();

    Ok(Some((
        Arguments {
            arguments,
            range: Range { begin, end },
        },
        p_current,
    )))
}

pub(crate) fn _try_parse_argument<'doc>(
    parser: &Parser<'doc>,
) -> Result<Option<(Argument, Parser<'doc>)>> {
    let begin = parser.get_position();

    //if let Some(p1) = parser.consume_matching_char_immutable('\'') {
    if let Some((value, p)) = _try_parse_enclosed_string(&parser, "'")? {
        let end = p.get_position();
        let arg = Argument {
            value,
            range: Range { begin, end },
        };
        return Ok(Some((arg, p)));
    }
    //}

    //if let Some(p1) = parser.consume_matching_char_immutable('"') {
    if let Some((value, p)) = _try_parse_enclosed_string(&parser, "\"")? {
        let end = p.get_position();
        let arg = Argument {
            value,
            range: Range { begin, end },
        };
        return Ok(Some((arg, p)));
    }
    //}

    if let Some((value, p)) = _try_parse_nude_string(parser)? {
        let end = p.get_position();
        let arg = Argument {
            value,
            range: Range { begin, end },
        };
        return Ok(Some((arg, p)));
    }

    Ok(None)
}

pub(crate) fn _try_parse_identifier<'doc>(
    parser: &Parser<'doc>,
) -> Result<Option<(String, Parser<'doc>)>> {
    let (first_char, parser1) =
        match parser.consume_char_if_immutable(|c| c.is_alphabetic() || c == '_') {
            Some((c, p)) => (c, p),
            None => return Ok(None),
        };

    let (rest, parser2) = parser1.consume_many_if_immutable(|c| c.is_alphanumeric() || c == '_');

    let mut identifier = String::new();
    identifier.push(first_char);
    identifier.push_str(&rest);

    Ok(Some((identifier, parser2)))
}

pub(crate) fn _try_parse_value<'doc>(
    parser: &Parser<'doc>,
) -> Result<Option<(serde_json::Value, Parser<'doc>)>> {
    /* if let Some(p1) = parser.consume_matching_char_immutable('"') {
        // try to parse a double-quoted string
        _try_parse_enclosed_value(&p1, "\"")
    } else if let Some(p1) = parser.consume_matching_char_immutable('\'') {
        // try to parse a single-quoted string
        _try_parse_enclosed_value(&p1, "\'")
    } else {
        // try to parse a "nude" value (unquoted)
        _try_parse_nude_value(parser)
    } */
    if let Some(x) = _try_parse_enclosed_value(parser, "\"")? {
        Ok(Some(x))
    } else if let Some(x) = _try_parse_enclosed_value(parser, "\'")? {
        Ok(Some(x))
    } else {
        _try_parse_nude_value(parser)
    }
}

pub(crate) fn _try_parse_enclosed_value<'doc>(
    parser: &Parser<'doc>,
    closure: &str,
) -> Result<Option<(serde_json::Value, Parser<'doc>)>> {
    match _try_parse_enclosed_string(parser, closure)? {
        Some((s, p)) => Ok(Some((serde_json::Value::String(s), p))),
        None => Ok(None),
    }
}

pub(crate) fn _try_parse_enclosed_string<'doc>(
    parser: &Parser<'doc>,
    closure: &str,
) -> Result<Option<(String, Parser<'doc>)>> {
    let begin_pos = parser.get_position();
    let mut value = String::new();
    let mut current_parser = parser.clone();

    if let Some(p) = current_parser.consume_matching_string_immutable(closure) {
        current_parser = p;
        loop {
            if let Some(p) = current_parser.consume_matching_string_immutable("\\\"") {
                value.push('\"');
                current_parser = p;
            } else if let Some(p) = current_parser.consume_matching_string_immutable("\\\'") {
                value.push('\'');
                current_parser = p;
            } else if let Some(p) = current_parser.consume_matching_string_immutable("\\n") {
                value.push('\n');
                current_parser = p;
            } else if let Some(p) = current_parser.consume_matching_string_immutable("\\r") {
                value.push('\r');
                current_parser = p;
            } else if let Some(p) = current_parser.consume_matching_string_immutable("\\t") {
                value.push('\t');
                current_parser = p;
            } else if let Some(p) = current_parser.consume_matching_string_immutable("\\\\") {
                value.push('\\');
                current_parser = p;
            } else if let Some(p) = current_parser.consume_matching_string_immutable(closure) {
                return Ok(Some((value, p)));
            } else if current_parser.is_eod() {
                return Err(Ast2Error::UnclosedString {
                    position: begin_pos,
                });
            } else {
                match current_parser.advance_immutable() {
                    None => unreachable!("Checked is_eod() already"),
                    Some((x, p)) => {
                        value.push(x);
                        current_parser = p;
                    }
                }
            }
        }
    }
    Ok(None)
}

pub(crate) fn _try_parse_nude_value<'doc>(
    parser: &Parser<'doc>,
) -> Result<Option<(serde_json::Value, Parser<'doc>)>> {
    if let Some((x, p)) = _try_parse_nude_float(parser)? {
        return Ok(Some((json!(x), p)));
    }
    if let Some((x, p)) = _try_parse_nude_integer(parser)? {
        return Ok(Some((json!(x), p)));
    }
    if let Some((x, p)) = _try_parse_nude_bool(parser)? {
        return Ok(Some((json!(x), p)));
    }
    if let Some((x, p)) = _try_parse_nude_string(parser)? {
        return Ok(Some((json!(x), p)));
    }
    Ok(None)
}

pub(crate) fn _try_parse_nude_integer<'doc>(
    parser: &Parser<'doc>,
) -> Result<Option<(i64, Parser<'doc>)>> {
    let (number_str, new_parser) = parser.consume_many_if_immutable(|x| x.is_digit(10));

    if number_str.is_empty() {
        return Ok(None);
    }

    match i64::from_str_radix(&number_str, 10) {
        Ok(num) => Ok(Some((num, new_parser))),

        Err(e) => Err(Ast2Error::ParseIntError(e)),
    }
}

pub(crate) fn _try_parse_nude_float<'doc>(
    parser: &Parser<'doc>,
) -> Result<Option<(f64, Parser<'doc>)>> {
    let (int_part, p1) = parser.consume_many_if_immutable(|x| x.is_digit(10));

    if let Some(p2) = p1.consume_matching_char_immutable('.') {
        // Found a dot.

        let (frac_part, p3) = p2.consume_many_if_immutable(|x| x.is_digit(10));

        if int_part.is_empty() && frac_part.is_empty() {
            return Ok(None); // Just a dot, not a number
        }

        let num_str = format!("{}.{}", int_part, frac_part);

        match f64::from_str(&num_str) {
            Ok(n) => Ok(Some((n, p3))),

            Err(e) => Err(Ast2Error::ParseFloatError(e)),
        }
    } else {
        // No dot, not a float for our purposes.

        Ok(None)
    }
}

pub(crate) fn _try_parse_nude_bool<'doc>(
    parser: &Parser<'doc>,
) -> Result<Option<(bool, Parser<'doc>)>> {
    if let Some(p) = parser.consume_matching_string_immutable("true") {
        return Ok(Some((true, p)));
    } else if let Some(p) = parser.consume_matching_string_immutable("false") {
        return Ok(Some((false, p)));
    } else {
        return Ok(None);
    }
}

pub(crate) fn _try_parse_nude_string<'doc>(
    parser: &Parser<'doc>,
) -> Result<Option<(String, Parser<'doc>)>> {
    let (result, new_parser) = parser.consume_many_if_immutable(|x| {
        x.is_alphanumeric() || x == '/' || x == '.' || x == '_' || x == '-'
    });

    if result.is_empty() {
        Ok(None)
    } else {
        Ok(Some((result, new_parser)))
    }
}

pub(crate) fn _try_parse_uuid<'doc>(parser: &Parser<'doc>) -> Result<Option<(Uuid, Parser<'doc>)>> {
    let start_pos = parser.get_position();

    let (uuid_str, new_parser) =
        parser.consume_many_if_immutable(|c| c.is_ascii_hexdigit() || c == '-');

    match Uuid::parse_str(&uuid_str) {
        Ok(uuid) => Ok(Some((uuid, new_parser))),

        Err(_) => Err(Ast2Error::InvalidUuid {
            position: start_pos,
        }),
    }
}

pub(crate) fn _try_parse_text<'doc>(parser: &Parser<'doc>) -> Result<Option<(Text, Parser<'doc>)>> {
    let begin = parser.get_position();

    if parser.is_eod() {
        return Ok(None);
    }

    let mut p_current = parser.clone();
    let mut content = String::new();

    loop {
        match p_current.advance_immutable() {
            None => break, // EOD
            Some(('\n', p_next)) => {
                content.push('\n');
                p_current = p_next;
                break; // Consumed newline and stopped
            }
            Some((c, p_next)) => {
                content.push(c);
                p_current = p_next;
            }
        }
    }

    if content.is_empty() {
        return Ok(None);
    }

    let end = p_current.get_position();
    let text = Text {
        content,
        range: Range { begin, end },
    };
    Ok(Some((text, p_current)))
}

pub(crate) fn _try_parse_jsonplus_object<'doc>(
    parser: &Parser<'doc>,
) -> Result<Option<(JsonPlusObject, Parser<'doc>)>> {
    let mut p1 = match parser.consume_matching_char_immutable('{') {
        Some(p) => p,
        None => return Ok(None),
    };

    let mut properties = HashMap::new();

    while !p1.is_eod() {
        p1.skip_many_whitespaces_or_eol();

        if let Some(p2) = p1.consume_matching_string_immutable("}") {
            return Ok(Some((JsonPlusObject { properties }, p2)));
        }

        if let Some((key, mut p3)) = _try_parse_identifier(&p1)? {
            p3.skip_many_whitespaces_or_eol();
            if let Some((_, mut p3)) = p3.consume_char_if_immutable(|x| (x == ':') | (x == '=')) {
                p3.skip_many_whitespaces_or_eol();
                if let Some((object, p4)) = _try_parse_jsonplus_object(&p3)? {
                    properties.insert(key, JsonPlusEntity::Object(object));
                    p1 = p4;
                } else if let Some((array, p4)) = _try_parse_jsonplus_array(&p3)? {
                    properties.insert(key, JsonPlusEntity::Array(array));
                    p1 = p4;
                } else if let Some((single_quoted_content, p4)) =
                    _try_parse_enclosed_string(&p3, "'")?
                {
                    properties.insert(
                        key,
                        JsonPlusEntity::SingleQuotedString(single_quoted_content),
                    );
                    p1 = p4;
                } else if let Some((double_quoted_content, p4)) =
                    _try_parse_enclosed_string(&p3, "\"")?
                {
                    properties.insert(
                        key,
                        JsonPlusEntity::DoubleQuotedString(double_quoted_content),
                    );
                    p1 = p4;
                } else if let Some((x, p4)) = _try_parse_nude_float(&p3)? {
                    properties.insert(key, JsonPlusEntity::Float(x));
                    p1 = p4
                } else if let Some((x, p4)) = _try_parse_nude_integer(&p3)? {
                    properties.insert(key, JsonPlusEntity::Integer(x));
                    p1 = p4
                } else if let Some((x, p4)) = _try_parse_nude_bool(&p3)? {
                    properties.insert(key, JsonPlusEntity::Boolean(x));
                    p1 = p4
                } else if let Some((x, p4)) = _try_parse_nude_string(&p3)? {
                    properties.insert(key, JsonPlusEntity::NudeString(x));
                    p1 = p4
                } else {
                    // No value
                    return Err(Ast2Error::MissingParameterValue {
                        position: p1.get_position(),
                    });
                }
            } else {
                properties.insert(key, JsonPlusEntity::Flag);
                p1 = p3;
            }
        } else {
            // Missing identifier
            return Err(Ast2Error::MissingParameterKey {
                position: p1.get_position(),
            });
        }

        p1.skip_many_whitespaces_or_eol();        
        if let Some(p2) = p1.consume_matching_string_immutable("}") {
            // Will exit on next iteration
        } else if let Some(p2) = p1.consume_matching_string_immutable(",") {
            // Will consume another object on next iteration
            p1 = p2;
        } else {
            // Missing separator
            return Err(Ast2Error::MissingCommaInParameters {
                position: p1.get_position(),
            });
        }
    }

    Err(Ast2Error::UnterminatedObject {
        position: p1.get_position(),
    })
}

pub(crate) fn _try_parse_jsonplus_array<'doc>(
    parser: &Parser<'doc>,
) -> Result<Option<(Vec<JsonPlusEntity>, Parser<'doc>)>> {
    let mut p1 = match parser.consume_matching_char_immutable('[') {
        Some(p) => p,
        None => return Ok(None),
    };

    let mut array = Vec::new();

    while !p1.is_eod() {
        p1.skip_many_whitespaces_or_eol();

        if let Some(p2) = p1.consume_matching_string_immutable("]") {
            return Ok(Some((array, p2)));
        }

        if let Some((object, p4)) = _try_parse_jsonplus_object(&p1)? {
            array.push(JsonPlusEntity::Object(object));
            p1 = p4;
        } else if let Some((array2, p4)) = _try_parse_jsonplus_array(&p1)? {
            array.push(JsonPlusEntity::Array(array2));
            p1 = p4;
        } else if let Some((single_quoted_content, p4)) = _try_parse_enclosed_string(&p1, "'")? {
            array.push(JsonPlusEntity::SingleQuotedString(single_quoted_content));
            p1 = p4;
        } else if let Some((double_quoted_content, p4)) = _try_parse_enclosed_string(&p1, "\"")? {
            array.push(JsonPlusEntity::DoubleQuotedString(double_quoted_content));
            p1 = p4;
        } else if let Some((x, p4)) = _try_parse_nude_float(&p1)? {
            array.push(JsonPlusEntity::Float(x));
            p1 = p4
        } else if let Some((x, p4)) = _try_parse_nude_integer(&p1)? {
            array.push(JsonPlusEntity::Integer(x));
            p1 = p4
        } else if let Some((x, p4)) = _try_parse_nude_bool(&p1)? {
            array.push(JsonPlusEntity::Boolean(x));
            p1 = p4
        } else if let Some((x, p4)) = _try_parse_nude_string(&p1)? {
            array.push(JsonPlusEntity::NudeString(x));
            p1 = p4
        } else {
            return Err(Ast2Error::MalformedValue {
                position: p1.get_position(),
            });
        }

        p1.skip_many_whitespaces_or_eol();
        if let Some(p2) = p1.consume_matching_string_immutable("]") {
            // Will exit on next iteration
        } else if let Some(p2) = p1.consume_matching_string_immutable(",") {
            // Will consume another object on next iteration
            p1 = p2;
        } else {
            // Missing separator
            return Err(Ast2Error::MissingCommaInParameters {
                position: p1.get_position(),
            });
        }
    }

    Err(Ast2Error::UnterminatedArray {
        position: p1.get_position(),
    })
}

#[cfg(test)]
#[path = "./tests/test_parse_anchor.rs"]
mod test_parse_anchor;

#[cfg(test)]
#[path = "./tests/test_parse_argument.rs"]
mod test_parse_argument;

#[cfg(test)]
#[path = "./tests/test_parse_arguments.rs"]
mod test_parse_arguments;

#[cfg(test)]
#[path = "./tests/test_parse_document.rs"]
mod test_parse_document;

#[cfg(test)]
#[path = "./tests/test_parse_enclosed_values.rs"]
mod test_parse_enclosed_values;

#[cfg(test)]
#[path = "./tests/test_position_range.rs"]
mod test_position_range;

#[cfg(test)]
#[path = "./tests/utils.rs"]
mod utils;

#[cfg(test)]
#[path = "./tests/test_parser_advance.rs"]
mod test_parser_advance;

#[cfg(test)]
#[path = "./tests/test_parser_consume.rs"]
mod test_parser_consume;

#[cfg(test)]
#[path = "./tests/test_parse_uuid.rs"]
mod test_parse_uuid;

#[cfg(test)]
#[path = "./tests/test_parse_text.rs"]
mod test_parse_text;

#[cfg(test)]
#[path = "./tests/test_parse_identifier.rs"]
mod test_parse_identifier;

#[cfg(test)]
#[path = "./tests/test_parse_kinds.rs"]
mod test_parse_kinds;

#[cfg(test)]
#[path = "./tests/test_parse_nude_values.rs"]
mod test_parse_nude_values;

#[cfg(test)]
#[path = "./tests/test_parse_parameters.rs"]
mod test_parse_parameters;

#[cfg(test)]
#[path = "./tests/test_parse_tag.rs"]
mod test_parse_tag;

#[cfg(test)]
#[path = "./tests/test_parse_jsonplus.rs"]
mod test_parse_jsonplus;
