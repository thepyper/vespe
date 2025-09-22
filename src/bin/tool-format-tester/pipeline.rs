use anyhow::Result;
use reqwest::Client;
use handlebars::Handlebars;
use serde_json::json;

use super::ollama_client::query_ollama;
use super::policies::ToolCallPolicy;

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
    tracing::info!("Narrator Query: {}", prompt);
    let response = query_ollama(client, ollama_url, narrator_model, &prompt, None).await?;
    Ok((response, prompt))
}

pub async fn get_student_response(
    client: &Client,
    ollama_url: &str,
    hero_model: &str,
    student_prompt: &str,
    policy: &dyn ToolCallPolicy,
    all_tools_specs_json: &str,
    handlebars: &Handlebars<'_>,
) -> Result<(String, String)> {
    let format_specs_content = policy.build_prompt_section(handlebars)?;
    tracing::debug!("get_student_response: Built format specs using '{}' policy", policy.name());
    let data = json!({
        "tool_spec": all_tools_specs_json,
        "format_specs": format_specs_content,
	"student_prompt": student_prompt,
    });    
    tracing::debug!("get_student_response: Data for rendering: {:#?}", data);
    let prompt = handlebars.render("system_prompt", &data)?;
    tracing::info!("Hero System Prompt: {}", prompt);
    tracing::info!("Hero User Prompt: {}", prompt);
    let response = query_ollama(client, ollama_url, hero_model, &prompt, Some(&prompt)).await?;
    Ok((response, prompt))
}