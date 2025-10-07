use super::types::*;
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

pub fn format_document(lines: &Vec<Line>) -> String {
    lines
        .iter()
        .map(|line| line.to_string())
        .collect::<Vec<String>>()
        .join("\n")
}

pub fn parse_document(input: &str) -> Result<Vec<Line>, String> {
    input
        .lines()
        .enumerate()
        .map(|(line_num, line_str)| {
            if line_str.trim().is_empty() {
                Ok(Line::Text(line_str.to_string()))
            } else {
                parse_line(line_str).map_err(|e| format!("Error on line {}: {}", line_num + 1, e))
            }
        })
        .collect::<Result<Vec<Line>, String>>()
}

pub fn parse_line(input: &str) -> Result<Line, String> {
    let (content_str, anchor_opt) = parse_anchor(input)?;

    if let Some(anchor) = anchor_opt {
        return Ok(Line::Anchor(anchor));
    }

    if content_str.trim_start().starts_with('@') {
        let tagged_line_kind = parse_tagged_line(&content_str)?;
        match tagged_line_kind {
            LineKind::Tagged { tag, parameters, arguments } => {
                return Ok(Line::Tagged {
                    tag,
                    parameters,
                    arguments,
                });
            }
            LineKind::Text(s) => {
                // If parse_tagged_line returned Text, it means it wasn't a valid tagged line
                return Ok(Line::Text(s));
            }
        }
    }

    Ok(Line::Text(content_str.to_string()))
}

fn parse_anchor(input: &str) -> Result<(String, Option<Anchor>), String> {
    let anchor_start = input.rfind("<!--");
    if let Some(start_idx) = anchor_start {
        let anchor_str = &input[start_idx..];
        if anchor_str.ends_with("-->") {
            let inner_anchor = anchor_str[4..anchor_str.len() - 3].trim();
            let parts: Vec<&str> = inner_anchor.splitn(2, '-').collect();
            if parts.len() != 2 {
                // Invalid format, treat as no anchor
                return Ok((input.to_string(), None));
            }
            let kind_str = parts[0];
            let uuid_and_tag_str = parts[1];

            let uuid_parts: Vec<&str> = uuid_and_tag_str.splitn(2, ':').collect();
            let uuid_str = uuid_parts[0];
            let tag_str = if uuid_parts.len() == 2 {
                uuid_parts[1]
            } else {
                ""
            };

            let kind = match AnchorKind::from_str(kind_str) {
                Ok(k) => k,
                Err(_) => return Ok((input.to_string(), None)), // Unknown kind, treat as no anchor
            };
            let tag = match AnchorTag::from_str(tag_str) {
                Ok(t) => t,
                Err(_) => return Ok((input.to_string(), None)), // Unknown tag, treat as no anchor
            };
            let uid = Uuid::parse_str(uuid_str).map_err(|e| e.to_string())?;

            let content_before_anchor = input[..start_idx].trim_end().to_string();
            return Ok((content_before_anchor, Some(Anchor { kind, uid, tag })))
        }
    }
    Ok((input.to_string(), None))
}

fn parse_tagged_line(input: &str) -> Result<LineKind, String> {
    let trimmed_input = input.trim_start();
    if !trimmed_input.starts_with('@') {
        return Err("Tagged line must start with '@'\n".to_string());
    }

    let mut chars = trimmed_input.chars().skip(1).peekable();
    let mut tag_name = String::new();
    let mut tag_name_len = 0;

    while let Some(c) = chars.next() {
        if c.is_alphanumeric() || c == '_' {
            tag_name.push(c);
            tag_name_len += 1;
        } else {
            // If the next char is not alphanumeric or underscore, it's the end of the tag name
            // It must be a '[' or a whitespace for a valid tag. If not, it's an invalid tag format.
            if c != ' ' && c != '[' {
                // Invalid character after tag name, treat as Text
                return Ok(LineKind::Text(input.to_string()));
            }
            break;
        }
    }

    let tag = match TagKind::from_str(&tag_name) {
        Ok(kind) => kind,
        Err(_) => return Ok(LineKind::Text(input.to_string())),
    };

    let remaining_after_tag = &trimmed_input[1 + tag_name_len..];
    let mut parameters = HashMap::new();
    let mut arguments = Vec::new();

    let mut current_pos = 0;

    // Check for parameters
    // No whitespace allowed between @tag and [parameters]
    if remaining_after_tag.starts_with("[") {
        let param_end = remaining_after_tag
            .find("]")
            .ok_or("Missing closing ']'
 for parameters".to_string())?;
        let param_str = &remaining_after_tag[1..param_end];
        parameters = parse_parameters(param_str)?;
        current_pos = param_end + 1;
    }

    let arg_str = remaining_after_tag[current_pos..].trim_start();
    if !arg_str.is_empty() {
        arguments = parse_arguments(arg_str)?;
    }

    Ok(LineKind::Tagged {
        tag,
        parameters,
        arguments,
    })
}

fn parse_parameters(input: &str) -> Result<HashMap<String, String>, String> {
    let mut params = HashMap::new();
    if input.is_empty() {
        return Ok(params);
    }

    for pair_str in input.split(';') {
        let trimmed_pair = pair_str.trim();
        if trimmed_pair.is_empty() {
            continue;
        }
        let parts: Vec<&str> = trimmed_pair.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid parameter format: {}", trimmed_pair));
        }
        let key = parts[0].trim().to_string();
        let value = parts[1].trim().to_string();

        // Validate key format: [a-zA-Z_][a-zA-Z0-9_]*
        if !key
            .chars()
            .next()
            .map_or(false, |c| c.is_alphabetic() || c == '_')
            || !key.chars().all(|c| c.is_alphanumeric() || c == '_')
        {
            return Err(format!("Invalid parameter key format: {}", key));
        }

        // Validate value format: [a-zA-Z0-9_+\-./]* (single token, no spaces)
        if !value.chars().all(|c| {
            c.is_alphanumeric() || c == '_' || c == '+' || c == '-' || c == '.' || c == '/'
        }) {
            return Err(format!("Invalid parameter value format: {}", value));
        }

        params.insert(key, value);
    }
    Ok(params)
}

fn parse_arguments(input: &str) -> Result<Vec<String>, String> {
    let mut args = Vec::new();
    let mut in_quote = false;
    let mut current_arg = String::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                if let Some(next_c) = chars.peek() {
                    if *next_c == '"' {
                        current_arg.push('"');
                        chars.next(); // Consume the escaped quote
                    } else {
                        current_arg.push(c); // Keep the backslash if not escaping a quote
                    }
                } else {
                    current_arg.push(c);
                }
            }
            '"' => {
                in_quote = !in_quote;
                if !in_quote && !current_arg.is_empty() {
                    args.push(current_arg.clone());
                    current_arg.clear();
                }
            }
            ' ' => {
                if in_quote {
                    current_arg.push(c);
                } else if !current_arg.is_empty() {
                    args.push(current_arg.clone());
                    current_arg.clear();
                }
            }
            _ => {
                current_arg.push(c);
            }
        }
    }

    if !current_arg.is_empty() {
        args.push(current_arg);
    }

    if in_quote {
        return Err("Unclosed quote in arguments".to_string());
    }

    Ok(args)
}