use crate::ast2::{AnchorKind, Ast2Error, CommandKind, Content, Parser};
use serde_json::json;
use uuid::Uuid;

#[test]
fn test_parse_content_mixed() {
    let uuid_str = "123e4567-e89b-12d3-a456-426614174000";
    let doc = format!(
        "Some text\n@tag [param=1] 'arg1'\n<!-- include-{}:begin -->\nmore text",
        uuid_str
    );
    let parser = Parser::new(&doc);
    let (content_vec, p_next) = super::parse_content(parser).unwrap();

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
        assert_eq!(
            anchor.range.begin.offset,
            "Some text @tag\n[param=1] 'arg1'\n".len()
        );
        assert_eq!(
            anchor.range.end.offset,
            "Some text @tag\n[param=1] 'arg1'\n".len() + anchor_str.len()
        );
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

#[test]
fn test_parse_document_simple() {
    let doc = "hello world";
    let document = super::parse_document(doc).unwrap();
    assert_eq!(document.content.len(), 1);
    if let Content::Text(text) = &document.content[0] {
        assert_eq!(text.range.begin.offset, 0);
        assert_eq!(text.range.end.offset, doc.len());
    } else {
        panic!("Expected Text");
    }
    assert_eq!(document.range.begin.offset, 0);
    assert_eq!(document.range.end.offset, doc.len());
}

#[test]
fn test_parse_document_empty() {
    let doc = "";
    let document = super::parse_document(doc).unwrap();
    assert!(document.content.is_empty());
    assert_eq!(document.range.begin.offset, 0);
    assert_eq!(document.range.end.offset, 0);
}

#[test]
fn test_parse_document_with_error() {
    let doc = "@tag [param=] rest"; // Missing parameter value
    let result = super::parse_document(doc);
    assert!(matches!(
        result,
        Err(Ast2Error::MissingParameterValue { .. })
    ));
}
