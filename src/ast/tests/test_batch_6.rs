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
fn test_parse_anchor_end_with_parameters() {
    let uuid = Uuid::new_v4();
    let document = format!("<!-- answer-{}:end --> {{status: \"ok\"}}", uuid);
    let mut parser = Parser::new(&document);
    let anchor = parse_anchor(&mut parser).unwrap().unwrap();
    assert_eq!(anchor.command, Command::Answer);
    assert_eq!(anchor.uuid, uuid);
    assert_eq!(anchor.kind, Kind::End);
    assert_eq!(anchor.parameters.len(), 1);
    assert_eq!(
        anchor.parameters["status"],
        ParameterValue::String("ok".to_string())
    );
    assert!(anchor.arguments.is_empty());
    assert_eq!(
        anchor.range,
        create_range(0, 1, 1, document.len(), 1, document.len() + 1)
    );
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
