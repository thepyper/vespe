use crate::ast::*;
use uuid::Uuid;

fn create_pos(offset: usize, line: usize, column: usize) -> Position {
    Position {
        offset,
        line,
        column,
    }
}

fn create_range(
    start_offset: usize,
    start_line: usize,
    start_column: usize,
    end_offset: usize,
    end_line: usize,
    end_column: usize,
) -> Range {
    Range {
        start: create_pos(start_offset, start_line, start_column),
        end: create_pos(end_offset, end_line, end_column),
    }
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
    assert_eq!(
        tag.parameters["key"],
        ParameterValue::String("value".to_string())
    );
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
    assert_eq!(
        anchor.range,
        create_range(0, 1, 1, document.len(), 1, document.len() + 1)
    );
}
