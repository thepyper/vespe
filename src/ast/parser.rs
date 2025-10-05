use super::types::*;
use uuid::Uuid;
use std::collections::HashMap;

// Helper function to parse quoted strings with escape sequences
fn parse_quoted_string(s: &str) -> Result<String, AstError> {
    let mut chars = s.chars().peekable();
    let mut result = String::new();

    if chars.next() != Some('"') {
        return Err(AstError::UnclosedQuote);
    }

    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                if let Some(next_c) = chars.next() {
                    match next_c {
                        '"' => result.push('"'),
                        '\\' => result.push('\\'), // Correctly push a single backslash
                        _ => result.push(next_c), // For now, just push the escaped char
                    }
                } else {
                    return Err(AstError::UnclosedQuote);
                }
            }
            '"' => return Ok(result),
            _ => result.push(c),
        }
    }
    Err(AstError::UnclosedQuote)
}

// Parses a single line into a Line struct
pub fn parse_line(line_str: &str) -> Result<Line, AstError> {
    let (content, anchor_option) = extract_anchor(line_str);

    let trimmed_content = content.trim_start();

    let line_kind = if trimmed_content.starts_with("@") {
        parse_tagged_line(trimmed_content)?
    } else {
        LineKind::Text(content.to_string())
    };

    Ok(Line {
        kind: line_kind,
        anchor: anchor_option,
    })
}

// Extracts the anchor comment from a line, if present
fn extract_anchor(line_str: &str) -> (String, Option<Anchor>) {
    if let Some(anchor_start) = line_str.rfind("<!--") {
        if let Some(anchor_end) = line_str[anchor_start..].find("-->") {
            let anchor_str = &line_str[anchor_start + "<!--".len()..anchor_start + anchor_end];
            let content_before_anchor = line_str[..anchor_start].trim_end();

            match parse_anchor(anchor_str) {
                Ok(anchor) => (content_before_anchor.to_string(), Some(anchor)),
                Err(_) => (line_str.to_string(), None), // If anchor parsing fails, treat as regular text
            }
        } else {
            (line_str.to_string(), None)
        }
    } else {
        (line_str.to_string(), None)
    }
}

// Parses the content within an anchor comment (e.g., "kind-uuid:tag")
fn parse_anchor(anchor_content: &str) -> Result<Anchor, AstError> {
    let parts: Vec<&str> = anchor_content.splitn(2, ':').collect();
    let (kind_uuid_str, tag_str) = if parts.len() == 2 {
        (parts[0], parts[1])
    } else {
        (parts[0], "")
    };

    let kind_uuid_parts: Vec<&str> = kind_uuid_str.splitn(2, '-').collect();
    if kind_uuid_parts.len() != 2 {
        return Err(AstError::InvalidAnchorFormat);
    }

    let kind = match kind_uuid_parts[0] {
        "inline" => AnchorKind::Inline,
        "answer" => AnchorKind::Answer,
        s => return Err(AstError::InvalidAnchorKind(s.to_string())),
    };
    let uid = Uuid::parse_str(kind_uuid_parts[1]).map_err(AstError::InvalidUuid)?;
    let tag = match tag_str {
        "begin" => AnchorTag::Begin,
        "end" => AnchorTag::End,
        "" => AnchorTag::None,
        s => return Err(AstError::InvalidAnchorTag(s.to_string())),
    };

    Ok(Anchor { kind, uid, tag })
}

// Parses a tagged line (e.g., "@tag[param=value] arg1 arg2")
fn parse_tagged_line(line_str: &str) -> Result<LineKind, AstError> {
    let mut current_idx = 0;

    // Consume '@'
    if !line_str.starts_with('@') {
        return Err(AstError::InvalidTagFormat);
    }
    current_idx += 1;

    // Skip leading whitespace after '@'
    let after_at = &line_str[current_idx..];
    let tag_name_start_offset = after_at.find(|c: char| !c.is_whitespace())
        .unwrap_or(after_at.len());
    current_idx += tag_name_start_offset;

    let tag_name_and_rest = &line_str[current_idx..];
    let tag_name_end_offset = tag_name_and_rest.find(|c: char| c.is_whitespace() || c == '[')
        .unwrap_or(tag_name_and_rest.len());
    let tag_name = &tag_name_and_rest[..tag_name_end_offset];

    if tag_name.is_empty() {
        return Err(AstError::MissingTagName);
    }
    let tag = match tag_name {
        "include" => TagKind::Include,
        "inline" => TagKind::Inline,
        "answer" => TagKind::Answer,
        "summary" => TagKind::Summary,
        s => return Err(AstError::InvalidTagKind(s.to_string())),
    };
    current_idx += tag_name_end_offset;

    let mut parameters = HashMap::new();
    let mut arguments = Vec::new();

    let remaining = &line_str[current_idx..].trim_start();
    current_idx = 0; // Reset current_idx for remaining string

    // Parse parameters
    if remaining.starts_with('[') {
        let closing_bracket_idx = remaining[1..].find(']')
            .ok_or(AstError::InvalidParameterFormat)?;
        let params_str = &remaining[1..1 + closing_bracket_idx];
        parameters = parse_parameters(params_str)?;
        current_idx = 1 + closing_bracket_idx + 1; // Move past ']'
    }

    // Parse arguments
    let arguments_str = &remaining[current_idx..].trim_start();
    if !arguments_str.is_empty() {
        arguments = parse_arguments(arguments_str)?;
    }

    Ok(LineKind::Tagged(TaggedLine {
        tag,
        parameters,
        arguments,
    }))
}

// Parses the parameters string (e.g., "key1=value1;key2=value2")
fn parse_parameters(params_str: &str) -> Result<HashMap<String, String>, AstError> {
    let mut parameters = HashMap::new();
    for param_pair in params_str.split(';') {
        let trimmed_pair = param_pair.trim();
        if trimmed_pair.is_empty() {
            continue;
        }
        let parts: Vec<&str> = trimmed_pair.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(AstError::InvalidParameterFormat);
        }
        let key = parts[0].trim().to_string();
        let value = parts[1].trim().to_string();

        if key.is_empty() {
            return Err(AstError::EmptyParameterKey);
        }
        // Validate key format (Rust identifier like)
        if !key.chars().next().map_or(false, |c| c.is_alphabetic() || c == '_') ||
           !key.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(AstError::InvalidParameterKey(key));
        }

        // Handle quoted values
        let parsed_value = if value.starts_with('"') {
            parse_quoted_string(&value)?
        } else {
            // Validate unquoted value characters
            if !value.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '+' || c == '-' || c == '.' || c == '/') {
                // If it contains spaces or other special chars, it should have been quoted
                return Err(AstError::InvalidParameterFormat);
            }
            value
        };

        parameters.insert(key, parsed_value);
    }
    Ok(parameters)
}

// Parses the arguments string (e.g., "arg1 \"arg with space\" arg3")
fn parse_arguments(args_str: &str) -> Result<Vec<String>, AstError> {
    let mut arguments = Vec::new();
    let mut current_pos = 0;
    let chars: Vec<char> = args_str.chars().collect();

    while current_pos < chars.len() {
        // Skip whitespace
        while current_pos < chars.len() && chars[current_pos].is_whitespace() {
            current_pos += 1;
        }
        if current_pos == chars.len() {
            break; // End of string
        }

        if chars[current_pos] == '"' {
            // Quoted argument
            let _start_quote_pos = current_pos;
            current_pos += 1; // Move past opening quote
            let mut arg = String::new();
            let mut escaped = false;

            while current_pos < chars.len() {
                match chars[current_pos] {
                    '\\' if !escaped => {
                        escaped = true;
                    },
                    '"' if !escaped => {
                        arguments.push(arg);
                        current_pos += 1; // Move past closing quote
                        break;
                    },
                    c => {
                        arg.push(c);
                        escaped = false;
                    }
                }
                current_pos += 1;
            }
            if escaped { // If loop ended with an unclosed escape sequence
                return Err(AstError::UnclosedQuote);
            }
            if current_pos == chars.len() && chars[current_pos - 1] != '"' { // If string ended without closing quote
                return Err(AstError::UnclosedQuote);
            }

        } else {
            // Unquoted argument
            let start_arg_pos = current_pos;
            while current_pos < chars.len() && !chars[current_pos].is_whitespace() {
                current_pos += 1;
            }
            let arg = &args_str[start_arg_pos..current_pos];
            if !arg.is_empty() {
                arguments.push(arg.to_string());
            }
        }
    }
    Ok(arguments)
}
