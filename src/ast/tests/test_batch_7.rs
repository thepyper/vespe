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
fn test_parse_node_tag() {
    let mut parser = Parser::new("@include arg");
    let node = parse_node(&mut parser).unwrap().unwrap();
    assert!(matches!(node, Node::Tag(_)));
}

#[test]
fn test_parse_node_anchor() {
    let uuid = Uuid::new_v4();
    let document = format!("<!-- include-{}:begin -->", uuid);
    let mut parser = Parser::new(&document);
    let node = parse_node(&mut parser).unwrap().unwrap();
    assert!(matches!(node, Node::Anchor(_)));
}

#[test]
fn test_parse_node_text() {
    let mut parser = Parser::new("Just some text.");
    let node = parse_node(&mut parser).unwrap().unwrap();
    assert!(matches!(node, Node::Text(_)));
}

#[test]
fn test_parse_mixed_content() {
    let uuid1 = Uuid::new_v4();
    let uuid2 = Uuid::new_v4();
    let document = format!("Some initial text.\n@include file.md arg1\n<!-- derive-{}:begin -->\nMore text here.\n<!-- derive-{}:end -->\nFinal text.\n", uuid1, uuid2);

    let root = parse(&document).unwrap();
    assert_eq!(root.children.len(), 6);

    // Node 1: Text
    if let Node::Text(text) = &root.children[0] {
        assert_eq!(text.content, "Some initial text.\n");
    } else {
        panic!("Expected Text node");
    }

    // Node 2: Tag
    if let Node::Tag(tag) = &root.children[1] {
        assert_eq!(tag.command, Command::Include);
        assert_eq!(tag.arguments, vec!["file.md", "arg1"]);
    } else {
        panic!("Expected Tag node");
    }

    // Node 3: Anchor (begin)
    if let Node::Anchor(anchor) = &root.children[2] {
        assert_eq!(anchor.command, Command::Derive);
        assert_eq!(anchor.uuid, uuid1);
        assert_eq!(anchor.kind, Kind::Begin);
    } else {
        panic!("Expected Anchor node");
    }

    // Node 4: Text
    if let Node::Text(text) = &root.children[3] {
        assert_eq!(text.content, "More text here.\n");
    } else {
        panic!("Expected Text node");
    }

    // Node 5: Anchor (end)
    if let Node::Anchor(anchor) = &root.children[4] {
        assert_eq!(anchor.command, Command::Derive);
        assert_eq!(anchor.uuid, uuid2);
        assert_eq!(anchor.kind, Kind::End);
    } else {
        panic!("Expected Anchor node");
    }

    // Node 6: Text
    if let Node::Text(text) = &root.children[5] {
        assert_eq!(text.content, "Final text.\n");
    } else {
        panic!("Expected Text node");
    }
}

#[test]
fn test_parse_empty_document() {
    let document = "";
    let root = parse(document).unwrap();
    assert!(root.children.is_empty());
    assert_eq!(root.range, create_range(0, 1, 1, 0, 1, 1));
}
