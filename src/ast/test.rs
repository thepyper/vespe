#![cfg(test)]

use super::parser::{parse_context, parse_line, parse_snippet};
use super::resolver::Resolver;
use super::types::{AnchorKind, LineKind};
use std::path::{Path, PathBuf};
use uuid::Uuid;

// Mock Resolver for testing purposes
struct MockResolver {
    temp_dir: PathBuf,
}

impl MockResolver {
    fn new(temp_dir: PathBuf) -> Self {
        MockResolver { temp_dir }
    }
}

impl Resolver for MockResolver {
    fn resolve_context(&self, ctx_name: &str) -> PathBuf {
        let relative_path = match ctx_name {
            "test_ctx_1" => PathBuf::from("test_data/test_ctx_1.md"),
            "test_ctx_2" => PathBuf::from("test_data/test_ctx_2.md"),
            "my_context_name" => PathBuf::from("test_data/my_context_name.md"),
            "my_context" => PathBuf::from("test_data/my_context.md"),
            "my_summary_context" => PathBuf::from("test_data/my_summary_context.md"),
            _ => PathBuf::from(format!("test_data/{}.md", ctx_name)),
        };
        let resolved_path = self.temp_dir.join(relative_path);
        dbg!(&resolved_path);
        resolved_path
    }

    fn resolve_snippet(&self, snippet_name: &str) -> PathBuf {
        let relative_path = match snippet_name {
            "test_snip_1" => PathBuf::from("test_data/test_snip_1.sn"),
            "another_snip" => PathBuf::from("test_data/another_snip.sn"),
            "my_snippet" => PathBuf::from("test_data/my_snippet.sn"),
            _ => PathBuf::from(format!("test_data/{}.sn", snippet_name)),
        };
        let resolved_path = self.temp_dir.join(relative_path);
        dbg!(&resolved_path);
        resolved_path
    }
}

#[test]
fn test_parse_line_plain_text() {
    let resolver = MockResolver::new(PathBuf::new());
    let line_text = "This is a plain text line.";
    let line = parse_line(line_text, &resolver).unwrap();
    assert_eq!(line.text, line_text);
    assert!(matches!(line.kind, LineKind::Text));
    assert!(line.anchor.is_none());
}

#[test]
fn test_parse_line_with_valid_anchor() {
    let resolver = MockResolver::new(PathBuf::new());
    let line_text = "Text with an anchor. <!-- inline-12345678-1234-5678-1234-567812345678:some_data -->";
    let result = parse_line(line_text, &resolver);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Unknown anchor data value: some_data"));
}

#[test]
fn test_parse_line_with_anchor_no_data() {
    let resolver = MockResolver::new(PathBuf::new());
    let line_text = "Text with an anchor, no data. <!-- answer-11111111-1111-1111-1111-111111111111 -->";
    let line = parse_line(line_text, &resolver).unwrap();
    assert_eq!(line.text, "Text with an anchor, no data.");
    assert!(matches!(line.kind, LineKind::Text));
    assert!(line.anchor.is_some());
    let anchor = line.anchor.unwrap();
    assert!(matches!(anchor.kind, AnchorKind::Answer));
    assert_eq!(
        anchor.uid,
        Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap()
    );
    assert!(anchor.data.is_none());
}

#[test]
fn test_parse_line_with_anchor_begin_data() {
    let resolver = MockResolver::new(PathBuf::new());
    let line_text = "Text with an anchor, begin data. <!-- inline-22222222-2222-2222-2222-222222222222:begin -->";
    let line = parse_line(line_text, &resolver).unwrap();
    assert_eq!(line.text, "Text with an anchor, begin data.");
    assert!(matches!(line.kind, LineKind::Text));
    assert!(line.anchor.is_some());
    let anchor = line.anchor.unwrap();
    assert!(matches!(anchor.kind, AnchorKind::Inline));
    assert_eq!(
        anchor.uid,
        Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap()
    );
    assert_eq!(anchor.data, Some(super::types::AnchorDataValue::Begin));
}

#[test]
fn test_parse_line_with_anchor_end_data() {
    let resolver = MockResolver::new(PathBuf::new());
    let line_text = "Text with an anchor, end data. <!-- inline-33333333-3333-3333-3333-333333333333:end -->";
    let line = parse_line(line_text, &resolver).unwrap();
    assert_eq!(line.text, "Text with an anchor, end data.");
    assert!(matches!(line.kind, LineKind::Text));
    assert!(line.anchor.is_some());
    let anchor = line.anchor.unwrap();
    assert!(matches!(anchor.kind, AnchorKind::Inline));
    assert_eq!(
        anchor.uid,
        Uuid::parse_str("33333333-3333-3333-3333-333333333333").unwrap()
    );
    assert_eq!(anchor.data, Some(super::types::AnchorDataValue::End));
}

#[test]
fn test_parse_line_with_invalid_anchor_format() {
    let resolver = MockResolver::new(PathBuf::new());
    let line_text = "Text with a malformed anchor. <!-- inline-invalid_uuid:some_data -->";
    let line = parse_line(line_text, &resolver).unwrap();
    assert_eq!(line.text, line_text); // Malformed anchor is treated as part of text
    assert!(matches!(line.kind, LineKind::Text));
    assert!(line.anchor.is_none());
}

#[test]
fn test_parse_line_with_include_tag() {
    let temp_dir = tempfile::tempdir().unwrap();
    let resolver = MockResolver::new(temp_dir.path().to_path_buf());
    let ctx_path = temp_dir.path().join("test_data/my_context_name.md");
    create_temp_file(&ctx_path, "Included content");

    let line_text = "This is an include line."; // The text after the tag

    // Manually construct the expected Context for the include tag
    let included_context = super::types::Context {
        path: resolver.resolve_context("my_context_name"),
        lines: vec![super::types::Line { kind: LineKind::Text, text: "Included content".to_string(), anchor: None }], // Populated with content
    };
    let mut parameters = std::collections::HashMap::new();
    parameters.insert("key".to_string(), serde_json::json!("value"));

    let expected_line = super::types::Line {
        kind: LineKind::Include {
            context: included_context,
            parameters,
        },
        text: line_text.to_string(),
        anchor: None,
    };

    // Now, compare the manually constructed line with the one parsed by parse_line
    let parsed_line = parse_line("@include[key = \"value\"] my_context_name This is an include line.", &resolver).unwrap();
    assert_eq!(parsed_line, expected_line);

    temp_dir.close().unwrap();
}

#[test]
fn test_parse_line_with_answer_tag_and_anchor() {
    let resolver = MockResolver::new(PathBuf::new());
    let line_text = "@answer[param = \"test\"] Answer line. <!-- answer-87654321-4321-8765-4321-876543214321:more_data -->";
    let result = parse_line(line_text, &resolver);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Unknown anchor data value: more_data"));
}

#[test]
fn test_parse_line_with_malformed_tag() {
    let resolver = MockResolver::new(PathBuf::new());
    let line_text = "@malformed[key=value] This line has a malformed tag.";
    let line = parse_line(line_text, &resolver).unwrap();
    assert_eq!(line.text, line_text); // Malformed tag is treated as part of text
    assert!(matches!(line.kind, LineKind::Text));
    assert!(line.anchor.is_none());
}

#[test]
fn test_parse_line_with_include_tag_only_argument() {
    let temp_dir = tempfile::tempdir().unwrap();
    let resolver = MockResolver::new(temp_dir.path().to_path_buf());
    let ctx_path = temp_dir.path().join("test_data/my_context.md");
    create_temp_file(&ctx_path, "Included content for my_context");

    // Manually construct the expected Context for the include tag
    let included_context = super::types::Context {
        path: resolver.resolve_context("my_context"),
        lines: vec![super::types::Line { kind: LineKind::Text, text: "Included content for my_context".to_string(), anchor: None }],
    };

    let expected_line = super::types::Line {
        kind: LineKind::Include {
            context: included_context,
            parameters: std::collections::HashMap::new(),
        },
        text: "".to_string(),
        anchor: None,
    };

    // Now, compare the manually constructed line with the one parsed by parse_line
    let parsed_line = parse_line("@include my_context", &resolver).unwrap();
    assert_eq!(parsed_line, expected_line);

    temp_dir.close().unwrap();
}

#[test]
fn test_parse_line_with_inline_tag_only_argument() {
    let temp_dir = tempfile::tempdir().unwrap();
    let resolver = MockResolver::new(temp_dir.path().to_path_buf());
    let snip_path = temp_dir.path().join("test_data/my_snippet.sn");
    create_temp_file(&snip_path, "Snippet content for my_snippet");

    // Manually construct the expected Snippet for the inline tag
    let included_snippet = super::types::Snippet {
        path: resolver.resolve_snippet("my_snippet"),
        lines: vec![super::types::Line { kind: LineKind::Text, text: "Snippet content for my_snippet".to_string(), anchor: None }],
    };

    let expected_line = super::types::Line {
        kind: LineKind::Inline {
            snippet: included_snippet,
            parameters: std::collections::HashMap::new(),
        },
        text: "".to_string(),
        anchor: None,
    };

    // Now, compare the manually constructed line with the one parsed by parse_line
    let parsed_line = parse_line("@inline my_snippet", &resolver).unwrap();
    assert_eq!(parsed_line, expected_line);

    temp_dir.close().unwrap();
}

#[test]
fn test_parse_line_with_summary_tag_only_argument() {
    let temp_dir = tempfile::tempdir().unwrap();
    let resolver = MockResolver::new(temp_dir.path().to_path_buf());
    let ctx_path = temp_dir.path().join("test_data/my_summary_context.md");
    create_temp_file(&ctx_path, "Summarized content for my_summary_context");

    // Manually construct the expected Context for the summary tag
    let summarized_context = super::types::Context {
        path: resolver.resolve_context("my_summary_context"),
        lines: vec![super::types::Line { kind: LineKind::Text, text: "Summarized content for my_summary_context".to_string(), anchor: None }],
    };

    let expected_line = super::types::Line {
        kind: LineKind::Summary {
            context: summarized_context,
            parameters: std::collections::HashMap::new(),
        },
        text: "".to_string(),
        anchor: None,
    };

    // Now, compare the manually constructed line with the one parsed by parse_line
    let parsed_line = parse_line("@summary my_summary_context", &resolver).unwrap();
    assert_eq!(parsed_line, expected_line);

    temp_dir.close().unwrap();
}

#[test]
fn test_parse_line_with_answer_tag_only_parameters() {
    let resolver = MockResolver::new(PathBuf::new());
    let line_text = "@answer[param = \"value\"]";
    let line = parse_line(line_text, &resolver).unwrap();
    assert_eq!(line.text, "");
    assert!(matches!(line.kind, LineKind::Answer { .. }));
    if let LineKind::Answer { parameters } = line.kind {
        assert_eq!(parameters.get("param").unwrap(), &serde_json::json!("value"));
    } else {
        panic!("Expected Answer LineKind");
    }
    assert!(line.anchor.is_none());
}

#[test]
fn test_parse_line_with_answer_tag_no_params_no_args() {
    let resolver = MockResolver::new(PathBuf::new());
    let line_text = "@answer";
    let line = parse_line(line_text, &resolver).unwrap();
    assert_eq!(line.text, "");
    assert!(matches!(line.kind, LineKind::Answer { .. }));
    if let LineKind::Answer { parameters } = line.kind {
        assert!(parameters.is_empty());
    } else {
        panic!("Expected Answer LineKind");
    }
    assert!(line.anchor.is_none());
}

#[test]
fn test_parse_line_with_include_tag_no_params_no_args() {
    let temp_dir = tempfile::tempdir().unwrap();
    let resolver = MockResolver::new(temp_dir.path().to_path_buf());
    let ctx_path = temp_dir.path().join("test_data/.md"); // Default/empty context path
    create_temp_file(&ctx_path, "Included content for default context");

    // Manually construct the expected Context for the include tag
    let included_context = super::types::Context {
        path: resolver.resolve_context(""), // Should resolve default/empty context
        lines: vec![super::types::Line { kind: LineKind::Text, text: "Included content for default context".to_string(), anchor: None }],
    };

    let expected_line = super::types::Line {
        kind: LineKind::Include {
            context: included_context,
            parameters: std::collections::HashMap::new(),
        },
        text: "".to_string(),
        anchor: None,
    };

    // Now, compare the manually constructed line with the one parsed by parse_line
    let parsed_line = parse_line("@include", &resolver).unwrap();
    assert_eq!(parsed_line, expected_line);

    temp_dir.close().unwrap();
}

// Helper function to create a temporary file
fn create_temp_file(path: &Path, content: &str) {
    dbg!(&path, &content);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

#[test]
fn test_parse_context() {
    let temp_dir = tempfile::tempdir().unwrap();
    let resolver = MockResolver::new(temp_dir.path().to_path_buf());
    let ctx_path_1 = temp_dir.path().join("test_data/test_ctx_1.md");
    let ctx_path_2 = temp_dir.path().join("test_data/test_ctx_2.md");

    let content_1 = r###"Line 1
@include[key = "value"] test_ctx_2 Line 2
Line 3 <!-- inline-12345678-1234-5678-1234-567812345678:begin -->
"###;
    let content_2 = r###"Included Line 1
Included Line 2
"###;

    create_temp_file(&ctx_path_1, content_1);
    create_temp_file(&ctx_path_2, content_2);

    let context = parse_context(&ctx_path_1, &resolver).unwrap();
    assert_eq!(context.path, ctx_path_1);
    assert_eq!(context.lines.len(), 3);

    // Verify Line 1
    assert_eq!(context.lines[0].text, "Line 1");
    assert!(matches!(context.lines[0].kind, LineKind::Text));

    // Verify Line 2 (Include)
    assert_eq!(context.lines[1].text, "Line 2");
    assert!(matches!(context.lines[1].kind, LineKind::Include { .. }));
    if let LineKind::Include { context: included_ctx, parameters } = &context.lines[1].kind {
        assert_eq!(included_ctx.path, resolver.resolve_context("test_ctx_2"));
        assert_eq!(parameters.get("key").unwrap(), &serde_json::json!("value"));
    }

    // Verify Line 3 (Text with Anchor)
    assert_eq!(context.lines[2].text, "Line 3");
    assert!(matches!(context.lines[2].kind, LineKind::Text));
    assert!(context.lines[2].anchor.is_some());
    let anchor = context.lines[2].anchor.as_ref().unwrap();
    assert!(matches!(anchor.kind, AnchorKind::Inline));
    assert_eq!(anchor.data, Some(super::types::AnchorDataValue::Begin));

    temp_dir.close().unwrap();
}

#[test]
fn test_parse_snippet() {
    let temp_dir = tempfile::tempdir().unwrap();
    let resolver = MockResolver::new(temp_dir.path().to_path_buf());
    let snip_path_1 = temp_dir.path().join("test_data/test_snip_1.sn");
    let snip_path_2 = temp_dir.path().join("test_data/another_snip.sn");

    let content_1 = r###"Snippet Line 1
@inline[key = "value"] another_snip Snippet Line 2
"###;
    let content_2 = r###"Another Snippet Line 1
Another Snippet Line 2
"###;

    create_temp_file(&snip_path_1, content_1);
    create_temp_file(&snip_path_2, content_2);

    let snippet = parse_snippet(&snip_path_1, &resolver).unwrap();
    assert_eq!(snippet.path, snip_path_1);
    assert_eq!(snippet.lines.len(), 2);

    // Verify Snippet Line 1
    assert_eq!(snippet.lines[0].text, "Snippet Line 1");
    assert!(matches!(snippet.lines[0].kind, LineKind::Text));

    // Verify Snippet Line 2 (Inline)
    assert_eq!(snippet.lines[1].text, "Snippet Line 2");
    assert!(matches!(snippet.lines[1].kind, LineKind::Inline { .. }));
    if let LineKind::Inline { snippet: included_snip, parameters } = &snippet.lines[1].kind {
        assert_eq!(included_snip.path, resolver.resolve_snippet("another_snip"));
        assert_eq!(parameters.get("key").unwrap(), &serde_json::json!("value"));
    }

    temp_dir.close().unwrap();
}