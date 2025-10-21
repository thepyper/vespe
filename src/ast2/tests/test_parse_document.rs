use crate::ast2::{parse_document, Content, Text, Anchor, Argument, Arguments};
use uuid::Uuid;

#[test]
fn test_parse_document_empty() {
    let input = "";
    let document = parse_document(input).unwrap();
    assert_eq!(document.content.len(), 0);
}

#[test]
fn test_parse_document_with_text() {
    let input = "Hello, world!";
    let document = parse_document(input).unwrap();
    assert_eq!(document.content.len(), 1);
    assert!(matches!(document.content[0], Content::Text(_)));
}

#[test]
fn test_parse_document_with_newline() {
    let input = "Hello,\nworld!";
    let document = parse_document(input).unwrap();
    assert_eq!(document.content.len(), 1);
    assert!(matches!(document.content[0], Content::Text(_)));
}

#[test]
fn test_parse_document_with_anchor() {
    let uuid = Uuid::new_v4();
    let input = format!("#[{{}}మని", uuid);
    let document = parse_document(&input).unwrap();
    assert_eq!(document.content.len(), 1);
    assert!(matches!(document.content[0], Content::Anchor(_)));
}

#[test]
fn test_parse_document_multiple_nodes() {
    let uuid1 = Uuid::new_v4();
    let uuid2 = Uuid::new_v4();
    let input = format!("Text1\n#[{{}}]\nText2\n#[{{}}]", uuid1, uuid2);
    let document = parse_document(&input).unwrap();
    assert_eq!(document.content.len(), 4);
    assert!(matches!(document.content[0], Content::Text(_)));
    assert!(matches!(document.content[1], Content::Anchor(_)));
    assert!(matches!(document.content[2], Content::Text(_)));
    assert!(matches!(document.content[3], Content::Anchor(_)));
}

#[test]
fn test_parse_document_with_mixed_content() {
    let uuid = Uuid::new_v4();
    let input = format!("Some text #[{{}}] and more text.", uuid);
    let document = parse_document(&input).unwrap();
    assert_eq!(document.content.len(), 3);
    assert!(matches!(document.content[0], Content::Text(_)));
    assert!(matches!(document.content[1], Content::Anchor(_)));
    assert!(matches!(document.content[2], Content::Text(_)));
}

#[test]
fn test_parse_document_with_tag() {
    let input = "@tag";
    let document = parse_document(input).unwrap();
    assert_eq!(document.content.len(), 1);
    assert!(matches!(document.content[0], Content::Tag(_)));
}

#[test]
fn test_parse_document_with_tag_and_text() {
    let input = "@tag Some text";
    let document = parse_document(input).unwrap();
    assert_eq!(document.content.len(), 2);
    assert!(matches!(document.content[0], Content::Tag(_)));
    assert!(matches!(document.content[1], Content::Text(_)));
}

#[test]
fn test_parse_document_with_tag_and_anchor() {
    let uuid = Uuid::new_v4();
    let input = format!("@tag #[{{}}]", uuid);
    let document = parse_document(&input).unwrap();
    assert_eq!(document.content.len(), 2);
    assert!(matches!(document.content[0], Content::Tag(_)));
    assert!(matches!(document.content[1], Content::Anchor(_)));
}

#[test]
fn test_parse_document_with_multiple_tags() {
    let input = "@tag1 @tag2 Some text";
    let document = parse_document(input).unwrap();
    assert_eq!(document.content.len(), 3);
    assert!(matches!(document.content[0], Content::Tag(_)));
    assert!(matches!(document.content[1], Content::Tag(_)));
    assert!(matches!(document.content[2], Content::Text(_)));
}

#[test]
fn test_parse_document_with_multiple_anchors() {
    let uuid1 = Uuid::new_v4();
    let uuid2 = Uuid::new_v4();
    let input = format!("#[{{}}] #[{{}}] Some text", uuid1, uuid2);
    let document = parse_document(&input).unwrap();
    assert_eq!(document.content.len(), 3);
    assert!(matches!(document.content[0], Content::Anchor(_)));
    assert!(matches!(document.content[1], Content::Anchor(_)));
    assert!(matches!(document.content[2], Content::Text(_)));
}

#[test]
fn test_parse_document_with_tag_and_arguments() {
    let input = "@tag(arg1=value1, arg2=\"value 2\")";
    let document = parse_document(input).unwrap();
    assert_eq!(document.content.len(), 1);
    assert!(matches!(document.content[0], Content::Tag(_)));
    if let Content::Tag(tag) = &document.content[0] {
        assert_eq!(tag.name, "tag");
        assert!(tag.arguments.is_some());
        let args = tag.arguments.as_ref().unwrap();
        assert_eq!(args.len(), 2);
        assert_eq!(args[0].name, "arg1");
        assert_eq!(args[0].value, "value1");
        assert_eq!(args[1].name, "arg2");
        assert_eq!(args[1].value, "value 2");
    }
}

#[test]
fn test_parse_document_with_anchor_and_arguments() {
    let uuid = Uuid::new_v4();
    let input = format!("#[{{}}(arg1=value1, arg2=\"value 2\")]", uuid);
    let document = parse_document(&input).unwrap();
    assert_eq!(document.content.len(), 1);
    assert!(matches!(document.content[0], Content::Anchor(_)));
    if let Content::Anchor(anchor) = &document.content[0] {
        assert_eq!(anchor.uuid, uuid);
        assert!(anchor.arguments.is_some());
        let args = anchor.arguments.as_ref().unwrap();
        assert_eq!(args.len(), 2);
        assert_eq!(args[0].name, "arg1");
        assert_eq!(args[0].value, "value1");
        assert_eq!(args[1].name, "arg2");
        assert_eq!(args[1].value, "value 2");
    }
}

#[test]
fn test_is_begin_of_line() {
    assert!(is_begin_of_line("test", 0));
    assert!(!is_begin_of_line(" test", 1));
    assert!(is_begin_of_line("test\nnew_line", 5));
    assert!(!is_begin_of_line("test\n new_line", 6));
}

#[test]
fn test_consume_matching_char_immutable() {
    let input = "abc";
    let (remaining, consumed) = consume_matching_char_immutable('a', input).unwrap();
    assert_eq!(remaining, "bc");
    assert_eq!(consumed, 'a');

    let input = "abc";
    let result = consume_matching_char_immutable('b', input);
    assert!(result.is_none());

    let input = "";
    let result = consume_matching_char_immutable('a', input);
    assert!(result.is_none());
}