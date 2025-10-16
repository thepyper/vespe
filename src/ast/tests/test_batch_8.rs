use crate::ast::*;

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
fn test_parse_only_whitespace() {
    let document = "   \n\n";
    let root = parse(document).unwrap();
    assert_eq!(root.children.len(), 1);

    if let Node::Text(text) = &root.children[0] {
        assert_eq!(text.content, "   \n\n");
        assert_eq!(text.range, create_range(0, 1, 1, 5, 3, 1));
    } else {
        panic!("Expected Text node");
    }
}

// New tests for @tag snippets

#[test]
fn test_parse_tag_simple_args() {
    let document = "Some text before.\n@tag arg1 arg2 arg3\nSome text after.";
    let root = parse(document).unwrap();
    assert_eq!(root.children.len(), 3);

    if let Node::Text(text) = &root.children[0] {
        assert_eq!(text.content, "Some text before.\n");
    } else {
        panic!("Expected Text node");
    }

    if let Node::Tag(tag) = &root.children[1] {
        assert_eq!(tag.command, Command::Tag);
        assert!(tag.parameters.is_empty());
        assert_eq!(tag.arguments, vec!["arg1", "arg2", "arg3"]);
        assert_eq!(tag.range, create_range(19, 2, 1, 35, 2, 17));
    } else {
        panic!("Expected Tag node");
    }

    if let Node::Text(text) = &root.children[2] {
        assert_eq!(text.content, "Some text after.");
    } else {
        panic!("Expected Text node");
    }
}

#[test]
fn test_parse_tag_empty_params() {
    let document = "Before.\n@tag {{}}\nAfter.";
    let root = parse(document).unwrap();
    assert_eq!(root.children.len(), 3);

    if let Node::Text(text) = &root.children[0] {
        assert_eq!(text.content, "Before.\n");
    } else {
        panic!("Expected Text node");
    }

    if let Node::Tag(tag) = &root.children[1] {
        assert_eq!(tag.command, Command::Tag);
        assert!(tag.parameters.is_empty());
        assert!(tag.arguments.is_empty());
        assert_eq!(tag.range, create_range(9, 2, 1, 17, 2, 9));
    } else {
        panic!("Expected Tag node");
    }

    if let Node::Text(text) = &root.children[2] {
        assert_eq!(text.content, "After.");
    } else {
        panic!("Expected Text node");
    }
}

#[test]
fn test_parse_tag_empty_params_args() {
    let document = "Before.\n@tag{} arg1 arg2 arg3\nAfter.";
    let root = parse(document).unwrap();
    assert_eq!(root.children.len(), 3);

    if let Node::Text(text) = &root.children[0] {
        assert_eq!(text.content, "Before.\n");
    } else {
        panic!("Expected Text node");
    }

    if let Node::Tag(tag) = &root.children[1] {
        assert_eq!(tag.command, Command::Tag);
        assert!(tag.parameters.is_empty());
        assert_eq!(tag.arguments, vec!["arg1", "arg2", "arg3"]);
        assert_eq!(tag.range, create_range(9, 2, 1, 29, 2, 21));
    } else {
        panic!("Expected Tag node");
    }

    if let Node::Text(text) = &root.children[2] {
        assert_eq!(text.content, "After.");
    } else {
        panic!("Expected Text node");
    }
}

#[test]
fn test_parse_tag_empty_params_multiline_args() {
    let document = "Before.\n@tag{\n} arg1 arg2 arg3\nAfter.";
    let root = parse(document).unwrap();
    assert_eq!(root.children.len(), 3);

    if let Node::Text(text) = &root.children[0] {
        assert_eq!(text.content, "Before.\n");
    } else {
        panic!("Expected Text node");
    }

    if let Node::Tag(tag) = &root.children[1] {
        assert_eq!(tag.command, Command::Tag);
        assert!(tag.parameters.is_empty());
        assert_eq!(tag.arguments, vec!["arg1", "arg2", "arg3"]);
        assert_eq!(tag.range, create_range(9, 2, 1, 31, 3, 15));
    } else {
        panic!("Expected Tag node");
    }

    if let Node::Text(text) = &root.children[2] {
        assert_eq!(text.content, "After.");
    } else {
        panic!("Expected Text node");
    }
}
