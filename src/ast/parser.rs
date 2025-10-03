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
    if let Some((anchor, remaining_text)) = parse_anchor(&current_line_text) {
        anchor_data = Some(anchor);
        current_line_text = remaining_text;
    }

    // Parse tag
    if let Some((tag_name, params_str_opt, args_str_opt, remaining_text)) = parse_tag_and_content(&current_line_text) {
        let parameters = if let Some(params_str) = params_str_opt {
            parse_parameters(&params_str)?
        } else {
            HashMap::new()
        };

        let new_line_kind = match tag_name.as_str() {
            "include" => {
                let ctx_name = args_str_opt.unwrap_or_default(); // Use argument directly
                let context_path = resolver.resolve_context(&ctx_name);
                let context = parse_context(&context_path.to_string_lossy(), resolver)?;
                LineKind::Include { context, parameters }
            },
            "inline" => {
                let snippet_name = args_str_opt.unwrap_or_default(); // Use argument directly
                let snippet_path = resolver.resolve_snippet(&snippet_name);
                let snippet = parse_snippet(&snippet_path.to_string_lossy(), resolver)?;
                LineKind::Inline { snippet, parameters }
            },
            "answer" => LineKind::Answer { parameters },
            "summary" => {
                let ctx_name = args_str_opt.unwrap_or_default(); // Use argument directly
                let context_path = resolver.resolve_context(&ctx_name);
                let context = parse_context(&context_path.to_string_lossy(), resolver)?;
                LineKind::Summary { context, parameters }
            },
            _ => LineKind::Text, // Unknown tag, treat as text
        };

        if !matches!(new_line_kind, LineKind::Text) {
            line_kind = new_line_kind;
            current_line_text = remaining_text; // Use the remaining text from tag parsing
        }
    }

    Ok(Line {
        kind: line_kind,
        text: current_line_text.trim().to_string(), // Final trim
        anchor: anchor_data,
    })
}

/// Parses an anchor from the end of a line.
fn parse_anchor(line_text: &str) -> Option<(AnchorData, String)> {
    let anchor_regex = Regex::new(r"<!-- (inline|answer)-([0-9a-fA-F-]+):(.*?) -->$").unwrap();
    if let Some(captures) = anchor_regex.captures(line_text) {
        let kind_str = captures.get(1).unwrap().as_str();
        let uid_str = captures.get(2).unwrap().as_str();
        let data_str = captures.get(3).unwrap().as_str();

        let kind = match kind_str {
            "inline" => AnchorKind::Inline,
            "answer" => AnchorKind::Answer,
            _ => unreachable!(), // Regex ensures this won't happen
        };
        let uid = Uuid::parse_str(uid_str).unwrap(); // Handle error properly in real code

        let anchor_data = AnchorData {
            kind,
            uid,
            data: data_str.to_string(),
        };

        let remaining_text = anchor_regex.replace(line_text, "").trim().to_string();
        Some((anchor_data, remaining_text))
    } else {
        None
    }
}

/// Parses a tag, its optional parameters, and optional arguments from the beginning of a line.
fn parse_tag_and_content(line_text: &str) -> Option<(String, Option<String>, Option<String>, String)> {
    // Regex to capture tag name, optional parameters, and optional arguments
    let tag_full_regex = Regex::new(r"^@([a-zA-Z]+)(?:\[(.*?)\])?(?:\s+([^\s].*?))?").unwrap();

    if let Some(captures) = tag_full_regex.captures(line_text) {
        let tag_name = captures.get(1).unwrap().as_str().to_string();
        let params_str = captures.get(2).map(|m| m.as_str().to_string());
        let args_str = captures.get(3).map(|m| m.as_str().to_string());

        let remaining_text = line_text[captures.get(0).unwrap().end()..].trim().to_string();
        Some((tag_name, params_str, args_str, remaining_text))
    } else {
        None
    }
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
