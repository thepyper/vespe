use crate::ast2::model::core::{AnchorKind, CommandKind};
use crate::ast2::parser::Parser;
use crate::ast2::parser::tags_anchors;

#[test]
fn test_try_parse_command_kind_valid() {
    let (kind, p_next) = tags_anchors::_try_parse_command_kind(&parser).unwrap().unwrap();
    assert_eq!(kind, CommandKind::Tag);
    assert_eq!(p_next.remain(), " rest");

    let (kind, p_next) = tags_anchors::_try_parse_command_kind(&parser).unwrap().unwrap();
    assert_eq!(kind, CommandKind::Include);
    assert_eq!(p_next.remain(), " rest");

    let (kind, p_next) = tags_anchors::_try_parse_command_kind(&parser).unwrap().unwrap();
    assert_eq!(kind, CommandKind::Answer);
    assert_eq!(p_next.remain(), " rest");
}

#[test]
fn test_try_parse_command_kind_invalid() {
    let result = tags_anchors::_try_parse_command_kind(&parser).unwrap();
    assert!(result.is_none());
    assert_eq!(parser.remain(), "invalid_command rest");
}

#[test]
fn test_try_parse_anchor_kind_valid() {
    let (kind, p_next) = tags_anchors::_try_parse_anchor_kind(&parser).unwrap().unwrap();
    assert_eq!(kind, AnchorKind::Begin);
    assert_eq!(p_next.remain(), " rest");

    let (kind, p_next) = tags_anchors::_try_parse_anchor_kind(&parser).unwrap().unwrap();
    assert_eq!(kind, AnchorKind::End);
    assert_eq!(p_next.remain(), " rest");
}

#[test]
fn test_try_parse_anchor_kind_invalid() {
    let result = tags_anchors::_try_parse_anchor_kind(&parser).unwrap();
    assert!(result.is_none());
    assert_eq!(parser.remain(), "invalid_anchor rest");
}
