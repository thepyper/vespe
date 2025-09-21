use anyhow::Result;
use reqwest::Client;
use handlebars::Handlebars;
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use chrono::Local;

use super::cli_args::CliArgs;
use super::ollama_client::query_ollama;
use super::tool_definitions::{TOOLS_DEFINITION};
use super::prompt_templates::NORMATIVE_SYSTEM_PROMPT;

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
    args: &CliArgs,
    tool_name: &str,
    tool_spec_json: &str,
    tool_description: &str,
    use_case: &str,
    complexity: &str,
    user_style: &str,
    context_length: &str,
    handlebars: &Handlebars<'_>,
) -> Result<String> {
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
    query_ollama(client, &args.ollama_url, &args.big_model, &prompt, None).await
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
    args: &CliArgs,
    student_prompt: &str,
    handlebars: &Handlebars<'_>,
) -> Result<(String, String)> {
    let tool_spec = build_tool_spec(handlebars, &args.tool_format)?;
    tracing::debug!("get_student_response: Built tool spec: {}", tool_spec);
    let system_prompt = format!("{}\n{}", NORMATIVE_SYSTEM_PROMPT, tool_spec);
    tracing::debug!("get_student_response: Constructed system prompt: {}", system_prompt);
    let response = query_ollama(client, &args.ollama_url, &args.small_model, student_prompt, Some(&system_prompt)).await?;
    Ok((response, system_prompt))
}

pub async fn label_student_response(
    client: &Client,
    args: &CliArgs,
    tool_name: &str,
    tool_description: &str,
    tool_spec_json: &str,
    small_query: &str,
    small_reply: &str,
    system_prompt_used: &str,
    handlebars: &Handlebars<'_>,
) -> Result<String> {
    let data = json!({
        "system_prompt_used": system_prompt_used,
        "small_query": small_query,
        "small_reply": small_reply,
        "tool_name": tool_name,
        "tool_description": tool_description,
        "tool_spec": tool_spec_json
    });
    tracing::debug!("label_student_response: Data for rendering: {:#?}", data);
    let prompt = handlebars.render("labeling_prompt", &data)?;
    tracing::debug!("label_student_response: Rendered prompt: {}", prompt);
    query_ollama(client, &args.ollama_url, &args.big_model, &prompt, None).await
}

pub fn save_labeled_example(
    output_dir: &PathBuf,
    example_json_str: &str,
    _example_index: u32,
) -> Result<PathBuf> {
    fs::create_dir_all(output_dir)?;
    let timestamp = Local::now().format("%Y%m%d%H%M%S%f").to_string();
    let file_path = output_dir.join(format!("example_{}.json", timestamp));
    let parsed_json: serde_json::Value = serde_json::from_str(example_json_str)?;
    let formatted_json = serde_json::to_string_pretty(&parsed_json)?;
    fs::write(&file_path, formatted_json)?;
    tracing::info!("Esempio salvato in '{}'", file_path.display());
    Ok(file_path)
}
