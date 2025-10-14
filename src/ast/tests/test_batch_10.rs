use crate::ast::*;
use uuid::Uuid;

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
fn test_parse_anchor_simple_new() {
    let uuid = Uuid::parse_str("12341234-1234-1234-1234-123412341234").unwrap();
    let document = format!("Before.\n<!-- tag-{}:begin -->\nAfter.", uuid);
    let root = parse(document.as_str()).unwrap();
    assert_eq!(root.children.len(), 3);

    if let Node::Text(text) = &root.children[0] {
        assert_eq!(text.content, "Before.\n");
    } else {
        panic!("Expected Text node");
    }

    if let Node::Anchor(anchor) = &root.children[1] {
        assert_eq!(anchor.command, Command::Tag);
        assert_eq!(anchor.uuid, uuid);
        assert_eq!(anchor.kind, Kind::Begin);
        assert!(anchor.parameters.is_empty());
        assert!(anchor.arguments.is_empty());
        assert_eq!(anchor.range, create_range(9, 2, 1, 54, 2, 46));
    } else {
        panic!("Expected Anchor node");
    }

    if let Node::Text(text) = &root.children[2] {
        assert_eq!(text.content, "After.");
    } else {
        panic!("Expected Text node");
    }
}

#[test]
fn test_parse_anchor_empty_params_new() {
    let uuid = Uuid::parse_str("12341234-1234-1234-1234-123412341234").unwrap();
    let document = format!("Before.\n<!-- tag-{}:begin {{ }} -->\nAfter.", uuid);
    let root = parse(document.as_str()).unwrap();
    assert_eq!(root.children.len(), 3);

    if let Node::Text(text) = &root.children[0] {
        assert_eq!(text.content, "Before.\n");
    } else {
        panic!("Expected Text node");
    }

    if let Node::Anchor(anchor) = &root.children[1] {
        assert_eq!(anchor.command, Command::Tag);
        assert_eq!(anchor.uuid, uuid);
        assert_eq!(anchor.kind, Kind::Begin);
        assert!(anchor.parameters.is_empty());
        assert!(anchor.arguments.is_empty());
        assert_eq!(anchor.range, create_range(9, 2, 1, 57, 2, 49));
    } else {
        panic!("Expected Anchor node");
    }

    if let Node::Text(text) = &root.children[2] {
        assert_eq!(text.content, "After.");
    } else {
        panic!("Expected Text node");
    }
}

#[test]
fn test_parse_anchor_empty_params_multiline_new() {
    let uuid = Uuid::parse_str("12341234-1234-1234-1234-123412341234").unwrap();
    let document = format!("Before.\n<!-- tag-{}:begin {{\n }} -->\nAfter.", uuid);
    let root = parse(document.as_str()).unwrap();
    assert_eq!(root.children.len(), 3);

    if let Node::Text(text) = &root.children[0] {
        assert_eq!(text.content, "Before.\n");
    } else {
        panic!("Expected Text node");
    }

    if let Node::Anchor(anchor) = &root.children[1] {
        assert_eq!(anchor.command, Command::Tag);
        assert_eq!(anchor.uuid, uuid);
        assert_eq!(anchor.kind, Kind::Begin);
        assert!(anchor.parameters.is_empty());
        assert!(anchor.arguments.is_empty());
        assert_eq!(anchor.range, create_range(9, 2, 1, 60, 3, 5));
    } else {
        panic!("Expected Anchor node");
    }

    if let Node::Text(text) = &root.children[2] {
        assert_eq!(text.content, "After.");
    } else {
        panic!("Expected Text node");
    }
}

#[test]
fn test_parse_anchor_empty_params_multiline_args_new() {
    let uuid = Uuid::parse_str("12341234-1234-1234-1234-123412341234").unwrap();
    let document = format!("Before.\n<!-- tag-{}:begin {{\n }} arg1 arg2 arg3 -->\nAfter.", uuid);
    let root = parse(document.as_str()).unwrap();
    assert_eq!(root.children.len(), 3);

    if let Node::Text(text) = &root.children[0] {
        assert_eq!(text.content, "Before.\n");
    } else {
        panic!("Expected Text node");
    }

    if let Node::Anchor(anchor) = &root.children[1] {
        assert_eq!(anchor.command, Command::Tag);
        assert_eq!(anchor.uuid, uuid);
        assert_eq!(anchor.kind, Kind::Begin);
        assert!(anchor.parameters.is_empty());
        assert_eq!(anchor.arguments, vec!["arg1", "arg2", "arg3"]);
        assert_eq!(anchor.range, create_range(9, 2, 1, 76, 3, 21));
    } else {
        panic!("Expected Anchor node");
    }

    if let Node::Text(text) = &root.children[2] {
        assert_eq!(text.content, "After.");
    } else {
        panic!("Expected Text node");
    }
}

#[test]
fn test_parse_anchor_empty_params_multiline_args_spaced_new() {
    let uuid = Uuid::parse_str("12341234-1234-1234-1234-123412341234").unwrap();
    let document = format!("Before.\n<!-- tag-{}:begin \n{{\n }} arg1 arg2 arg3 \n-->\nAfter.", uuid);
    let root = parse(document.as_str()).unwrap();
    assert_eq!(root.children.len(), 3);

    if let Node::Text(text) = &root.children[0] {
        assert_eq!(text.content, "Before.\n");
    } else {
        panic!("Expected Text node");
    }

    if let Node::Anchor(anchor) = &root.children[1] {
        assert_eq!(anchor.command, Command::Tag);
        assert_eq!(anchor.uuid, uuid);
        assert_eq!(anchor.kind, Kind::Begin);
        assert!(anchor.parameters.is_empty());
        assert_eq!(anchor.arguments, vec!["arg1", "arg2", "arg3"]);
        assert_eq!(anchor.range, create_range(9, 2, 1, 80, 5, 5));
    } else {
        panic!("Expected Anchor node");
    }

    if let Node::Text(text) = &root.children[2] {
        assert_eq!(text.content, "After.");
    } else {
        panic!("Expected Text node");
    }
}
