use crate::ast::types::{AnchorDataValue, AnchorKind, Context, Line, LineKind, Snippet};
use crate::ast::visitor::walk_context;
use crate::visitors::inline_visitors::InlineBeginDecorator;
use std::path::PathBuf;
use uuid::Uuid;

#[test]
fn test_inline_begin_decorator() {
    // Create a Context with an @inline line without a Begin anchor
    let mut context = Context {
        path: PathBuf::from("mock_context.txt"),
        lines: vec![
            Line {
                kind: LineKind::Inline {
                    snippet: Snippet { path: PathBuf::from("mock_snippet.txt"), lines: vec![] },
                    parameters: std::collections::HashMap::new(),
                },
                text: "@inline my_snippet".to_string(),
                anchor: None,
            },
            Line {
                kind: LineKind::Text,
                text: "Some other text".to_string(),
                anchor: None,
            },
        ],
    };

    // Apply the InlineBeginDecorator
    let mut decorator = InlineBeginDecorator;
    walk_context(&mut context, &mut decorator);

    // Assert that the @inline line now has a Begin anchor
    let inline_line = &context.lines[0];
    assert!(matches!(inline_line.kind, LineKind::Inline { .. }));
    assert!(inline_line.anchor.is_some());

    let anchor = inline_line.anchor.as_ref().unwrap();
    assert_eq!(anchor.kind, AnchorKind::Inline);
    assert_eq!(anchor.data, Some(AnchorDataValue::Begin));

    // Assert that the text of the line has been updated to include the anchor string
    let expected_prefix = format!("<!-- inline-{}:begin -->", anchor.uid);
    assert!(inline_line.text.starts_with(&expected_prefix));
}
