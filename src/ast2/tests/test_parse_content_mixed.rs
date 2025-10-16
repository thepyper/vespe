use crate::ast2::*;
use crate::ast2::tests::utils::*;
use anyhow::Result;
use serde_json::json;
use uuid::Uuid;

#[test]
fn test_parse_content_mixed() -> Result<()> {
    let uuid_str = "123e4567-e89b-12d3-a456-426614174000";
    let input = format!("Some text.\n@tag arg1\n<!-- answer-{}:begin -->\nMore text.\n<!-- answer-{}:end -->\nFinal text.", uuid_str, uuid_str);
    let mut parser = Parser::new(&input);
    let content = parse_content(&input, &mut parser)?;

    assert_eq!(content.len(), 6);

    // Text 1
    assert!(matches!(content[0], Content::Text(_)));
    if let Content::Text(text) = &content[0] {
        assert_eq!(text.range, create_range(0, 1, 1, 11, 2, 1));
    }

    // Tag
    assert!(matches!(content[1], Content::Tag(_)));
    if let Content::Tag(tag) = &content[1] {
        assert!(matches!(tag.command, CommandKind::Tag));
        assert_eq!(tag.arguments.arguments[0].value, "arg1");
        assert_eq!(tag.range, create_range(11, 2, 1, 22, 2, 12));
    }

    // Anchor Begin
    assert!(matches!(content[2], Content::Anchor(_)));
    if let Content::Anchor(anchor) = &content[2] {
        assert!(matches!(anchor.command, CommandKind::Answer));
        assert!(matches!(anchor.kind, AnchorKind::Begin));
        assert_eq!(anchor.uuid.to_string(), uuid_str);
        assert_eq!(anchor.range, create_range(22, 2, 12, 71, 3, 1));
    }

    // Text 2
    assert!(matches!(content[3], Content::Text(_)));
    if let Content::Text(text) = &content[3] {
        assert_eq!(text.range, create_range(71, 3, 1, 82, 4, 1));
    }

    // Anchor End
    assert!(matches!(content[4], Content::Anchor(_)));
    if let Content::Anchor(anchor) = &content[4] {
        assert!(matches!(anchor.command, CommandKind::Answer));
        assert!(matches!(anchor.kind, AnchorKind::End));
        assert_eq!(anchor.uuid.to_string(), uuid_str);
        assert_eq!(anchor.range, create_range(82, 4, 1, 131, 5, 1));
    }

    // Text 3
    assert!(matches!(content[5], Content::Text(_)));
    if let Content::Text(text) = &content[5] {
        assert_eq!(text.range, create_range(131, 5, 1, 142, 5, 12));
    }

    Ok(())
}
