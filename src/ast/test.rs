#![cfg(test)]

use super::parser::{parse_context, parse_line, parse_snippet};
use super::resolver::Resolver;
use super::types::{AnchorData, AnchorKind, Context, Line, LineKind, Snippet};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use uuid::Uuid;

// Mock Resolver for testing purposes
struct MockResolver {
    base_dir: PathBuf,
}

impl Resolver for MockResolver {
    fn resolve_context(&self, ctx_name: &str) -> PathBuf {
        self.base_dir.join("test_data").join(format!("{}.md", ctx_name))
    }

    fn resolve_snippet(&self, snippet_name: &str) -> PathBuf {
        self.base_dir.join("test_data").join(format!("{}.sn", snippet_name))
    }
}

#[test]
fn test_parse_line_plain_text() {
    let resolver = MockResolver { base_dir: PathBuf::from(".") };
    let line_text = "This is a plain text line.";
    let line = parse_line(line_text, &resolver).unwrap();
    assert_eq!(line.text, line_text);
    assert!(matches!(line.kind, LineKind::Text));
    assert!(line.anchor.is_none());
}

#[test]
fn test_parse_line_with_valid_anchor() {
    let resolver = MockResolver { base_dir: PathBuf::from(".") };
    let line_text = "Text with an anchor. <!-- inline-12345678-1234-5678-1234-567812345678:some_data -->";
    let line = parse_line(line_text, &resolver).unwrap();
    assert_eq!(line.text, "Text with an anchor.");
    assert!(matches!(line.kind, LineKind::Text));
    assert!(line.anchor.is_some());
    let anchor = line.anchor.unwrap();
    assert!(matches!(anchor.kind, AnchorKind::Inline));
    assert_eq!(
        anchor.uid,
        Uuid::parse_str("12345678-1234-5678-1234-567812345678").unwrap()
    );
    assert_eq!(anchor.data, "some_data");
}

#[test]
fn test_parse_line_with_invalid_anchor_format() {
    let resolver = MockResolver { base_dir: PathBuf::from(".") };
    let line_text = "Text with a malformed anchor. <!-- inline-invalid_uuid:some_data -->";
    let line = parse_line(line_text, &resolver).unwrap();
    assert_eq!(line.text, line_text); // Malformed anchor is treated as part of text
    assert!(matches!(line.kind, LineKind::Text));
    assert!(line.anchor.is_none());
}

#[test]
fn test_parse_line_with_include_tag() {
    let temp_dir = tempfile::tempdir().unwrap();
    let resolver = MockResolver { base_dir: temp_dir.path().to_path_buf() };
    let line_text = r#"@include[context = "test_ctx_2"] This is an include line."#;
    // Create the included context file
    let included_ctx_path = temp_dir.path().join("test_data/test_ctx_2.md");
    create_temp_file(&included_ctx_path, "Included Line 1\nIncluded Line 2");

    let line = parse_line(line_text, &resolver).unwrap();
    assert_eq!(line.text, "This is an include line.");
    assert!(matches!(line.kind, LineKind::Include { .. }));
    if let LineKind::Include { context, parameters } = line.kind {
        assert_eq!(context.path, included_ctx_path);
        assert_eq!(parameters.get("context").and_then(|v| v.as_str()).unwrap(), "test_ctx_2");
    } else {
        panic!("Expected Include LineKind");
    }
    assert!(line.anchor.is_none());
    temp_dir.close().unwrap();
}

#[test]
fn test_parse_line_with_answer_tag_and_anchor() {
    let resolver = MockResolver { base_dir: PathBuf::from(".") };
    let line_text = r#"@answer[param = "test"] Answer line. <!-- answer-87654321-4321-8765-4321-876543214321:more_data -->"#;
    let line = parse_line(line_text, &resolver).unwrap();
    assert_eq!(line.text, "Answer line.");
    assert!(matches!(line.kind, LineKind::Answer { .. }));
    if let LineKind::Answer { parameters } = line.kind {
        assert_eq!(parameters.get("param").unwrap(), &serde_json::json!("test"));
    } else {
        panic!("Expected Answer LineKind");
    }
    assert!(line.anchor.is_some());
    let anchor = line.anchor.unwrap();
    assert!(matches!(anchor.kind, AnchorKind::Answer));
    assert_eq!(
        anchor.uid,
        Uuid::parse_str("87654321-4321-8765-4321-876543214321").unwrap()
    );
    assert_eq!(anchor.data, "more_data");
}

#[test]
fn test_parse_line_with_malformed_tag() {
    let resolver = MockResolver { base_dir: PathBuf::from(".") };
    let line_text = "@malformed[key=value] This line has a malformed tag.";
    let line = parse_line(line_text, &resolver).unwrap();
    assert_eq!(line.text, line_text); // Malformed tag is treated as part of text
    assert!(matches!(line.kind, LineKind::Text));
    assert!(line.anchor.is_none());
}

// Helper function to create a temporary file
fn create_temp_file(path: &Path, content: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

#[test]
fn test_parse_context() {
    let temp_dir = tempfile::tempdir().unwrap();
    let resolver = MockResolver { base_dir: temp_dir.path().to_path_buf() };

    // Create the main context file
    let ctx_path = temp_dir.path().join("test_data/test_ctx_1.md");
    let content = r#"Line 1
@include[context = "test_ctx_2"] Line 2
Line 3 <!-- inline-12345678-1234-5678-1234-567812345678:data -->"#;
    create_temp_file(&ctx_path, content);

    // Create the included context file
    let included_ctx_path = temp_dir.path().join("test_data/test_ctx_2.md");
    create_temp_file(&included_ctx_path, "Included Line 1\nIncluded Line 2");

    let context = parse_context(ctx_path.to_str().unwrap(), &resolver).unwrap();
    assert_eq!(context.path, ctx_path);
    assert_eq!(context.lines.len(), 3);

    // Verify Line 1
    assert_eq!(context.lines[0].text, "Line 1");
    assert!(matches!(context.lines[0].kind, LineKind::Text));

    // Verify Line 2 (Include)
    assert_eq!(context.lines[1].text, "Line 2"); // Trimmed
    assert!(matches!(context.lines[1].kind, LineKind::Include { .. }));
    if let LineKind::Include { context: included_ctx, parameters } = &context.lines[1].kind {
        assert_eq!(included_ctx.path, included_ctx_path);
        assert_eq!(parameters.get("context").and_then(|v| v.as_str()).unwrap(), "test_ctx_2");
    } else {
        panic!("Expected Include LineKind");
    }

    // Verify Line 3 (Text with Anchor)
    assert_eq!(context.lines[2].text, "Line 3"); // Trimmed
    assert!(matches!(context.lines[2].kind, LineKind::Text));
    assert!(context.lines[2].anchor.is_some());
    let anchor = context.lines[2].anchor.as_ref().unwrap();
    assert!(matches!(anchor.kind, AnchorKind::Inline));
    assert_eq!(anchor.data, "data");

    temp_dir.close().unwrap();
}

#[test]
fn test_parse_snippet() {
    let temp_dir = tempfile::tempdir().unwrap();
    let resolver = MockResolver { base_dir: temp_dir.path().to_path_buf() };

    // Create the main snippet file
    let snip_path = temp_dir.path().join("test_data/test_snip_1.sn");
    let content = r#"Snippet Line 1
@inline[snippet = "another_snip"] Snippet Line 2"#;
    create_temp_file(&snip_path, content);

    // Create the inlined snippet file
    let inlined_snip_path = temp_dir.path().join("test_data/another_snip.sn");
    create_temp_file(&inlined_snip_path, "Inlined Snippet Line 1\nInlined Snippet Line 2");

    let snippet = parse_snippet(snip_path.to_str().unwrap(), &resolver).unwrap();
    assert_eq!(snippet.path, snip_path);
    assert_eq!(snippet.lines.len(), 2);

    // Verify Snippet Line 1
    assert_eq!(snippet.lines[0].text, "Snippet Line 1");
    assert!(matches!(snippet.lines[0].kind, LineKind::Text));

    // Verify Snippet Line 2 (Inline)
    assert_eq!(snippet.lines[1].text, "Snippet Line 2"); // Trimmed
    assert!(matches!(snippet.lines[1].kind, LineKind::Inline { .. }));
    if let LineKind::Inline { snippet: included_snip, parameters } = &snippet.lines[1].kind {
        assert_eq!(included_snip.path, inlined_snip_path);
        assert_eq!(parameters.get("snippet").and_then(|v| v.as_str()).unwrap(), "another_snip");
    } else {
        panic!("Expected Inline LineKind");
    }

    temp_dir.close().unwrap();
}