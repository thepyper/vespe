use super::*;

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
fn test_parse_tag_with_params_args() {
    let document = "Before.\n@tag{x=1,b=2,c=3,d=haha,e='hoho',f=\"huhu\"} arg1 arg2 arg3\nAfter.";
    let root = parse(document).unwrap();
    assert_eq!(root.children.len(), 3);

    if let Node::Text(text) = &root.children[0] {
        assert_eq!(text.content, "Before.\n");
    } else {
        panic!("Expected Text node");
    }

    if let Node::Tag(tag) = &root.children[1] {
        assert_eq!(tag.command, Command::Tag);
        assert_eq!(tag.parameters.len(), 6);
        assert_eq!(tag.parameters["x"], ParameterValue::Integer(1));
        assert_eq!(tag.parameters["b"], ParameterValue::Integer(2));
        assert_eq!(tag.parameters["c"], ParameterValue::Integer(3));
        assert_eq!(tag.parameters["d"], ParameterValue::String("haha".to_string()));
        assert_eq!(tag.parameters["e"], ParameterValue::String("hoho".to_string()));
        assert_eq!(tag.parameters["f"], ParameterValue::String("huhu".to_string()));
        assert_eq!(tag.arguments, vec!["arg1", "arg2", "arg3"]);
        assert_eq!(tag.range, create_range(9, 2, 1, 76, 2, 68));
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
fn test_parse_tag_with_params_spaced_args() {
    let document = "Before.\n@tag {x=1,b=2, c = 3, d   =     haha,   e=  'hoho',f  = \"huhu\"    }      arg1 arg2 arg3\nAfter.";
    let root = parse(document).unwrap();
    assert_eq!(root.children.len(), 3);

    if let Node::Text(text) = &root.children[0] {
        assert_eq!(text.content, "Before.\n");
    } else {
        panic!("Expected Text node");
    }

    if let Node::Tag(tag) = &root.children[1] {
        assert_eq!(tag.command, Command::Tag);
        assert_eq!(tag.parameters.len(), 6);
        assert_eq!(tag.parameters["x"], ParameterValue::Integer(1));
        assert_eq!(tag.parameters["b"], ParameterValue::Integer(2));
        assert_eq!(tag.parameters["c"], ParameterValue::Integer(3));
        assert_eq!(tag.parameters["d"], ParameterValue::String("haha".to_string()));
        assert_eq!(tag.parameters["e"], ParameterValue::String("hoho".to_string()));
        assert_eq!(tag.parameters["f"], ParameterValue::String("huhu".to_string()));
        assert_eq!(tag.arguments, vec!["arg1", "arg2", "arg3"]);
        assert_eq!(tag.range, create_range(9, 2, 1, 109, 2, 101));
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
fn test_parse_tag_with_params_multiline_args() {
    let document = "Before.\n@tag{\n    x=1,b=2,c=3,d=haha,e='hoho',f=\"huhu\"\n} arg1 arg2 arg3\nAfter.";
    let root = parse(document).unwrap();
    assert_eq!(root.children.len(), 3);

    if let Node::Text(text) = &root.children[0] {
        assert_eq!(text.content, "Before.\n");
    } else {
        panic!("Expected Text node");
    }

    if let Node::Tag(tag) = &root.children[1] {
        assert_eq!(tag.command, Command::Tag);
        assert_eq!(tag.parameters.len(), 6);
        assert_eq!(tag.parameters["x"], ParameterValue::Integer(1));
        assert_eq!(tag.parameters["b"], ParameterValue::Integer(2));
        assert_eq!(tag.parameters["c"], ParameterValue::Integer(3));
        assert_eq!(tag.parameters["d"], ParameterValue::String("haha".to_string()));
        assert_eq!(tag.parameters["e"], ParameterValue::String("hoho".to_string()));
        assert_eq!(tag.parameters["f"], ParameterValue::String("huhu".to_string()));
        assert_eq!(tag.arguments, vec!["arg1", "arg2", "arg3"]);
        assert_eq!(tag.range, create_range(9, 2, 1, 80, 4, 15));
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
fn test_parse_tag_with_params_newline_before_params_args() {
    let document = "Before.\n@tag\n{x=1,b=2,c=3,d=haha,e='hoho',f=\"huhu\"}\n arg1 arg2 arg3\nAfter.";
    let root = parse(document).unwrap();
    assert_eq!(root.children.len(), 3);

    if let Node::Text(text) = &root.children[0] {
        assert_eq!(text.content, "Before.\n");
    } else {
        panic!("Expected Text node");
    }

    if let Node::Tag(tag) = &root.children[1] {
        assert_eq!(tag.command, Command::Tag);
        assert_eq!(tag.parameters.len(), 6);
        assert_eq!(tag.parameters["x"], ParameterValue::Integer(1));
        assert_eq!(tag.parameters["b"], ParameterValue::Integer(2));
        assert_eq!(tag.parameters["c"], ParameterValue::Integer(3));
        assert_eq!(tag.parameters["d"], ParameterValue::String("haha".to_string()));
        assert_eq!(tag.parameters["e"], ParameterValue::String("hoho".to_string()));
        assert_eq!(tag.parameters["f"], ParameterValue::String("huhu".to_string()));
        assert_eq!(tag.arguments, vec!["arg1", "arg2", "arg3"]);
        assert_eq!(tag.range, create_range(9, 2, 1, 80, 4, 15));
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
fn test_parse_tag_with_params_multiline_spaced_args() {
    let document = "Before.\n@tag{\n    x=1,\n    b=2,\n    c=3,\n    d=haha,\n    e='hoho',\n    f=\"huhu\"\n} arg1 arg2 arg3\nAfter.";
    let root = parse(document).unwrap();
    assert_eq!(root.children.len(), 3);

    if let Node::Text(text) = &root.children[0] {
        assert_eq!(text.content, "Before.\n");
    } else {
        panic!("Expected Text node");
    }

    if let Node::Tag(tag) = &root.children[1] {
        assert_eq!(tag.command, Command::Tag);
        assert_eq!(tag.parameters.len(), 6);
        assert_eq!(tag.parameters["x"], ParameterValue::Integer(1));
        assert_eq!(tag.parameters["b"], ParameterValue::Integer(2));
        assert_eq!(tag.parameters["c"], ParameterValue::Integer(3));
        assert_eq!(tag.parameters["d"], ParameterValue::String("haha".to_string()));
        assert_eq!(tag.parameters["e"], ParameterValue::String("hoho".to_string()));
        assert_eq!(tag.parameters["f"], ParameterValue::String("huhu".to_string()));
        assert_eq!(tag.arguments, vec!["arg1", "arg2", "arg3"]);
        assert_eq!(tag.range, create_range(9, 2, 1, 109, 9, 15));
    } else {
        panic!("Expected Tag node");
    }

    if let Node::Text(text) = &root.children[2] {
        assert_eq!(text.content, "After.");
    } else {
        panic!("Expected Text node");
    }
}
