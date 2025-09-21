use anyhow::Result;
use reqwest::Client;
use handlebars::Handlebars;
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use chrono::Local;

use super::cli_args::CliArgs;
use super::ollama_client::query_ollama;
use super::tool_definitions::TOOLS_DEFINITION;
use super::prompt_templates::NORMATIVE_SYSTEM_PROMPT;


pub async fn generate_student_prompt(
    client: &Client,
    args: &CliArgs,
    tool_name: &str,
    handlebars: &Handlebars<'_>,
) -> Result<String> {
    let data = json!({ "tool_name": tool_name });
    let prompt = handlebars.render("meta_prompt", &data)?;
    query_ollama(client, &args.ollama_url, &args.big_model, &prompt, None).await
}

pub fn build_tool_spec(
    handlebars: &Handlebars<'_>,
    tool_format: &str,
) -> Result<String> {
    let template_name = format!("{}_spec", tool_format);
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
    Ok(spec)
}


pub async fn get_student_response(
    client: &Client,
    args: &CliArgs,
    student_prompt: &str,
    handlebars: &Handlebars<'_>,
) -> Result<(String, String)> {
    let tool_spec = build_tool_spec(handlebars, &args.tool_format)?;
    let system_prompt = format!("{}\n{}", NORMATIVE_SYSTEM_PROMPT, tool_spec);
    let response = query_ollama(client, &args.ollama_url, &args.small_model, student_prompt, Some(&system_prompt)).await?;
    Ok((response, system_prompt))
}

pub async fn label_student_response(
    client: &Client,
    args: &CliArgs,
    student_prompt: &str,
    student_response: &str,
    system_prompt_used: &str,
    handlebars: &Handlebars<'_>,
) -> Result<String> {
    let data = json!({
        "system_prompt_used": system_prompt_used,
        "student_prompt": student_prompt,
        "student_response": student_response
    });
    let prompt = handlebars.render("labeling_prompt", &data)?;
    query_ollama(client, &args.ollama_url, &args.big_model, &prompt, None).await
}

pub fn save_labeled_example(
    output_dir: &PathBuf,
    example_json_str: &str,
    _example_index: u32,
) -> Result<()> {
    fs::create_dir_all(output_dir)?;
    let timestamp = Local::now().format("%Y%m%d%H%M%S%f").to_string();
    let file_path = output_dir.join(format!("example_{}.json", timestamp));
    let parsed_json: serde_json::Value = serde_json::from_str(example_json_str)?;
    let formatted_json = serde_json::to_string_pretty(&parsed_json)?;
    fs::write(&file_path, formatted_json)?;
    println!("PASSO 4: Esempio salvato in '{}'", file_path.display());
    Ok(())
}
