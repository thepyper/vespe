use crate::ast2::{Parser, CommandKind, AnchorKind, Ast2Error, Content};
use serde_json::json;
use uuid::Uuid;

#[test]
fn test_parse_document_simple() {
    let doc = "hello world";
    let document = super::super::parse_document(doc).unwrap();
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
    let document = super::super::parse_document(doc).unwrap();
    assert!(document.content.is_empty());
    assert_eq!(document.range.begin.offset, 0);
    assert_eq!(document.range.end.offset, 0);
}

#[test]
fn test_parse_document_with_error() {
    let doc = "@tag [param=] rest"; // Missing parameter value
    let result = super::super::parse_document(doc);
    assert!(matches!(result, Err(Ast2Error::MissingParameterValue { .. })));
}