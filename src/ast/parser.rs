use std::collections::HashMap;
use anyhow::Result;
use std::path::{Path, PathBuf};
use regex::Regex;
use std::fs;
use std::io::{self, BufRead};
use serde_json::Value;
use uuid::Uuid;

use super::types::{AnchorData, AnchorKind, Context, Line, LineKind, Parameters, Snippet};
use super::resolver::Resolver;

/// Parses a file into a vector of `Line`s.
fn parse_file_to_lines<R: Resolver>(path: &Path, resolver: &R) -> Result<Vec<Line>> {
    let file = fs::File::open(path)?;
    let reader = io::BufReader::new(file);
    let mut lines = Vec::new();

    for line_str in reader.lines() {
        let line_str = line_str?;
        let parsed_line = parse_line(&line_str, resolver)?;
        lines.push(parsed_line);
    }
    Ok(lines)
}

/// Parses a context file from the given path string.
pub fn parse_context<R: Resolver>(path_str: &str, resolver: &R) -> Result<Context> {
    let path = PathBuf::from(path_str);
    let lines = parse_file_to_lines(&path, resolver)?;
    Ok(Context { path, lines })
}

/// Parses a snippet file from the given path string.
pub fn parse_snippet<R: Resolver>(path_str: &str, resolver: &R) -> Result<Snippet> {
    let path = PathBuf::from(path_str);
    let lines = parse_file_to_lines(&path, resolver)?;
    Ok(Snippet { path, lines })
}

/// Parses a single line of text into a `Line` struct.
pub fn parse_line<R: Resolver>(text: &str, resolver: &R) -> Result<Line, anyhow::Error> {
    let mut line_kind = LineKind::Text;
    let mut current_line_text = text.to_string();
    let mut anchor_data: Option<AnchorData> = None;

    // Parse anchor
    if let Some((anchor, remaining_text)) = parse_anchor(&current_line_text)? {
        anchor_data = Some(anchor);
        current_line_text = remaining_text.to_string();
    }

    // Parse tag
    if let Some((kind, remaining_text)) = parse_tag_and_parameters(&current_line_text, resolver)? {
        line_kind = kind;
        current_line_text = remaining_text.to_string();
    }

    Ok(Line {
        kind: line_kind,
        text: current_line_text.trim().to_string(),
        anchor: anchor_data,
    })
}

/// Parses a string of parameters into a HashMap.
fn parse_parameters(params_str: &str) -> Result<Parameters, anyhow::Error> {
    let mut parameters = HashMap::new();
    for param in params_str.split(';') {
        let param = param.trim();
        if param.is_empty() {
            continue;
        }
        let parts: Vec<&str> = param.splitn(2, '=').collect();
        if parts.len() == 2 {
            let key = parts[0].trim().to_string();
            let value_str = parts[1].trim();
            // Attempt to parse as JSON, otherwise treat as string
            let value = serde_json::from_str(value_str).unwrap_or_else(|_| Value::String(value_str.to_string()));
            parameters.insert(key, value);
        }
    }
    Ok(parameters)
}

/// Parses an anchor from the end of a line.
/// Returns `Some((AnchorData, remaining_line_text))` if an anchor is found, `None` otherwise.
fn parse_anchor(line: &str) -> Result<Option<(AnchorData, &str)>> {
    let anchor_regex = Regex::new(r"<!-- (inline|answer)-([0-9a-fA-F-]+):(.*?) -->$")?;

    if let Some(captures) = anchor_regex.captures(line) {
        let kind_str = captures.get(1).unwrap().as_str();
        let uid_str = captures.get(2).unwrap().as_str();
        let data_str = captures.get(3).unwrap().as_str();

        let kind = match kind_str {
            "inline" => AnchorKind::Inline,
            "answer" => AnchorKind::Answer,
            _ => unreachable!(), // Regex ensures this won't happen
        };
        let uid = Uuid::parse_str(uid_str)?;

        let anchor_data = AnchorData {
            kind,
            uid,
            data: data_str.to_string(),
        };

        // Get the text before the anchor
        let (remaining_text, _) = line.split_at(captures.get(0).unwrap().start());
        Ok(Some((anchor_data, remaining_text.trim_end())))\n    } else {\n        Ok(None)\n    }\n}\n\n/// Parses a tag, its parameters, and any subsequent arguments from the beginning of a line.\n/// Returns `Some((LineKind, remaining_line_text))` if a tag is found, `None` otherwise.\npub fn parse_tag_and_parameters<'a, R: Resolver>(line: &'a str, resolver: &R) -> Result<Option<(LineKind, &'a str)>> {\n    let tag_regex = Regex::new(r\"^@([a-zA-Z]+)\[(.*?)\]\\s*(.*)$\")?;\n\n    if let Some(captures) = tag_regex.captures(line) {\n        let tag_name = captures.get(1).unwrap().as_str();\n        let params_str = captures.get(2).unwrap().as_str();\n        let arguments_str = captures.get(3).unwrap().as_str().trim();\n        let arguments = if arguments_str.is_empty() { None } else { Some(arguments_str.to_string()) };\n\n        let parameters = parse_parameters(params_str)?;\n\n        let line_kind = match tag_name {\n            \"include\" => {\n                let ctx_name = parameters.get(\"context\").and_then(|v| v.as_str()).unwrap_or_default();\n                let context_path = resolver.resolve_context(ctx_name);\n                let context = parse_context(&context_path.to_string_lossy(), resolver)?;\n                LineKind::Include { context, parameters, arguments }\n            },\n            \"inline\" => {\n                let snippet_name = parameters.get(\"snippet\").and_then(|v| v.as_str()).unwrap_or_default();\n                let snippet_path = resolver.resolve_snippet(snippet_name);\n                let snippet = parse_snippet(&snippet_path.to_string_lossy(), resolver)?;\n                LineKind::Inline { snippet, parameters, arguments }\n            },\n            \"answer\" => LineKind::Answer { parameters },\n            \"summary\" => {\n                let ctx_name = parameters.get(\"context\").and_then(|v| v.as_str()).unwrap_or_default();\n                let context_path = resolver.resolve_context(ctx_name);\n                let context = parse_context(&context_path.to_string_lossy(), resolver)?;\n                LineKind::Summary { context, parameters, arguments }\n            },\n            _ => LineKind::Text, // Unknown tag, treat as text\n        };\n\n        // Get the text before the tag\n        let (remaining_text, _) = line.split_at(captures.get(0).unwrap().start());\n        Ok(Some((line_kind, remaining_text.trim_end())))\n    } else {\n        Ok(None)\n    }\n}\n
