use super::super::{CommandKind};
use super::parser::Parser;
use super::tag::_try_parse_tag;

#[test]
fn test_try_parse_tag_simple() {
    let doc = "@tag ";
    let parser = Parser::new(doc);
    let (tag, p_next) = _try_parse_tag(&parser).unwrap().unwrap();
    assert_eq!(tag.command, CommandKind::Tag);
    assert!(tag.parameters.parameters.properties.is_empty());
    assert!(tag.arguments.arguments.is_empty());
    assert_eq!(p_next.remain(), "");

    let tag_str = "@tag ";
    assert_eq!(tag.range.begin.offset, 0);
    assert_eq!(tag.range.end.offset, tag_str.len());
}

#[test]
fn test_try_parse_tag_with_parameters() {
    let doc = "@include {file=\"path/to/file.txt\"} ";
    let parser = Parser::new(doc);
    let (tag, p_next) = _try_parse_tag(&parser).unwrap().unwrap();
    assert_eq!(tag.command, CommandKind::Include);
    assert_eq!(tag.parameters.parameters.properties.len(), 1);
    //assert_eq!(tag.parameters.parameters["file"], json!("path/to/file.txt"));
    assert!(tag.arguments.arguments.is_empty());
    assert_eq!(p_next.remain(), "");

    let tag_str = "@include {file=\"path/to/file.txt\"} ";
    assert_eq!(tag.range.begin.offset, 0);
    assert_eq!(tag.range.end.offset, tag_str.len());
}

#[test]
fn test_try_parse_tag_with_arguments() {
    let doc = "@inline 'arg1' \"arg2\" ";
    let parser = Parser::new(doc);
    let (tag, p_next) = _try_parse_tag(&parser).unwrap().unwrap();
    assert_eq!(tag.command, CommandKind::Inline);
    assert!(tag.parameters.parameters.properties.is_empty());
    assert_eq!(tag.arguments.arguments.len(), 2);
    assert_eq!(tag.arguments.arguments[0].value, "arg1");
    assert_eq!(tag.arguments.arguments[1].value, "arg2");
    assert_eq!(p_next.remain(), "");

    let tag_str = "@inline 'arg1' \"arg2\" ";
    assert_eq!(tag.range.begin.offset, 0);
    assert_eq!(tag.range.end.offset, tag_str.len());
}

#[test]
fn test_try_parse_tag_with_parameters_and_arguments() {
    let doc = "@answer {id:123} 'arg1' ";
    let parser = Parser::new(doc);
    let (tag, p_next) = _try_parse_tag(&parser).unwrap().unwrap();
    assert_eq!(tag.command, CommandKind::Answer);
    assert_eq!(tag.parameters.parameters.properties.len(), 1);
    // TODO assert_eq!(tag.parameters.parameters["id"], json!(123));
    assert_eq!(tag.arguments.arguments.len(), 1);
    assert_eq!(tag.arguments.arguments[0].value, "arg1");
    assert_eq!(p_next.remain(), "");

    let tag_str = "@answer {id:123} 'arg1' ";
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
    let doc = "@tag\nrest";
    let parser = Parser::new(doc);
    let (tag, p_next) = _try_parse_tag(&parser).unwrap().unwrap();
    assert_eq!(tag.command, CommandKind::Tag);
    assert_eq!(p_next.remain(), "rest");
    assert_eq!(p_next.get_position().line, 2);
    assert_eq!(p_next.get_position().column, 1);
}
