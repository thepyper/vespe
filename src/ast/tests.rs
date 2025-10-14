use super::*;

fn create_pos(offset: usize, line: usize, column: usize) -> Position {
    Position { offset, line, column }
}

fn create_range(start_offset: usize, start_line: usize, start_column: usize, end_offset: usize, end_line: usize, end_column: usize) -> Range {
    Range {
        start: create_pos(start_offset, start_line, start_column),
        end: create_pos(end_offset, end_line, end_column),
    }
}

#[test]
fn test_parser_new() {
    let document = "hello";
    let parser = Parser::new(document);
    assert_eq!(parser.current_pos, create_pos(0, 1, 1));
    assert_eq!(parser.document, "hello");
}

#[test]
fn test_parser_peek_consume_advance() {
    let mut parser = Parser::new("abc\n123");
    assert_eq!(parser.peek(), Some('a'));
    assert_eq!(parser.consume(), Some('a'));
    assert_eq!(parser.current_pos, create_pos(1, 1, 2));

    assert_eq!(parser.peek(), Some('b'));
    assert_eq!(parser.consume(), Some('b'));
    assert_eq!(parser.current_pos, create_pos(2, 1, 3));

    assert_eq!(parser.peek(), Some('c'));
    assert_eq!(parser.consume(), Some('c'));
    assert_eq!(parser.current_pos, create_pos(3, 1, 4));

    assert_eq!(parser.peek(), Some('\n'));
    assert_eq!(parser.consume(), Some('\n'));
    assert_eq!(parser.current_pos, create_pos(4, 2, 1)); // New line

    assert_eq!(parser.peek(), Some('1'));
    assert_eq!(parser.consume(), Some('1'));
    assert_eq!(parser.current_pos, create_pos(5, 2, 2));

    parser.advance_position_by_str("23");
    assert_eq!(parser.current_pos, create_pos(7, 2, 4));

    assert_eq!(parser.peek(), None);
    assert_eq!(parser.consume(), None);
}

#[test]
fn test_parser_take_while() {
    let mut parser = Parser::new("  hello world");
    parser.skip_whitespace();
    assert_eq!(parser.current_pos, create_pos(2, 1, 3));

    let word = parser.take_while(|c| c.is_alphabetic());
    assert_eq!(word, "hello");
    assert_eq!(parser.current_pos, create_pos(7, 1, 8));
}

#[test]
fn test_parser_parse_quoted_string_double_quotes() {
    let mut parser = Parser::new("\"hello world\"");
    let (s, r) = parser.parse_quoted_string('"').unwrap();
    assert_eq!(s, "hello world");
    assert_eq!(r, create_range(0, 1, 1, 13, 1, 14));
}

#[test]
fn test_parser_parse_quoted_string_single_quotes() {
    let mut parser = Parser::new("'hello world'");
    let (s, r) = parser.parse_quoted_string('\'').unwrap();
    assert_eq!(s, "hello world");
    assert_eq!(r, create_range(0, 1, 1, 13, 1, 14));
}

#[test]
fn test_parser_parse_quoted_string_with_escapes() {
    let mut parser = Parser::new("\"hello\\nworld\"");
    let (s, r) = parser.parse_quoted_string('"').unwrap();
    assert_eq!(s, "hello\nworld");
    assert_eq!(r, create_range(0, 1, 1, 14, 1, 15));
}

#[test]
fn test_parser_parse_quoted_string_unterminated() {
    let mut parser = Parser::new("\"hello world");
    let err = parser.parse_quoted_string('"').unwrap_err();
    assert_eq!(err, ParsingError::UnterminatedString { range: create_range(0, 1, 1, 12, 1, 13) });
}

#[test]
fn test_parser_parse_unquoted_identifier() {
    let mut parser = Parser::new("my_identifier123 rest");
    let (s, r) = parser.parse_unquoted_identifier().unwrap();
    assert_eq!(s, "my_identifier123");
    assert_eq!(r, create_range(0, 1, 1, 16, 1, 17));
    assert_eq!(parser.remaining_slice(), " rest");
}

#[test]
fn test_parser_parse_number_integer() {
    let mut parser = Parser::new("12345 rest");
    let (val, r) = parser.parse_number().unwrap().unwrap();
    assert_eq!(val, ParameterValue::Integer(12345));
    assert_eq!(r, create_range(0, 1, 1, 5, 1, 6));
    assert_eq!(parser.remaining_slice(), " rest");
}

#[test]
fn test_parser_parse_number_float() {
    let mut parser = Parser::new("123.45 rest");
    let (val, r) = parser.parse_number().unwrap().unwrap();
    assert_eq!(val, ParameterValue::Float(123.45));
    assert_eq!(r, create_range(0, 1, 1, 6, 1, 7));
    assert_eq!(parser.remaining_slice(), " rest");
}

#[test]
fn test_parser_parse_number_negative() {
    let mut parser = Parser::new("-123 rest");
    let (val, r) = parser.parse_number().unwrap().unwrap();
    assert_eq!(val, ParameterValue::Integer(-123));
    assert_eq!(r, create_range(0, 1, 1, 4, 1, 5));
    assert_eq!(parser.remaining_slice(), " rest");
}

#[test]
fn test_parser_parse_boolean_true() {
    let mut parser = Parser::new("true rest");
    let (val, r) = parser.parse_boolean().unwrap();
    assert_eq!(val, ParameterValue::Boolean(true));
    assert_eq!(r, create_range(0, 1, 1, 4, 1, 5));
    assert_eq!(parser.remaining_slice(), " rest");
}

#[test]
fn test_parser_parse_boolean_false() {
    let mut parser = Parser::new("false rest");
    let (val, r) = parser.parse_boolean().unwrap();
    assert_eq!(val, ParameterValue::Boolean(false));
    assert_eq!(r, create_range(0, 1, 1, 5, 1, 6));
    assert_eq!(parser.remaining_slice(), " rest");
}

#[test]
fn test_parse_parameters_empty() {
    let mut parser = Parser::new("{}");
    let (params, r) = parse_parameters(&mut parser).unwrap();
    assert!(params.is_empty());
    assert_eq!(r, create_range(0, 1, 1, 2, 1, 3));
}

#[test]
fn test_parse_parameters_single_unquoted() {
    let mut parser = Parser::new("{key: value}");
    let (params, r) = parse_parameters(&mut parser).unwrap();
    assert_eq!(params.len(), 1);
    assert_eq!(params["key"], ParameterValue::String("value".to_string()));
    assert_eq!(r, create_range(0, 1, 1, 12, 1, 13));
}

#[test]
fn test_parse_parameters_multiple_quoted() {
    let mut parser = Parser::new("{\"key1\": \"value1\", 'key2': 123, key3: true}");
    let (params, r) = parse_parameters(&mut parser).unwrap();
    assert_eq!(params.len(), 3);
    assert_eq!(params["key1"], ParameterValue::String("value1".to_string()));
    assert_eq!(params["key2"], ParameterValue::Integer(123));
    assert_eq!(params["key3"], ParameterValue::Boolean(true));
    assert_eq!(r, create_range(0, 1, 1, 43, 1, 44));
}

#[test]
fn test_parse_parameters_invalid_syntax() {
    let mut parser = Parser::new("{key: }");
    let err = parse_parameters(&mut parser).unwrap_err();
    assert!(matches!(err, ParsingError::InvalidSyntax { .. }));
}

#[test]
fn test_parse_argument_quoted() {
    let mut parser = Parser::new("\"arg1\" rest");
    let (arg, r) = parse_argument(&mut parser).unwrap().unwrap();
    assert_eq!(arg, "arg1");
    assert_eq!(r, create_range(0, 1, 1, 6, 1, 7));
    assert_eq!(parser.remaining_slice(), " rest");
}

#[test]
fn test_parse_argument_unquoted() {
    let mut parser = Parser::new("arg1 rest");
    let (arg, r) = parse_argument(&mut parser).unwrap().unwrap();
    assert_eq!(arg, "arg1");
    assert_eq!(r, create_range(0, 1, 1, 4, 1, 5));
    assert_eq!(parser.remaining_slice(), " rest");
}

#[test]
fn test_parse_arguments_multiple() {
    let mut parser = Parser::new("arg1 \"arg2 with spaces\" arg3");
    let (args, r) = parse_arguments(&mut parser).unwrap();
    assert_eq!(args, vec!["arg1", "arg2 with spaces", "arg3"]);
    assert_eq!(r, create_range(0, 1, 1, 28, 1, 29));
}

#[test]
fn test_parse_arguments_empty() {
    let mut parser = Parser::new("");
    let (args, r) = parse_arguments(&mut parser).unwrap();
    assert!(args.is_empty());
    assert_eq!(r, create_range(0, 1, 1, 0, 1, 1));
}

#[test]
fn test_parse_tag_simple() {
    let mut parser = Parser::new("@include arg1 arg2");
    let tag = parse_tag(&mut parser).unwrap().unwrap();
    assert_eq!(tag.command, Command::Include);
    assert!(tag.parameters.is_empty());
    assert_eq!(tag.arguments, vec!["arg1", "arg2"]);
    assert_eq!(tag.range, create_range(0, 1, 1, 18, 1, 19));
}

#[test]
fn test_parse_tag_with_parameters() {
    let mut parser = Parser::new("@answer {key: \"value\"} arg1");
    let tag = parse_tag(&mut parser).unwrap().unwrap();
    assert_eq!(tag.command, Command::Answer);
    assert_eq!(tag.parameters.len(), 1);
    assert_eq!(tag.parameters["key"], ParameterValue::String("value".to_string()));
    assert_eq!(tag.arguments, vec!["arg1"]);
    assert_eq!(tag.range, create_range(0, 1, 1, 27, 1, 28));
}

#[test]
fn test_parse_tag_not_at_column_1() {
    let mut parser = Parser::new(" @include arg1");
    let tag = parse_tag(&mut parser).unwrap();
    assert!(tag.is_none());
}

#[test]
fn test_parse_anchor_begin() {
    let uuid = Uuid::new_v4();
    let document = format!("<!-- include-{}:begin --> arg1", uuid);
    let mut parser = Parser::new(&document);
    let anchor = parse_anchor(&mut parser).unwrap().unwrap();
    assert_eq!(anchor.command, Command::Include);
    assert_eq!(anchor.uuid, uuid);
    assert_eq!(anchor.kind, Kind::Begin);
    assert!(anchor.parameters.is_empty());
    assert_eq!(anchor.arguments, vec!["arg1"]);
    assert_eq!(anchor.range, create_range(0, 1, 1, document.len(), 1, document.len() + 1));
}

#[test]
fn test_parse_anchor_end_with_parameters() {
    let uuid = Uuid::new_v4();
    let document = format!("<!-- answer-{}:end --> {{status: \"ok\"}}", uuid);
    let mut parser = Parser::new(&document);
    let anchor = parse_anchor(&mut parser).unwrap().unwrap();
    assert_eq!(anchor.command, Command::Answer);
    assert_eq!(anchor.uuid, uuid);
    assert_eq!(anchor.kind, Kind::End);
    assert_eq!(anchor.parameters.len(), 1);
    assert_eq!(anchor.parameters["status"], ParameterValue::String("ok".to_string()));
    assert!(anchor.arguments.is_empty());
    assert_eq!(anchor.range, create_range(0, 1, 1, document.len(), 1, document.len() + 1));
}

#[test]
fn test_parse_anchor_unterminated() {
    let uuid = Uuid::new_v4();
    let document = format!("<!-- include-{}:begin", uuid);
    let mut parser = Parser::new(&document);
    let err = parse_anchor(&mut parser).unwrap_err();
    assert!(matches!(err, ParsingError::UnterminatedString { .. }));
}

#[test]
fn test_parse_text_simple() {
    let mut parser = Parser::new("This is some text.");
    let text = parse_text(&mut parser).unwrap().unwrap();
    assert_eq!(text.content, "This is some text.");
    assert_eq!(text.range, create_range(0, 1, 1, 18, 1, 19));
}

#[test]
fn test_parse_text_until_tag() {
    let mut parser = Parser::new("Text before tag.\n@include arg");
    let text = parse_text(&mut parser).unwrap().unwrap();
    assert_eq!(text.content, "Text before tag.\n");
    assert_eq!(text.range, create_range(0, 1, 1, 17, 2, 1));
    assert_eq!(parser.remaining_slice(), "@include arg");
}

#[test]
fn test_parse_text_until_anchor() {
    let uuid = Uuid::new_v4();
    let document = format!("Text before anchor.\n<!-- include-{}:begin -->", uuid);
    let mut parser = Parser::new(&document);
    let text = parse_text(&mut parser).unwrap().unwrap();
    assert_eq!(text.content, "Text before anchor.\n");
    assert_eq!(text.range, create_range(0, 1, 1, 20, 2, 1));
    assert!(parser.remaining_slice().starts_with("<!--"));
}

#[test]
fn test_parse_node_tag() {
    let mut parser = Parser::new("@include arg");
    let node = parse_node(&mut parser).unwrap().unwrap();
    assert!(matches!(node, Node::Tag(_)));
}

#[test]
fn test_parse_node_anchor() {
    let uuid = Uuid::new_v4();
    let document = format!("<!-- include-{}:begin -->", uuid);
    let mut parser = Parser::new(&document);
    let node = parse_node(&mut parser).unwrap().unwrap();
    assert!(matches!(node, Node::Anchor(_)));
}

#[test]
fn test_parse_node_text() {
    let mut parser = Parser::new("Just some text.");
    let node = parse_node(&mut parser).unwrap().unwrap();
    assert!(matches!(node, Node::Text(_)));
}

#[test]
fn test_parse_mixed_content() {
    let uuid1 = Uuid::new_v4();
    let uuid2 = Uuid::new_v4();
    let document = format!("Some initial text.\n@include file.md arg1\n<!-- derive-{}:begin -->\nMore text here.\n<!-- derive-{}:end -->\nFinal text.", uuid1, uuid2);

    let root = parse(&document).unwrap();
    assert_eq!(root.children.len(), 6);

    // Node 1: Text
    if let Node::Text(text) = &root.children[0] {
        assert_eq!(text.content, "Some initial text.\n");
    } else {
        panic!("Expected Text node");
    }

    // Node 2: Tag
    if let Node::Tag(tag) = &root.children[1] {
        assert_eq!(tag.command, Command::Include);
        assert_eq!(tag.arguments, vec!["file.md", "arg1"]);
    } else {
        panic!("Expected Tag node");
    }

    // Node 3: Anchor (begin)
    if let Node::Anchor(anchor) = &root.children[2] {
        assert_eq!(anchor.command, Command::Derive);
        assert_eq!(anchor.uuid, uuid1);
        assert_eq!(anchor.kind, Kind::Begin);
    } else {
        panic!("Expected Anchor node");
    }

    // Node 4: Text
    if let Node::Text(text) = &root.children[3] {
        assert_eq!(text.content, "More text here.\n");
    } else {
        panic!("Expected Text node");
    }

    // Node 5: Anchor (end)
    if let Node::Anchor(anchor) = &root.children[4] {
        assert_eq!(anchor.command, Command::Derive);
        assert_eq!(anchor.uuid, uuid2);
        assert_eq!(anchor.kind, Kind::End);
    } else {
        panic!("Expected Anchor node");
    }
}

#[test]
fn test_parse_empty_document() {
    let document = "";
    let root = parse(document).unwrap();
    assert!(root.children.is_empty());
    assert_eq!(root.range, create_range(0, 1, 1, 0, 1, 1));
}

#[test]
fn test_parse_only_whitespace() {
    let document = "   \n\n";
    let root = parse(document).unwrap();
    assert!(root.children.is_empty());}
