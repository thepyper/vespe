use crate::ast2::model::core::{AnchorKind, CommandKind};
use crate::ast2::error::Ast2Error;
use crate::ast2::parser::Parser;
use crate::ast2::parser::tags_anchors;

use uuid::Uuid;

#[test]
fn test_try_parse_anchor_simple() {
    let uuid_str = "123e4567-e89b-12d3-a456-426614174000";
    let doc = format!("<!-- tag-{}:begin --> rest", uuid_str);
    let parser = Parser::new(&doc);
    let (anchor, p_next) = tags_anchors::_try_parse_anchor(&parser).unwrap().unwrap();
    assert_eq!(anchor.command, CommandKind::Tag);
    assert_eq!(anchor.uuid, Uuid::parse_str(uuid_str).unwrap());
    assert_eq!(anchor.kind, AnchorKind::Begin);
    assert!(anchor.parameters.parameters.properties.is_empty());
    assert!(anchor.arguments.arguments.is_empty());
    assert_eq!(p_next.remain(), "rest");

    let anchor_full_str = format!("<!-- tag-{}:begin --> ", uuid_str);
    assert_eq!(anchor.range.begin.offset, 0);
    assert_eq!(anchor.range.end.offset, anchor_full_str.len());
}

#[test]
fn test_try_parse_anchor_with_parameters() {
    let uuid_str = "123e4567-e89b-12d3-a456-426614174000";
    let doc = format!(
        "<!-- include-{}:end {{file=\"path.txt\"}} --> rest",
        uuid_str
    );
    let parser = Parser::new(&doc);
    let (anchor, p_next) = tags_anchors::_try_parse_anchor(&parser).unwrap().unwrap();
    assert_eq!(anchor.command, CommandKind::Include);
    assert_eq!(anchor.uuid, Uuid::parse_str(uuid_str).unwrap());
    assert_eq!(anchor.kind, AnchorKind::End);
    assert_eq!(anchor.parameters.parameters.properties.len(), 1);
    // TODO assert_eq!(anchor.parameters.parameters["file"], json!("path.txt"));
    assert!(anchor.arguments.arguments.is_empty());
    assert_eq!(p_next.remain(), "rest");

    let anchor_full_str = format!("<!-- include-{}:end {{file=\"path.txt\"}} --> ", uuid_str);
    assert_eq!(anchor.range.begin.offset, 0);
    assert_eq!(anchor.range.end.offset, anchor_full_str.len());
}

#[test]
fn test_try_parse_anchor_missing_closing_tag() {
    let uuid_str = "123e4567-e89b-12d3-a456-426614174000";
    let doc = format!("<!-- tag-{}:begin rest", uuid_str);
    let parser = Parser::new(&doc);
    let result = tags_anchors::_try_parse_anchor(&parser);
    assert!(matches!(result, Err(Ast2Error::UnclosedString { .. })));
}

#[test]
fn test_try_parse_anchor_invalid_uuid() {
    let doc = "<!-- tag-invalid-uuid:begin --> rest";
    let parser = Parser::new(doc);
    let result = tags_anchors::_try_parse_anchor(&parser);
    assert!(matches!(result, Err(Ast2Error::InvalidUuid { .. })));
}

#[test]
fn test_try_parse_anchor_no_opening_comment() {
    let doc = "tag-uuid:begin --> rest";
    let parser = Parser::new(doc);
    let result = tags_anchors::_try_parse_anchor(&parser).unwrap();
    assert!(result.is_none());
}
