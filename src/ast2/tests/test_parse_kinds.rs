use super::super::{AnchorKind, CommandKind};
use super::anchor::_try_parse_anchor_kind;
use super::command_kind::_try_parse_command_kind;
use super::parser::Parser;

#[test]
fn test_try_parse_command_kind_valid() {
    let doc = "tag rest";
    let parser = Parser::new(doc);
    let (kind, p_next) = _try_parse_command_kind(&parser).unwrap().unwrap();
    assert_eq!(kind, CommandKind::Tag);
    assert_eq!(p_next.remain(), " rest");

    let doc = "include rest";
    let parser = Parser::new(doc);
    let (kind, p_next) = _try_parse_command_kind(&parser).unwrap().unwrap();
    assert_eq!(kind, CommandKind::Include);
    assert_eq!(p_next.remain(), " rest");

    let doc = "answer rest";
    let parser = Parser::new(doc);
    let (kind, p_next) = _try_parse_command_kind(&parser).unwrap().unwrap();
    assert_eq!(kind, CommandKind::Answer);
    assert_eq!(p_next.remain(), " rest");
}

#[test]
fn test_try_parse_command_kind_invalid() {
    let doc = "invalid_command rest";
    let parser = Parser::new(doc);
    let result = _try_parse_command_kind(&parser).unwrap();
    assert!(result.is_none());
    assert_eq!(parser.remain(), "invalid_command rest");
}

#[test]
fn test_try_parse_anchor_kind_valid() {
    let doc = "begin rest";
    let parser = Parser::new(doc);
    let (kind, p_next) = _try_parse_anchor_kind(&parser).unwrap().unwrap();
    assert_eq!(kind, AnchorKind::Begin);
    assert_eq!(p_next.remain(), " rest");

    let doc = "end rest";
    let parser = Parser::new(doc);
    let (kind, p_next) = _try_parse_anchor_kind(&parser).unwrap().unwrap();
    assert_eq!(kind, AnchorKind::End);
    assert_eq!(p_next.remain(), " rest");
}

#[test]
fn test_try_parse_anchor_kind_invalid() {
    let doc = "invalid_anchor rest";
    let parser = Parser::new(doc);
    let result = _try_parse_anchor_kind(&parser).unwrap();
    assert!(result.is_none());
    assert_eq!(parser.remain(), "invalid_anchor rest");
}
