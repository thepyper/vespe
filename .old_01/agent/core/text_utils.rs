pub fn trim_markdown_code_blocks(text: &str) -> &str {
    let mut trimmed = text.trim();
    if trimmed.starts_with("```json") {
        trimmed = trimmed.strip_prefix("```json").unwrap_or(trimmed);
        if let Some(end_pos) = trimmed.rfind("```") {
            trimmed = &trimmed[..end_pos];
        }
    } else if trimmed.starts_with("```") {
        trimmed = trimmed.strip_prefix("```").unwrap_or(trimmed);
        if let Some(end_pos) = trimmed.rfind("```") {
            trimmed = &trimmed[..end_pos];
        }
    }
    trimmed.trim()
}
