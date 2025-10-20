use super::parser::Parser;
use super::error::{Ast2Error, Result};
use super::types::{Tag, Anchor, Text, Content, Document};
use super::parse_elements::{_try_parse_command_kind, _try_parse_anchor_kind, _try_parse_parameters, _try_parse_arguments};
use super::parse_primitives::_try_parse_uuid;

pub fn parse_document(document: &str) -> Result<Document> {
    let parser = Parser::new(document);
    let begin = parser.get_position();
    
    let (content, parser_after_content) = parse_content(parser)?;
    
    let end = parser_after_content.get_position();

    Ok(Document {
        content: content,
        range: super::types::Range { begin, end },
    })
}

pub(crate) fn parse_content<'doc>(parser: Parser<'doc>) -> Result<(Vec<Content>, Parser<'doc>)> {
    let mut contents = Vec::new();
    let mut p_current = parser; // Takes ownership

    loop {
        if p_current.is_eod() {
            break;
        }

        // TODO controlla di essere ad inizio linea. se non e' cosi, PROBLEMA perche'
        // le subroutine devono SEMPRE fermarsi ad un inizio linea.
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

    // Must start with '@'
    let p1 = match parser.consume_matching_char_immutable('@') {
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
        None => (super::types::Parameters::new(), p2.clone()), // No parameters found, use default and continue from p2
    };

    let p5 = p4.skip_many_whitespaces_immutable();

    // Then optional arguments
    let (arguments, p6) = match _try_parse_arguments(&p5)? {
        Some((a, p_next)) => (a, p_next),
        None => (
            super::types::Arguments {
                arguments: Vec::new(),
                range: super::types::Range::null(),
            },
            p4.clone(),
        ), // No arguments found, use default and continue from p4
    };

    let end = p6.get_position();

    let p7 = p6.skip_many_whitespaces_immutable();

    // Consume EOL if it's there, but don't fail if it's not (e.g. end of file)
    let p8 = p7.consume_matching_char_immutable('\n').unwrap_or(p7);

    let tag = Tag {
        command,
        parameters,
        arguments,
        range: super::types::Range { begin, end },
    };

    Ok(Some((tag, p8)))
}

pub(crate) fn _try_parse_anchor<'doc>(parser: &Parser<'doc>) -> Result<Option<(Anchor, Parser<'doc>)>> {
    let begin = parser.get_position();

    let p1 = match parser.consume_matching_string_immutable("<!--") {
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
        None => (super::types::Parameters::new(), p8.clone()),
    };

    let p10 = p9.skip_many_whitespaces_immutable();

    let (arguments, p11) = match _try_parse_arguments(&p10)? {
        Some((a, p_next)) => (a, p_next),
        None => (
            super::types::Arguments {
                arguments: Vec::new(),
                range: super::types::Range::null(),
            },
            p10.clone(),
        ),
    };

    let p12 = p11.skip_many_whitespaces_or_eol_immutable();

    let p13 = match p12.consume_matching_string_immutable("-->") {
        Some(p) => p,
        None => {
            return Err(Ast2Error::UnclosedString { // Using UnclosedString for a missing -->
                position: p12.get_position(),
            });
        }
    };

    let end = p13.get_position();

    let p14 = p13.skip_many_whitespaces_or_eol_immutable();

    // Consume EOL if it's there
    let p15 = p14.consume_matching_char_immutable('\n').unwrap_or(p14);

    let anchor = Anchor {
        command,
        uuid,
        kind,
        parameters,
        arguments,
        range: super::types::Range { begin, end },
    };

    Ok(Some((anchor, p15)))
}

pub(crate) fn _try_parse_text<'doc>(parser: &Parser<'doc>) -> Result<Option<(Text, Parser<'doc>)>> {
    let begin = parser.get_position();

    if parser.is_eod() {
        return Ok(None);
    }

    let mut p_current = parser.clone();
    let mut content_len = 0; // Track content length to avoid building string until needed

    loop {
        // Check if the next characters would start a tag or anchor
        if p_current.remain().starts_with('@') || p_current.remain().starts_with("<!--") {
            break;
        }

        match p_current.advance_immutable() {
            None => break, // EOD
            Some((c, p_next)) => {
                content_len += c.len_utf8(); // Increment byte length
                p_current = p_next;
                if c == '\n' {
                    break; // Consumed newline and stopped
                }
            }
        }
    }

    if content_len == 0 {
        return Ok(None);
    }

    let end = p_current.get_position();
    let text = Text { range: super::types::Range { begin, end } };
    Ok(Some((text, p_current)))
}

#[cfg(test)]
mod tests {
    use crate::ast2::parser::Parser;
    use crate::ast2::error::{Ast2Error, Result};
    use crate::ast2::types::{Tag, Anchor, Text, Content, Document, CommandKind, AnchorKind};
    use super::{_try_parse_text, _try_parse_tag, _try_parse_anchor, parse_content};
    use serde_json::json;
    use uuid::Uuid;

    #[test]
    fn test_try_parse_text_simple() {
        let doc = "hello world";
        let parser = Parser::new(doc);
        let (text, p_next) = _try_parse_text(&parser).unwrap().unwrap();
        assert_eq!(p_next.remain(), "");

        let text_str = "hello world";
        assert_eq!(text.range.begin.offset, 0);
        assert_eq!(text.range.end.offset, text_str.len());
    }

    #[test]
    fn test_try_parse_text_until_tag() {
        let doc = "hello @tag rest";
        let parser = Parser::new(doc);
        let (text, p_next) = _try_parse_text(&parser).unwrap().unwrap();
        assert_eq!(p_next.remain(), "@tag rest"); // Corrected remain()
        assert_eq!(text.range.begin.offset, 0);
        assert_eq!(text.range.end.offset, "hello ".len()); // Corrected end offset
    }

    #[test]
    fn test_try_parse_text_until_anchor() {
        let doc = "hello <!-- anchor --> rest";
        let parser = Parser::new(doc);
        let (text, p_next) = _try_parse_text(&parser).unwrap().unwrap();
        assert_eq!(p_next.remain(), "<!-- anchor --> rest"); // Corrected remain()
        assert_eq!(text.range.begin.offset, 0);
        assert_eq!(text.range.end.offset, "hello ".len()); // Corrected end offset
    }

    #[test]
    fn test_try_parse_text_with_newline() {
        let doc = "line1
line2 rest";
        let parser = Parser::new(doc);
        let (text, p_next) = _try_parse_text(&parser).unwrap().unwrap();
        assert_eq!(p_next.remain(), "line2 rest");
        assert_eq!(p_next.get_position().line, 2);
        assert_eq!(p_next.get_position().column, 1);

        let text_str = "line1
";
        assert_eq!(text.range.begin.offset, 0);
        assert_eq!(text.range.end.offset, text_str.len());
    }

    #[test]
    fn test_try_parse_text_empty() {
        let doc = "";
        let parser = Parser::new(doc);
        let result = _try_parse_text(&parser).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_try_parse_text_starts_with_tag() {
        let doc = "@tag rest";
        let parser = Parser::new(doc);
        let result = _try_parse_text(&parser).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_try_parse_text_starts_with_anchor() {
        let doc = "<!-- anchor --> rest";
        let parser = Parser::new(doc);
        let result = _try_parse_text(&parser).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_try_parse_tag_simple() {
        let doc = "@tag ";
        let parser = Parser::new(doc);
        let (tag, p_next) = _try_parse_tag(&parser).unwrap().unwrap();
        assert_eq!(tag.command, CommandKind::Tag);
        assert!(tag.parameters.parameters.is_empty());
        assert!(tag.arguments.arguments.is_empty());
        assert_eq!(p_next.remain(), "");

        let tag_str = "@tag";
        assert_eq!(tag.range.begin.offset, 0);
        assert_eq!(tag.range.end.offset, tag_str.len());
    }

    #[test]
    fn test_try_parse_tag_with_parameters() {
        let doc = "@include [file=\"path/to/file.txt\"] ";
        let parser = Parser::new(doc);
        let (tag, p_next) = _try_parse_tag(&parser).unwrap().unwrap();
        assert_eq!(tag.command, CommandKind::Include);
        assert_eq!(tag.parameters.parameters.len(), 1);
        assert_eq!(tag.parameters.parameters["file"], json!("path/to/file.txt"));
        assert!(tag.arguments.arguments.is_empty());
        assert_eq!(p_next.remain(), "");

        let tag_str = "@include [file=\"path/to/file.txt\"]";
        assert_eq!(tag.range.begin.offset, 0);
        assert_eq!(tag.range.end.offset, tag_str.len());
    }

    #[test]
    fn test_try_parse_tag_with_arguments() {
        let doc = "@inline 'arg1' \"arg2\" ";
        let parser = Parser::new(doc);
        let (tag, p_next) = _try_parse_tag(&parser).unwrap().unwrap();
        assert_eq!(tag.command, CommandKind::Inline);
        assert!(tag.parameters.parameters.is_empty());
        assert_eq!(tag.arguments.arguments.len(), 2);
        assert_eq!(tag.arguments.arguments[0].value, "arg1");
        assert_eq!(tag.arguments.arguments[1].value, "arg2");
        assert_eq!(p_next.remain(), "");

        let tag_str = "@inline 'arg1' \"arg2\"";
        assert_eq!(tag.range.begin.offset, 0);
        assert_eq!(tag.range.end.offset, tag_str.len());
    }

    #[test]
    fn test_try_parse_tag_with_parameters_and_arguments() {
        let doc = "@answer [id=123] 'arg1' ";
        let parser = Parser::new(doc);
        let (tag, p_next) = _try_parse_tag(&parser).unwrap().unwrap();
        assert_eq!(tag.command, CommandKind::Answer);
        assert_eq!(tag.parameters.parameters.len(), 1);
        assert_eq!(tag.parameters.parameters["id"], json!(123));
        assert_eq!(tag.arguments.arguments.len(), 1);
        assert_eq!(tag.arguments.arguments[0].value, "arg1");
        assert_eq!(p_next.remain(), "");

        let tag_str = "@answer [id=123] 'arg1'";
        assert_eq!(tag.range.begin.offset, 0);
        assert_eq!(tag.range.end.offset, tag_str.len());
    }

    #[test]
    fn test_try_parse_tag_no_at_sign() {
        let doc = "tag ";
        let parser = Parser::new(doc);
        let result = _try_parse_tag(&parser).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_try_parse_tag_invalid_command() {
        let doc = "@invalid_command ";
        let parser = Parser::new(doc);
        let result = _try_parse_tag(&parser).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_try_parse_tag_with_eol() {
        let doc = "@tag
rest";
        let parser = Parser::new(doc);
        let (tag, p_next) = _try_parse_tag(&parser).unwrap().unwrap();
        assert_eq!(tag.command, CommandKind::Tag);
        assert_eq!(p_next.remain(), "rest");
        assert_eq!(p_next.get_position().line, 2);
        assert_eq!(p_next.get_position().column, 1);
    }

    #[test]
    fn test_try_parse_anchor_simple() {
        let uuid_str = "123e4567-e89b-12d3-a456-426614174000";
        let doc = format!("<!-- tag-{}:begin --> rest", uuid_str);
        let parser = Parser::new(&doc);
        let (anchor, p_next) = _try_parse_anchor(&parser).unwrap().unwrap();
        assert_eq!(anchor.command, CommandKind::Tag);
        assert_eq!(anchor.uuid, Uuid::parse_str(uuid_str).unwrap());
        assert_eq!(anchor.kind, AnchorKind::Begin);
        assert!(anchor.parameters.parameters.is_empty());
        assert!(anchor.arguments.arguments.is_empty());
        assert_eq!(p_next.remain(), "rest");

        let anchor_full_str = format!("<!-- tag-{}:begin -->", uuid_str);
        assert_eq!(anchor.range.begin.offset, 0);
        assert_eq!(anchor.range.end.offset, anchor_full_str.len());
    }

    #[test]
    fn test_try_parse_anchor_with_parameters() {
        let uuid_str = "123e4567-e89b-12d3-a456-426614174000";
        let doc = format!("<!-- include-{}:end [file=\"path.txt\"] --> rest", uuid_str);
        let parser = Parser::new(&doc);
        let (anchor, p_next) = _try_parse_anchor(&parser).unwrap().unwrap();
        assert_eq!(anchor.command, CommandKind::Include);
        assert_eq!(anchor.uuid, Uuid::parse_str(uuid_str).unwrap());
        assert_eq!(anchor.kind, AnchorKind::End);
        assert_eq!(anchor.parameters.parameters.len(), 1);
        assert_eq!(anchor.parameters.parameters["file"], json!("path.txt"));
        assert!(anchor.arguments.arguments.is_empty());
        assert_eq!(p_next.remain(), "rest");

        let anchor_full_str = format!("<!-- include-{}:end [file=\"path.txt\"] -->", uuid_str);
        assert_eq!(anchor.range.begin.offset, 0);
        assert_eq!(anchor.range.end.offset, anchor_full_str.len());
    }

    #[test]
    fn test_try_parse_anchor_with_arguments() {
        let uuid_str = "123e4567-e89b-12d3-a456-426614174000";
        let doc = format!("<!-- inline-{}:begin 'arg1' \"arg2\" --> rest", uuid_str);
        let parser = Parser::new(&doc);
        let (anchor, p_next) = _try_parse_anchor(&parser).unwrap().unwrap();
        assert_eq!(anchor.command, CommandKind::Inline);
        assert_eq!(anchor.uuid, Uuid::parse_str(uuid_str).unwrap());
        assert_eq!(anchor.kind, AnchorKind::Begin);
        assert!(anchor.parameters.parameters.is_empty());
        assert_eq!(anchor.arguments.arguments.len(), 2);
        assert_eq!(anchor.arguments.arguments[0].value, "arg1");
        assert_eq!(anchor.arguments.arguments[1].value, "arg2");
        assert_eq!(p_next.remain(), "rest");

        let anchor_full_str = format!("<!-- inline-{}:begin 'arg1' \"arg2\" -->", uuid_str);
        assert_eq!(anchor.range.begin.offset, 0);
        assert_eq!(anchor.range.end.offset, anchor_full_str.len());
    }

    #[test]
    fn test_try_parse_anchor_missing_closing_tag() {
        let uuid_str = "123e4567-e89b-12d3-a456-426614174000";
        let doc = format!("<!-- tag-{}:begin rest", uuid_str);
        let parser = Parser::new(&doc);
        let result = _try_parse_anchor(&parser);
        assert!(matches!(result, Err(Ast2Error::UnclosedString { .. })));
    }

    #[test]
    fn test_try_parse_anchor_invalid_uuid() {
        let doc = "<!-- tag-invalid-uuid:begin --> rest";
        let parser = Parser::new(doc);
        let result = _try_parse_anchor(&parser);
        assert!(matches!(result, Err(Ast2Error::InvalidUuid { .. })));
    }

    #[test]
    fn test_try_parse_anchor_no_opening_comment() {
        let doc = "tag-uuid:begin --> rest";
        let parser = Parser::new(doc);
        let result = _try_parse_anchor(&parser).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_content_mixed() {
        let uuid_str = "123e4567-e89b-12d3-a456-426614174000";
        let doc = format!("Some text\n@tag [param=1] 'arg1'\n<!-- include-{}:begin -->\nmore text", uuid_str);
        let parser = Parser::new(&doc);
        let (content_vec, p_next) = parse_content(parser).unwrap();

        assert_eq!(content_vec.len(), 4);

        // Text 1
        if let Content::Text(text) = &content_vec[0] {
            assert_eq!(text.range.begin.offset, 0);
            assert_eq!(text.range.end.offset, "Some text\n".len());
        } else {
            panic!("Expected Text");
        }

        // Tag
        if let Content::Tag(tag) = &content_vec[1] {
            assert_eq!(tag.command, CommandKind::Tag);
            assert_eq!(tag.parameters.parameters["param"], json!(1));
            assert_eq!(tag.arguments.arguments[0].value, "arg1");
            let tag_str = "@tag [param=1] 'arg1'";
            assert_eq!(tag.range.begin.offset, "Some text ".len());
            assert_eq!(tag.range.end.offset, "Some text ".len() + tag_str.len());
        } else {
            panic!("Expected Tag");
        }

        // Anchor
        if let Content::Anchor(anchor) = &content_vec[2] {
            assert_eq!(anchor.command, CommandKind::Include);
            assert_eq!(anchor.uuid, Uuid::parse_str(uuid_str).unwrap());
            assert_eq!(anchor.kind, AnchorKind::Begin);
            let anchor_str = format!("<!-- include-{}:begin -->", uuid_str);
            assert_eq!(anchor.range.begin.offset, "Some text @tag\n[param=1] 'arg1'\n".len());
            assert_eq!(anchor.range.end.offset, "Some text @tag\n[param=1] 'arg1'\n".len() + anchor_str.len());
        } else {
            panic!("Expected Anchor");
        }

        // Text 2
        if let Content::Text(text) = &content_vec[3] {
            assert_eq!(text.range.begin.offset, doc.len() - "more text".len());
            assert_eq!(text.range.end.offset, doc.len());
        } else {
            panic!("Expected Text");
        }

        assert!(p_next.is_eod());
    }
}
