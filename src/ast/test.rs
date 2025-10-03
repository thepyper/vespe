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
            _ => PathBuf::from(format!("test_data/{}.md", ctx_name)),
        };
        self.temp_dir.join(relative_path)
    }

    fn resolve_snippet(&self, snippet_name: &str) -> PathBuf {
        let relative_path = match snippet_name {
            "test_snip_1" => PathBuf::from("test_data/test_snip_1.sn"),
            "another_snip" => PathBuf::from("test_data/another_snip.sn"),
            _ => PathBuf::from(format!("test_data/{}.sn", snippet_name)),
        };
        self.temp_dir.join(relative_path)
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
    let line = parse_line(line_text, &resolver).unwrap();
    assert_eq!(line.text, "Text with an anchor."); // Adjusted expected text
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
    let resolver = MockResolver::new(PathBuf::new());
    let line_text = "Text with a malformed anchor. <!-- inline-invalid_uuid:some_data -->";
    let line = parse_line(line_text, &resolver).unwrap();
    assert_eq!(line.text, line_text); // Malformed anchor is treated as part of text
    assert!(matches!(line.kind, LineKind::Text));
    assert!(line.anchor.is_none());
}

#[test]
fn test_parse_line_with_include_tag() {
    let resolver = MockResolver::new(PathBuf::new());
    let line_text = "@include[context = \"my_context\"; key = \"value\"] This is an include line.";
    let line = parse_line(line_text, &resolver).unwrap();
    assert_eq!(line.text, "");
    assert!(matches!(line.kind, LineKind::Include { .. }));
    if let LineKind::Include { context, parameters, arguments } = line.kind {
        assert_eq!(context.path, resolver.resolve_context("my_context"));
        assert_eq!(parameters.get("key").unwrap(), &serde_json::json!("value"));
        assert_eq!(arguments, Some("This is an include line.".to_string()));
    } else {
        panic!("Expected Include LineKind");
    }
    assert!(line.anchor.is_none());
}

#[test]
fn test_parse_line_with_answer_tag_and_anchor() {
    let resolver = MockResolver::new(PathBuf::new());
    let line_text = "@answer[param = \"test\"] Answer line. <!-- answer-87654321-4321-8765-4321-876543214321:more_data -->";
    let line = parse_line(line_text, &resolver).unwrap();
    assert_eq!(line.text, "");
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
fn test_parse_line_with_include_tag_and_arguments() {
    let temp_dir = tempfile::tempdir().unwrap();
    let resolver = MockResolver::new(temp_dir.path().to_path_buf());
    let line_text = "@include[context = \"my_context\"] some additional arguments here";
    let line = parse_line(line_text, &resolver).unwrap();
    assert_eq!(line.text, "");
    assert!(matches!(line.kind, LineKind::Include { .. }));
    if let LineKind::Include { context, parameters, arguments } = line.kind {
        assert_eq!(context.path, resolver.resolve_context("my_context"));
        assert!(parameters.is_empty());
        assert_eq!(arguments, Some("some additional arguments here".to_string()));
    } else {
        panic!("Expected Include LineKind");
    }
    assert!(line.anchor.is_none());
    temp_dir.close().unwrap();
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

// Helper function to create a temporary file
fn create_temp_file(path: &Path, content: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

#[test]
fn test_parse_context() {
    let temp_dir = tempfile::tempdir().unwrap();
    let resolver = MockResolver::new(temp_dir.path().to_path_buf());
    let ctx_path_1 = temp_dir.path().join("test_data/test_ctx_1.md");
    let ctx_path_2 = temp_dir.path().join("test_data/test_ctx_2.md");

    let content_1 = r#"Line 1
@include[context = "test_ctx_2"] Line 2
Line 3 <!-- inline-12345678-1234-5678-1234-567812345678:data -->
"#;
    let content_2 = r#"Included Line 1
Included Line 2
"#;

    create_temp_file(&ctx_path_1, content_1);
    create_temp_file(&ctx_path_2, content_2);

    let context = parse_context(ctx_path_1.to_str().unwrap(), &resolver).unwrap();
    assert_eq!(context.path, ctx_path_1);
    assert_eq!(context.lines.len(), 3);

    // Verify Line 1
    assert_eq!(context.lines[0].text, "Line 1");
    assert!(matches!(context.lines[0].kind, LineKind::Text));

    // Verify Line 2 (Include)
    assert_eq!(context.lines[1].text, "");
    assert!(matches!(context.lines[1].kind, LineKind::Include { .. }));
    if let LineKind::Include { context: included_ctx, parameters, arguments } = &context.lines[1].kind {
        assert_eq!(included_ctx.path, ctx_path_2);
        assert!(parameters.is_empty());
        assert!(arguments.is_none());
    }

    // Verify Line 3 (Text with Anchor)
    assert_eq!(context.lines[2].text, "Line 3");
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
    let resolver = MockResolver::new(temp_dir.path().to_path_buf());
    let snip_path_1 = temp_dir.path().join("test_data/test_snip_1.sn");
    let snip_path_2 = temp_dir.path().join("test_data/another_snip.sn");

    let content_1 = r#"Snippet Line 1
@inline[snippet = "another_snip"] Snippet Line 2
"#;
    let content_2 = r#"Another Snippet Line 1
Another Snippet Line 2
"#;

    create_temp_file(&snip_path_1, content_1);
    create_temp_file(&snip_path_2, content_2);

    let snippet = parse_snippet(snip_path_1.to_str().unwrap(), &resolver).unwrap();
    assert_eq!(snippet.path, snip_path_1);
    assert_eq!(snippet.lines.len(), 2);

    // Verify Snippet Line 1
    assert_eq!(snippet.lines[0].text, "Snippet Line 1");
    assert!(matches!(snippet.lines[0].kind, LineKind::Text));

    // Verify Snippet Line 2 (Inline)
    assert_eq!(snippet.lines[1].text, "");
    assert!(matches!(snippet.lines[1].kind, LineKind::Inline { .. }));
    if let LineKind::Inline { snippet: included_snip, parameters, arguments } = &snippet.lines[1].kind {
        assert_eq!(included_snip.path, snip_path_2);
        assert!(parameters.is_empty());
        assert!(arguments.is_none());
    }

    temp_dir.close().unwrap();
}
