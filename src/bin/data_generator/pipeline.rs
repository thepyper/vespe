use anyhow::{anyhow, Result};
use reqwest::Client;
use handlebars::Handlebars;
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use chrono::Local;

use super::cli_args::CliArgs;
use super::ollama_client::query_ollama;
use super::tool_definitions::{TOOLS_DEFINITION};

pub const USE_CASES: &[&str] = &[
    "data extraction",
    "code generation",
    "summarization",
    "question answering",
    "text transformation",
];

pub const COMPLEXITIES: &[&str] = &[
    "simple",
    "medium",
    "complex",
    "very complex",
];

pub const USER_STYLES: &[&str] = &[
    "formal",
    "informal",
    "technical",
    "casual",
];

pub const CONTEXT_LENGTHS: &[&str] = &[
    "short",
    "medium",
    "long",
    "very long",
];

pub async fn generate_student_prompt(
    client: &Client,
    ollama_url: &str,
    narrator_model: &str,
    tool_name: &str,
    tool_spec_json: &str,
    tool_description: &str,
    use_case: &str,
    complexity: &str,
    user_style: &str,
    context_length: &str,
    handlebars: &Handlebars<'_>,
) -> Result<(String, String)> {
    let data = json!({
        "tool_name": tool_name,
        "tool_spec": tool_spec_json,
        "tool_description": tool_description,
        "use_case": use_case,
        "complexity": complexity,
        "user_style": user_style,
        "context_length": context_length
    });
    tracing::debug!("generate_student_prompt: Data for rendering: {:#?}", data);
    let prompt = handlebars.render("meta_prompt", &data)?;
    tracing::debug!("generate_student_prompt: Rendered prompt: {}", prompt);
    let response = query_ollama(client, ollama_url, narrator_model, &prompt, None).await?;
    Ok((response, prompt))
}

pub fn build_tool_spec(
    handlebars: &Handlebars<'_>,
    tool_format: &str,
) -> Result<String> {
    let template_name = format!("{}_spec", tool_format);
    tracing::debug!("build_tool_spec: Using template name: {}", template_name);
    let mcp_example = json!({
        "role": "assistant",
        "content": [{
            "type": "tool_use",
            "id": "call_1",
            "name": "read_file",
            "input": { "absolute_path": "/path/to/file.txt" }
        }]
    });
    let json_example = json!({
        "tool_name": "<nome del tool>",
        "parameters": { "<nome parametro>": "<valore>" }
    });

    let data = json!({
        "tools": TOOLS_DEFINITION,
        "tools_json": serde_json::to_string_pretty(TOOLS_DEFINITION)?,
        "json_example": serde_json::to_string_pretty(&json_example)?,
        "mcp_json_example": serde_json::to_string_pretty(&mcp_example)?
    });
    let spec = handlebars.render(&template_name, &data)?;
    tracing::debug!("build_tool_spec: Generated tool spec: {}", spec);
    Ok(spec)
}


pub async fn get_student_response(
    client: &Client,
    ollama_url: &str,
    hero_model: &str,
    student_prompt: &str,
    tool_format: &str,
    handlebars: &Handlebars<'_>,
) -> Result<(String, String)> {
    let tool_spec = build_tool_spec(handlebars, tool_format)?;
    tracing::debug!("get_student_response: Built tool spec: {}", tool_spec);
    let data = json!({
        "tool_spec": tool_spec,
    });    
    tracing::debug!("get_student_response: Data for rendering: {:#?}", data);
    let system_prompt = handlebars.render("system_prompt", &data)?;
    tracing::debug!("get_student_response: Rendered system prompt: {}", system_prompt);
    let response = query_ollama(client, ollama_url, hero_model, student_prompt, Some(&system_prompt)).await?;
    Ok((response, system_prompt))
}

pub async fn label_student_response(
    client: &Client,
    ollama_url: &str,
    marker_model: &str,
    tool_name: &str,
    tool_description: &str,
    tool_spec_json: &str,
    small_query: &str,
    small_reply: &str,
    system_prompt_used: &str,
    all_tools_specs: &str,
    handlebars: &Handlebars<'_>,
) -> Result<(String, String)> {
    let data = json!({
        "system_prompt_used": system_prompt_used,
        "small_query": small_query,
        "small_reply": small_reply,
        "tool_name": tool_name,
        "tool_description": tool_description,
        "tool_spec": tool_spec_json,
        "tools_specs": all_tools_specs,
    });
    tracing::debug!("label_student_response: Data for rendering: {:#?}", data);
    let prompt = handlebars.render("labeling_prompt", &data)?;
    tracing::debug!("label_student_response: Rendered prompt: {}", prompt);
    let response = query_ollama(client, ollama_url, marker_model, &prompt, None).await?;
    Ok((response, prompt))
}

fn segmentation_to_json_conversion(input: &str) -> Result<String> {

    let mut full_text = String::new();
    let mut spans = Vec::new();
    let mut pos = 0;

    for (line_no, line) in input.lines().enumerate() {
        
        // Allow empty lines
        if line.is_empty() {
            continue;
        }
        
        if line == "<NL>" {
            full_text.push('\n');
            pos += 1;
            continue;
        }

        // Controllo che inizi con '<'
        if !line.starts_with('<') {
            return Err(anyhow!("Formato invalido alla riga {}: manca '<'", line_no + 1));
        }

        // Deve contenere almeno un '>'
        let (category, segment) = match line.find('>') {
            Some(idx) if idx > 1 => {
                let category = &line[1..idx];
                let segment = &line[idx + 1..];
                if category.is_empty() || segment.is_empty() {
                    return Err(anyhow!("Formato invalido alla riga {}", line_no + 1));
                }
                (category.to_string(), segment)
            }
            _ => return Err(anyhow!("Formato invalido alla riga {}", line_no + 1)),
        };

        let start = pos;
        full_text.push_str(segment);
        pos += segment.chars().count();
        let end = pos;

        spans.push(json!({
            "start": start,
            "end": end,
            "category": category
        }));
    }

    let result = json!({
        "full_text": full_text,
        "spans": spans
    });

    Ok(serde_json::to_string_pretty(&result)?)
}

pub async fn save_labeled_example(
    output_dir: &PathBuf,
    labeled_json_str: &str,
    _example_index: u32,
    narrator_query: &str,
    narrator_response: &str,
    hero_query: &str,
    hero_response: &str,
    marker_query: &str,
    marker_response: &str,
    config: &CliArgs,
) -> Result<PathBuf> {
    fs::create_dir_all(output_dir)?;
    let timestamp = Local::now().format("%Y%m%d%H%M%S%f").to_string();
    let file_path = output_dir.join(format!("example_{}.json", timestamp));

    tracing::debug!("save_labeled_example: Segmented response: {}", labeled_json_str);    
    let converted_json = segmentation_to_json_conversion(labeled_json_str)?;
    tracing::debug!("save_labeled_example: Segmented json conversion: {}", converted_json);
    
    let mut original_labeled_json: serde_json::Value = serde_json::from_str(&converted_json)?;

    let mut debug_json = json!({ "debug": {
        "narrator_query": narrator_query,
        "narrator_response": narrator_response,
        "hero_query": hero_query,
        "hero_response": hero_response,
        "marker_query": marker_query,
        "marker_response": marker_response,
        "config": serde_json::to_value(config)?,
    }});
    
    //original_labeled_json.as_object_mut().ok_or(anyhow!("internal error"))?["debug"] = debug_json;
    let _ = original_labeled_json.as_object_mut().insert(debug_json.as_object_mut().expect("internal error 001"));

    let formatted_json = serde_json::to_string_pretty(&original_labeled_json)?;
    fs::write(&file_path, formatted_json)?;
    tracing::info!("Esempio salvato in '{}'", file_path.display());
    Ok(file_path)
}
