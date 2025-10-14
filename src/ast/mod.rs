

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_parameters_empty() {
        let document = "some text";
        let parameters = parse_parameters(document, 0).unwrap();
        assert_eq!(parameters.parameters, json!({}));
        assert_eq!(parameters.range.begin, 0);
        assert_eq!(parameters.range.end, 0);
    }

    #[test]
    fn test_parse_parameters_simple() {
        let document = "{\"key\":\"value\"} some text";
        let parameters = parse_parameters(document, 0).unwrap();
        assert_eq!(parameters.parameters, json!({"key":"value"}));
        assert_eq!(parameters.range.begin, 0);
        assert_eq!(parameters.range.end, "{\"key\":\"value\"}".len());
    }

    #[test]
    fn test_parse_parameters_nested() {
        let document = "{\"key\":{\"nested_key\":123}} some text";
        let parameters = parse_parameters(document, 0).unwrap();
        assert_eq!(parameters.parameters, json!({"key":{"nested_key":123}}));
        assert_eq!(parameters.range.begin, 0);
        assert_eq!(parameters.range.end, "{\"key\":{\"nested_key\":123}}".len());
    }

    #[test]
    fn test_parse_parameters_unmatched_brace() {
        let document = "{\"key\":\"value\" some text";
        let err = parse_parameters(document, 0).unwrap_err();
        assert!(matches!(err, ParsingError::UnexpectedToken(_)));
    }

    #[test]
    fn test_parse_argument_word() {
        let document = "word next";
        let argument = parse_argument(document, 0).unwrap();
        assert_eq!(document[argument.range.begin..argument.range.end], "word");
    }

    #[test]
    fn test_parse_argument_single_quoted() {
        let document = "'single quoted' next";
        let argument = parse_argument(document, 0).unwrap();
        assert_eq!(document[argument.range.begin..argument.range.end], "'single quoted'");
    }

    #[test]
    fn test_parse_argument_double_quoted() {
        let document = \"\"double quoted\" next\";
        let argument = parse_argument(document, 0).unwrap();
        assert_eq!(document[argument.range.begin..argument.range.end], "\"double quoted\"");
    }

    #[test]
    fn test_parse_argument_quoted_with_escape() {
        let document = "\"escaped \\"quote\\"\" next";
        let argument = parse_argument(document, 0).unwrap();
        assert_eq!(document[argument.range.begin..argument.range.end], "\"escaped \\"quote\\"\"");
    }

    #[test]
    fn test_parse_argument_unclosed_quote() {
        let document = "\"unclosed quote next";
        let err = parse_argument(document, 0).unwrap_err();
        assert!(matches!(err, ParsingError::UnexpectedToken(_)));
    }

    #[test]
    fn test_parse_arguments_multiple() {
        let document = "arg1 'arg 2' \"arg 3\" {json}";
        let arguments = parse_arguments(document, 0).unwrap();
        assert_eq!(arguments.children.len(), 3);
        assert_eq!(document[arguments.children[0].range.begin..arguments.children[0].range.end], "arg1");
        assert_eq!(document[arguments.children[1].range.begin..arguments.children[1].range.end], "'arg 2'");
        assert_eq!(document[arguments.children[2].range.begin..arguments.children[2].range.end], "\"arg 3\"");
    }

    #[test]
    fn test_parse_tag_simple() {
        let document = "@include arg1 arg2\n";
        let tag = parse_tag(document, 0).unwrap().unwrap();
        assert_eq!(tag.opening.command, Command::Include);
        assert_eq!(tag.arguments.children.len(), 2);
        assert_eq!(document[tag.arguments.children[0].range.begin..tag.arguments.children[0].range.end], "arg1");
    }

    #[test]
    fn test_parse_tag_with_parameters() {
        let document = "@inline {\"file\":\"test.md\"} arg1\n";
        let tag = parse_tag(document, 0).unwrap().unwrap();
        assert_eq!(tag.opening.command, Command::Inline);
        assert_eq!(tag.parameters.parameters, json!({"file":"test.md"}));
        assert_eq!(tag.arguments.children.len(), 1);
    }

    #[test]
    fn test_parse_anchor_simple() {
        let uuid_str = "123e4567-e89b-12d3-a456-426614174000";
        let document = format!("<!-- include-{}:Begin -->\n", uuid_str);
        let anchor = parse_anchor(&document, 0).unwrap().unwrap();
        assert_eq!(anchor.opening.command, Command::Include);
        assert_eq!(anchor.opening.kind, Kind::Begin);
        assert_eq!(anchor.opening.uuid.to_string(), uuid_str);
    }

    #[test]
    fn test_parse_text_simple() {
        let document = "This is some plain text.\n@include tag";
        let text = parse_text(document, 0).unwrap().unwrap();
        assert_eq!(document[text.range.begin..text.range.end], "This is some plain text.\n");
    }

    #[test]
    fn test_parse_many_nodes_mixed() {
        let uuid_str = "123e4567-e89b-12d3-a456-426614174000";
        let document = format!("Text 1\n@include arg1\n<!-- include-{}:End -->\nText 2", uuid_str);
        let nodes = parse_many_nodes(&document, 0).unwrap();
        assert_eq!(nodes.len(), 4);

        match &nodes[0] {
            Node::Text(text) => assert_eq!(document[text.range.begin..text.range.end], "Text 1\n"),
            _ => panic!("Expected Text node"),
        }
        match &nodes[1] {
            Node::Tag(tag) => assert_eq!(tag.opening.command, Command::Include),
            _ => panic!("Expected Tag node"),
        }
        match &nodes[2] {
            Node::Anchor(anchor) => assert_eq!(anchor.opening.kind, Kind::End),
            _ => panic!("Expected Anchor node"),
        }
        match &nodes[3] {
            Node::Text(text) => assert_eq!(document[text.range.begin..text.range.end], "Text 2"),
            _ => panic!("Expected Text node"),
        }
    }
}
