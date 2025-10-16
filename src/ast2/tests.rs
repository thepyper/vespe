use super::*;
use anyhow::Result;
use uuid::Uuid;

fn create_pos(offset: usize, line: usize, column: usize) -> Position {
    Position { offset, line, column }
}

fn create_range(begin_offset: usize, begin_line: usize, begin_column: usize, end_offset: usize, end_line: usize, end_column: usize) -> Range {
    Range {
        begin: create_pos(begin_offset, begin_line, begin_column),
        end: create_pos(end_offset, end_line, end_column),
    }
}

#[test]
fn test_parser_new() {
    let document = "Hello";
    let parser = Parser::new(document);
    assert_eq!(parser.get_position(), create_pos(0, 1, 1));
    assert_eq!(parser.remain(), "Hello");
}

#[test]
fn test_parser_advance() {
    let document = "Hello";
    let mut parser = Parser::new(document);
    assert_eq!(parser.advance(), Some('H'));
    assert_eq!(parser.get_position(), create_pos(1, 1, 2));
    assert_eq!(parser.advance(), Some('e'));
    assert_eq!(parser.get_position(), create_pos(2, 1, 3));
    parser.advance();
    parser.advance();
    parser.advance();
    assert_eq!(parser.advance(), None);
    assert_eq!(parser.get_position(), create_pos(5, 1, 6));
}

#[test]
fn test_parser_advance_newline() {
    let document = "H\nello";
    let mut parser = Parser::new(document);
    assert_eq!(parser.advance(), Some('H'));
    assert_eq!(parser.get_position(), create_pos(1, 1, 2));
    assert_eq!(parser.advance(), Some('\n'));
    assert_eq!(parser.get_position(), create_pos(2, 2, 1));
}

#[test]
fn test_parser_consume_matching_char() {
    let document = "abc";
    let mut parser = Parser::new(document);
    assert!(parser.consume_matching_char('a'));
    assert_eq!(parser.get_position(), create_pos(1, 1, 2));
    assert!(!parser.consume_matching_char('c'));
    assert_eq!(parser.get_position(), create_pos(1, 1, 2));
}

#[test]
fn test_parser_consume_matching_string() {
    let document = "hello world";
    let mut parser = Parser::new(document);
    assert!(parser.consume_matching_string("hello"));
    assert_eq!(parser.get_position(), create_pos(5, 1, 6));
    assert!(!parser.consume_matching_string("world"));
    assert_eq!(parser.get_position(), create_pos(5, 1, 6));
}

#[test]
fn test_parser_consume_char_if() {
    let document = "123abc";
    let mut parser = Parser::new(document);
    assert_eq!(parser.consume_char_if(|c| c.is_ascii_digit()), Some('1'));
    assert_eq!(parser.get_position(), create_pos(1, 1, 2));
    assert_eq!(parser.consume_char_if(|c| c.is_ascii_digit()), Some('2'));
    assert_eq!(parser.get_position(), create_pos(2, 1, 3));
    assert_eq!(parser.consume_char_if(|c| c.is_ascii_alphabetic()), None);
    assert_eq!(parser.get_position(), create_pos(2, 1, 3));
}

#[test]
fn test_parser_skip_many_whitespaces() {
    let document = "  \t\rhello";
    let mut parser = Parser::new(document);
    parser.skip_many_whitespaces();
    assert_eq!(parser.get_position(), create_pos(4, 1, 5));
    assert_eq!(parser.remain(), "hello");
}

#[test]
fn test_parser_skip_many_whitespaces_or_eol() {
    let document = "  \n\r\thello";
    let mut parser = Parser::new(document);
    parser.skip_many_whitespaces_or_eol();
    assert_eq!(parser.get_position(), create_pos(5, 2, 2));
    assert_eq!(parser.remain(), "hello");
}

#[test]
fn test_try_parse_nude_integer() -> Result<()> {
    let mut parser = Parser::new("123");
    assert_eq!(_try_parse_nude_integer(&mut parser)?,
               Some(123));
    assert_eq!(parser.get_position(), create_pos(3, 1, 4));

    let mut parser = Parser::new("abc");
    assert_eq!(_try_parse_nude_integer(&mut parser)?,
               None);
    assert_eq!(parser.get_position(), create_pos(0, 1, 1));

    let mut parser = Parser::new("");
    assert_eq!(_try_parse_nude_integer(&mut parser)?,
               None);
    assert_eq!(parser.get_position(), create_pos(0, 1, 1));
    Ok(())
}

#[test]
fn test_try_parse_nude_float() -> Result<()> {
    let mut parser = Parser::new("123.45");
    assert_eq!(_try_parse_nude_float(&mut parser)?,
               Some(123.45));
    assert_eq!(parser.get_position(), create_pos(6, 1, 7));

    let mut parser = Parser::new(".5");
    assert_eq!(_try_parse_nude_float(&mut parser)?,
               Some(0.5));
    assert_eq!(parser.get_position(), create_pos(2, 1, 3));

    let mut parser = Parser::new("123");
    assert_eq!(_try_parse_nude_float(&mut parser)?,
               None);
    assert_eq!(parser.get_position(), create_pos(0, 1, 1));

    let mut parser = Parser::new("abc");
    assert_eq!(_try_parse_nude_float(&mut parser)?,
               None);
    assert_eq!(parser.get_position(), create_pos(0, 1, 1));

    let mut parser = Parser::new("");
    assert_eq!(_try_parse_nude_float(&mut parser)?,
               None);
    assert_eq!(parser.get_position(), create_pos(0, 1, 1));
    Ok(())
}

#[test]
fn test_try_parse_nude_bool() -> Result<()> {
    let mut parser = Parser::new("true");
    assert_eq!(_try_parse_nude_bool(&mut parser)?,
               Some(true));
    assert_eq!(parser.get_position(), create_pos(4, 1, 5));

    let mut parser = Parser::new("false");
    assert_eq!(_try_parse_nude_bool(&mut parser)?,
               Some(false));
    assert_eq!(parser.get_position(), create_pos(5, 1, 6));

    let mut parser = Parser::new("other");
    assert_eq!(_try_parse_nude_bool(&mut parser)?,
               None);
    assert_eq!(parser.get_position(), create_pos(0, 1, 1));
    Ok(())
}

#[test]
fn test_try_parse_nude_string() -> Result<()> {
    let mut parser = Parser::new("hello_world-1.0/path");
    assert_eq!(_try_parse_nude_string(&mut parser)?,
               Some("hello_world-1.0/path".to_string()));
    assert_eq!(parser.get_position(), create_pos(20, 1, 21));

    let mut parser = Parser::new(" hello");
    assert_eq!(_try_parse_nude_string(&mut parser)?,
               None);
    assert_eq!(parser.get_position(), create_pos(0, 1, 1));
    Ok(())
}

#[test]
fn test_try_parse_enclosed_value_double_quote() -> Result<()> {
    let mut parser = Parser::new("\"hello world\"");
    let value = _try_parse_enclosed_value(&mut parser, "\"")?;
    assert_eq!(value, Some(json!("hello world")));
    assert_eq!(parser.get_position(), create_pos(13, 1, 14));
    Ok(())
}

#[test]
fn test_try_parse_enclosed_value_single_quote() -> Result<()> {
    let mut parser = Parser::new("'hello world'" );
    let value = _try_parse_enclosed_value(&mut parser, "'")?;
    assert_eq!(value, Some(json!("hello world")));
    assert_eq!(parser.get_position(), create_pos(13, 1, 14));
    Ok(())
}

#[test]
fn test_try_parse_enclosed_value_with_escapes() -> Result<()> {
    let mut parser = Parser::new("\"hello\nworld\t\"\"" );
    let value = _try_parse_enclosed_value(&mut parser, "\"")?;
    assert_eq!(value, Some(json!("hello\nworld\t\"")));
    assert_eq!(parser.get_position(), create_pos(18, 1, 19));
    Ok(())
}

#[test]
fn test_try_parse_enclosed_value_unterminated() -> Result<()> {
    let mut parser = Parser::new("\"hello");
    let result = _try_parse_enclosed_value(&mut parser, "\"");
    assert!(matches!(result, Err(e) if e.downcast_ref::<ParsingError>().unwrap().to_string().contains("Unterminated string")));
    Ok(())
}

#[test]
fn test_try_parse_identifier() -> Result<()> {
    let mut parser = Parser::new("my_variable123");
    assert_eq!(_try_parse_identifier(&mut parser)?,
               Some("my_variable123".to_string()));
    assert_eq!(parser.get_position(), create_pos(14, 1, 15));

    let mut parser = Parser::new("123variable");
    assert_eq!(_try_parse_identifier(&mut parser)?,
               None);
    assert_eq!(parser.get_position(), create_pos(0, 1, 1));
    Ok(())
}

#[test]
fn test_try_parse_parameter() -> Result<()> {
    let mut parser = Parser::new("key: \"value\"" );
    let (key, value) = _try_parse_parameter(&mut parser)?.unwrap();
    assert_eq!(key, "key");
    assert_eq!(value, json!("value"));
    assert_eq!(parser.get_position(), create_pos(12, 1, 13));

    let mut parser = Parser::new("number: 123");
    let (key, value) = _try_parse_parameter(&mut parser)?.unwrap();
    assert_eq!(key, "number");
    assert_eq!(value, json!(123));
    assert_eq!(parser.get_position(), create_pos(11, 1, 12));

    let mut parser = Parser::new("boolean: true");
    let (key, value) = _try_parse_parameter(&mut parser)?.unwrap();
    assert_eq!(key, "boolean");
    assert_eq!(value, json!(true));
    assert_eq!(parser.get_position(), create_pos(13, 1, 14));

    let mut parser = Parser::new("float: 1.23");
    let (key, value) = _try_parse_parameter(&mut parser)?.unwrap();
    assert_eq!(key, "float");
    assert_eq!(value, json!(1.23));
    assert_eq!(parser.get_position(), create_pos(11, 1, 12));
    Ok(())
}

#[test]
fn test_try_parse_parameters0_empty() -> Result<()> {
    let mut parser = Parser::new("{}");
    let params = _try_parse_parameters0(&mut parser)?.unwrap();
    assert!(params.parameters.as_object().unwrap().is_empty());
    assert_eq!(params.range, create_range(0, 1, 1, 2, 1, 3));
    Ok(())
}

#[test]
fn test_try_parse_parameters0_single() -> Result<()> {
    let mut parser = Parser::new("{key: \"value\"}");
    let params = _try_parse_parameters0(&mut parser)?.unwrap();
    assert_eq!(params.parameters["key"], json!("value"));
    assert_eq!(params.range, create_range(0, 1, 1, 14, 1, 15));
    Ok(())
}

#[test]
fn test_try_parse_parameters0_multiple() -> Result<()> {
    let mut parser = Parser::new("{key1: \"value1\", key2: 123, key3: true}");
    let params = _try_parse_parameters0(&mut parser)?.unwrap();
    assert_eq!(params.parameters["key1"], json!("value1"));
    assert_eq!(params.parameters["key2"], json!(123));
    assert_eq!(params.parameters["key3"], json!(true));
    assert_eq!(params.range, create_range(0, 1, 1, 40, 1, 41));
    Ok(())
}

#[test]
fn test_try_parse_parameters0_multiline() -> Result<()> {
    let mut parser = Parser::new("{ \n  key1: \"value1\",\n  key2: 123\n}");
    let params = _try_parse_parameters0(&mut parser)?.unwrap();
    assert_eq!(params.parameters["key1"], json!("value1"));
    assert_eq!(params.parameters["key2"], json!(123));
    assert_eq!(params.range, create_range(0, 1, 1, 30, 4, 2));
    Ok(())
}

#[test]
fn test_try_parse_parameters0_missing_colon() -> Result<()> {
    let mut parser = Parser::new("{key value}");
    let result = _try_parse_parameters0(&mut parser);
    assert!(matches!(result, Err(e) if e.downcast_ref::<ParsingError>().unwrap().to_string().contains("Expected : ")));
    Ok(())
}

#[test]
fn test_try_parse_parameters0_missing_value() -> Result<()> {
    let mut parser = Parser::new("{key: }");
    let result = _try_parse_parameters0(&mut parser);
    assert!(matches!(result, Err(e) if e.downcast_ref::<ParsingError>().unwrap().to_string().contains("Expected parameter value")));
    Ok(())
}

#[test]
fn test_try_parse_parameters0_missing_comma_or_brace() -> Result<()> {
    let mut parser = Parser::new("{key1: \"value1\" key2: 123}");
    let result = _try_parse_parameters0(&mut parser);
    assert!(matches!(result, Err(e) if e.downcast_ref::<ParsingError>().unwrap().to_string().contains("Expected , or } ")));
    Ok(())
}

#[test]
fn test_try_parse_argument_quoted() -> Result<()> {
    let mut parser = Parser::new("\"hello world\"");
    let arg = _try_parse_argument(&mut parser)?.unwrap();
    assert_eq!(arg.value, "hello world");
    assert_eq!(arg.range, create_range(0, 1, 1, 13, 1, 14));
    Ok(())
}

#[test]
fn test_try_parse_argument_unquoted() -> Result<()> {
    let mut parser = Parser::new("hello_world-1.0/path");
    let arg = _try_parse_argument(&mut parser)?.unwrap();
    assert_eq!(arg.value, "hello_world-1.0/path");
    assert_eq!(arg.range, create_range(0, 1, 1, 20, 1, 21));
    Ok(())
}

#[test]
fn test_try_parse_arguments_single() -> Result<()> {
    let mut parser = Parser::new("arg1");
    let args = _try_parse_arguments(&mut parser)?.unwrap();
    assert_eq!(args.arguments.len(), 1);
    assert_eq!(args.arguments[0].value, "arg1");
    assert_eq!(args.range, create_range(0, 1, 1, 4, 1, 5));
    Ok(())
}

#[test]
fn test_try_parse_arguments_multiple() -> Result<()> {
    let mut parser = Parser::new("arg1 \"arg2 with spaces\" arg3");
    let args = _try_parse_arguments(&mut parser)?.unwrap();
    assert_eq!(args.arguments.len(), 3);
    assert_eq!(args.arguments[0].value, "arg1");
    assert_eq!(args.arguments[1].value, "arg2 with spaces");
    assert_eq!(args.arguments[2].value, "arg3");
    assert_eq!(args.range, create_range(0, 1, 1, 30, 1, 31));
    Ok(())
}

#[test]
fn test_try_parse_arguments_multiline() -> Result<()> {
    let mut parser = Parser::new("arg1\narg2");
    let args = _try_parse_arguments(&mut parser)?.unwrap();
    assert_eq!(args.arguments.len(), 1);
    assert_eq!(args.arguments[0].value, "arg1");
    assert_eq!(args.range, create_range(0, 1, 1, 4, 1, 5));
    assert_eq!(parser.get_position(), create_pos(4, 1, 5)); // Should stop before newline
    Ok(())
}

#[test]
fn test_try_parse_command_kind() -> Result<()> {
    let mut parser = Parser::new("answer");
    assert!(matches!(_try_parse_command_kind("", &mut parser)?,
              Some(CommandKind::Answer)));
    assert_eq!(parser.get_position(), create_pos(6, 1, 7));
    Ok(())
}

#[test]
fn test_try_parse_uuid() -> Result<()> {
    let uuid_str = "123e4567-e89b-12d3-a456-426614174000";
    let mut parser = Parser::new(uuid_str);
    let uuid = _try_parse_uuid(&mut parser)?.unwrap();
    assert_eq!(uuid.to_string(), uuid_str);
    assert_eq!(parser.get_position(), create_pos(36, 1, 37));

    let mut parser = Parser::new("invalid-uuid");
    let result = _try_parse_uuid(&mut parser);
    assert!(matches!(result, Err(e) if e.downcast_ref::<ParsingError>().unwrap().to_string().contains("Invalid UUID format")));
    Ok(())
}

#[test]
fn test_try_parse_anchor_kind() -> Result<()> {
    let mut parser = Parser::new("begin");
    assert!(matches!(_try_parse_anchor_kind("", &mut parser)?,
              Some(AnchorKind::Begin)));
    assert_eq!(parser.get_position(), create_pos(5, 1, 6));
    Ok(())
}

#[test]
fn test_try_parse_tag0_simple() -> Result<()> {
    let mut parser = Parser::new("@answer arg1 arg2\n");
    let tag = _try_parse_tag0("", &mut parser)?.unwrap();
    assert!(matches!(tag.command, CommandKind::Answer));
    assert_eq!(tag.arguments.arguments.len(), 2);
    assert_eq!(tag.arguments.arguments[0].value, "arg1");
    assert_eq!(tag.arguments.arguments[1].value, "arg2");
    assert_eq!(tag.range, create_range(0, 1, 1, 17, 1, 18));
    Ok(())
}

#[test]
fn test_try_parse_tag0_with_parameters() -> Result<()> {
    let mut parser = Parser::new("@inline {key: \"value\"} arg1\n");
    let tag = _try_parse_tag0("", &mut parser)?.unwrap();
    assert!(matches!(tag.command, CommandKind::Inline));
    assert_eq!(tag.parameters.parameters["key"], json!("value"));
    assert_eq!(tag.arguments.arguments.len(), 1);
    assert_eq!(tag.arguments.arguments[0].value, "arg1");
    assert_eq!(tag.range, create_range(0, 1, 1, 29, 1, 30));
    Ok(())
}

#[test]
fn test_try_parse_anchor0_simple() -> Result<()> {
    let uuid_str = "123e4567-e89b-12d3-a456-426614174000";
    let input = format!("<!-- answer-{}:begin -->\n", uuid_str);
    let mut parser = Parser::new(&input);
    let anchor = _try_parse_anchor0("", &mut parser)?.unwrap();
    assert!(matches!(anchor.command, CommandKind::Answer));
    assert_eq!(anchor.uuid.to_string(), uuid_str);
    assert!(matches!(anchor.kind, AnchorKind::Begin));
    assert!(anchor.parameters.parameters.as_object().unwrap().is_empty());
    assert!(anchor.arguments.arguments.is_empty());
    assert_eq!(anchor.range, create_range(0, 1, 1, 49, 1, 50));
    Ok(())
}

#[test]
fn test_try_parse_anchor0_with_parameters_and_args() -> Result<()> {
    let uuid_str = "123e4567-e89b-12d3-a456-426614174000";
    let input = format!("<!-- derive-{}:end {{key: \"value\"}} arg1 arg2 -->\n", uuid_str);
    let mut parser = Parser::new(&input);
    let anchor = _try_parse_anchor0("", &mut parser)?.unwrap();
    assert!(matches!(anchor.command, CommandKind::Derive));
    assert_eq!(anchor.uuid.to_string(), uuid_str);
    assert!(matches!(anchor.kind, AnchorKind::End));
    assert_eq!(anchor.parameters.parameters["key"], json!("value"));
    assert_eq!(anchor.arguments.arguments.len(), 2);
    assert_eq!(anchor.arguments.arguments[0].value, "arg1");
    assert_eq!(anchor.arguments.arguments[1].value, "arg2");
    assert_eq!(anchor.range, create_range(0, 1, 1, 74, 1, 75));
    Ok(())
}

#[test]
fn test_try_parse_text_simple() -> Result<()> {
    let mut parser = Parser::new("Hello world\n");
    let text = _try_parse_text("", &mut parser)?.unwrap();
    assert_eq!(text.range, create_range(0, 1, 1, 12, 1, 13));
    Ok(())
}

#[test]
fn test_try_parse_text_until_tag() -> Result<()> {
    let mut parser = Parser::new("Text before tag.\n@tag\n");
    let text = _try_parse_text("", &mut parser)?.unwrap();
    assert_eq!(text.range, create_range(0, 1, 1, 17, 2, 1));
    assert_eq!(parser.remain(), "@tag\n");
    Ok(())
}

#[test]
fn test_try_parse_text_until_anchor() -> Result<()> {
    let mut parser = Parser::new("Text before anchor.\n<!-- anchor -->\n");
    let text = _try_parse_text("", &mut parser)?.unwrap();
    assert_eq!(text.range, create_range(0, 1, 1, 20, 2, 1));
    assert_eq!(parser.remain(), "<!-- anchor -->\n");
    Ok(())
}

#[test]
fn test_parse_content_mixed() -> Result<()> {
    let uuid_str = "123e4567-e89b-12d3-a456-426614174000";
    let input = format!("Some text.\n@tag arg1\n<!-- answer-{}:begin -->\nMore text.\n<!-- answer-{}:end -->\nFinal text.", uuid_str, uuid_str);
    let mut parser = Parser::new(&input);
    let content = parse_content("", &mut parser)?;

    assert_eq!(content.len(), 6);

    // Text 1
    assert!(matches!(content[0], Content::Text(_)));
    if let Content::Text(text) = &content[0] {
        assert_eq!(text.range, create_range(0, 1, 1, 11, 2, 1));
    }

    // Tag
    assert!(matches!(content[1], Content::Tag(_)));
    if let Content::Tag(tag) = &content[1] {
        assert!(matches!(tag.command, CommandKind::Tag));
        assert_eq!(tag.arguments.arguments[0].value, "arg1");
        assert_eq!(tag.range, create_range(11, 2, 1, 22, 2, 12));
    }

    // Anchor Begin
    assert!(matches!(content[2], Content::Anchor(_)));
    if let Content::Anchor(anchor) = &content[2] {
        assert!(matches!(anchor.command, CommandKind::Answer));
        assert!(matches!(anchor.kind, AnchorKind::Begin));
        assert_eq!(anchor.uuid.to_string(), uuid_str);
        assert_eq!(anchor.range, create_range(22, 2, 12, 71, 3, 1));
    }

    // Text 2
    assert!(matches!(content[3], Content::Text(_)));
    if let Content::Text(text) = &content[3] {
        assert_eq!(text.range, create_range(71, 3, 1, 82, 4, 1));
    }

    // Anchor End
    assert!(matches!(content[4], Content::Anchor(_)));
    if let Content::Anchor(anchor) = &content[4] {
        assert!(matches!(anchor.command, CommandKind::Answer));
        assert!(matches!(anchor.kind, AnchorKind::End));
        assert_eq!(anchor.uuid.to_string(), uuid_str);
        assert_eq!(anchor.range, create_range(82, 4, 1, 131, 5, 1));
    }

    // Text 3
    assert!(matches!(content[5], Content::Text(_)));
    if let Content::Text(text) = &content[5] {
        assert_eq!(text.range, create_range(131, 5, 1, 142, 5, 12));
    }

    Ok(())
}

#[test]
fn test_parse_document_simple() -> Result<()> {
    let document_str = "Hello world";
    let document = parse_document(document_str)?;
    assert_eq!(document.content.len(), 1);
    assert_eq!(document.range, create_range(0, 1, 1, 11, 1, 12));
    Ok(())
}

#[test]
fn test_parse_document_empty() -> Result<()> {
    let document_str = "";
    let document = parse_document(document_str)?;
    assert!(document.content.is_empty());
    assert_eq!(document.range, create_range(0, 1, 1, 0, 1, 1));
    Ok(())
}

#[test]
fn test_parse_document_only_whitespace() -> Result<()> {
    let document_str = "   \n\t ";
    let document = parse_document(document_str)?;
    assert!(document.content.is_empty());
    assert_eq!(document.range, create_range(0, 1, 1, 6, 2, 3));
    Ok(())
}

#[test]
fn test_parse_document_error() -> Result<()> {
    let document_str = "@invalid-tag";
    let result = parse_document(document_str);
    assert!(matches!(result, Err(e) if e.downcast_ref::<ParsingError>().unwrap().to_string().contains("Expected command kind after @")));
    Ok(())
}
