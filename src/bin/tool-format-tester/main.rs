use anyhow::{anyhow, Result};
use chrono::Local;
use clap::Parser;
use handlebars::Handlebars;
use policies::{StructuredOutputBlock, ToolCallPolicy};
use crate::mcp_policy::McpPolicy;
use crate::tagged_policy::TaggedPolicy;
use crate::markdown_policy::MarkdownPolicy;
use rand::seq::SliceRandom;
use reqwest::Client;
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, EnvFilter};
use tracing_subscriber::prelude::*;

mod cli_args;
mod ollama_client;
mod pipeline;
mod policies;
mod mcp_policy;
mod tagged_policy;
mod prompt_templates;
mod tool_definitions;

const LOG_DIR: &str = ".vespe/log";

async fn save_test_result(
    output_dir: &PathBuf,
    narrator_prompt: &str,
    narrator_response: &str,
    hero_system_prompt: &str,
    hero_prompt: &str,
    hero_response: &str,
    validation_result: &Result<Vec<StructuredOutputBlock>, anyhow::Error>,
) -> Result<PathBuf> {
    let status = match validation_result {
        Ok(_) => "SUCCESS",
        Err(_) => "FAILURE",
    };
    let timestamp = Local::now().format("%Y%m%d%H%M%S%f").to_string();
    let file_path = output_dir.join(format!("test_{}_{}.json", timestamp, status));

    let result_data = json!({
        "status": status,
        "narrator_prompt": narrator_prompt,
        "narrator_response": narrator_response,
        "hero_system_prompt": hero_system_prompt,
        "hero_prompt": hero_prompt,
        "hero_response": hero_response,
        "validation_output": match validation_result {
            Ok(calls) => json!(calls),
            Err(e) => json!(e.to_string()),
        }
    });

    fs::create_dir_all(output_dir)?;
    fs::write(&file_path, serde_json::to_string_pretty(&result_data)?)?;
    Ok(file_path)
}

#[tokio::main]
async fn main() -> Result<()> {
    let file_name = format!("tool_format_tester_{}.log", Local::now().format("%Y%m%d%H%M%S"));
    let log_path = PathBuf::from(LOG_DIR).join(file_name);
    let file_appender = RollingFileAppender::new(Rotation::HOURLY, LOG_DIR, log_path);
    let (non_blocking_file_writer, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stdout).with_filter(EnvFilter::new("debug")))
        .with(fmt::layer().with_writer(non_blocking_file_writer).with_filter(EnvFilter::new("debug")))
        .init();

    tracing::info!("Tracing initialized successfully.");

    let args = cli_args::CliArgs::parse();
    tracing::info!("Parsed CLI arguments: {:?}", args);

    // --- Policy Setup ---
    let available_policies: Vec<Box<dyn ToolCallPolicy>> = vec![Box::new(McpPolicy), Box::new(TaggedPolicy), Box::new(MarkdownPolicy)];
    let selected_policy = available_policies
        .iter()
        .find(|p| p.name() == args.policy)
        .ok_or_else(|| anyhow!("Policy '{}' not found.", args.policy))?;
    tracing::info!("Selected policy: {}", selected_policy.name());

    let client = Client::new();

    let mut handlebars = Handlebars::new();
    tracing::info!("Registering Handlebars templates...");
    prompt_templates::register_all_templates(&mut handlebars)?;
    tracing::info!("Handlebars templates registered.");

    tracing::info!("--- Inizio Pipeline di Test Formato Tool ---");

    let mut rng = rand::thread_rng();

    for i in 1..=args.num_examples {
        tracing::info!("========== Inizio Esempio {}/{} ==========", i, args.num_examples);

        let selected_tool = if let Some(tool_name_arg) = &args.tool_name {
            tool_definitions::TOOLS_DEFINITION
                .iter()
                .find(|t| t.name == tool_name_arg)
                .ok_or_else(|| anyhow!("Tool '{}' not found", tool_name_arg))? 
        } else {
            tool_definitions::TOOLS_DEFINITION.choose(&mut rng).unwrap()
        };

        let tool_name = selected_tool.name;
        let tool_description = selected_tool.description;
        let tool_spec_json = serde_json::to_string_pretty(&selected_tool.to_tool_spec())?;

        // Generate all_tools_specs_json
        let all_tools_specs_json = tool_definitions::TOOLS_DEFINITION.iter()
            .map(|tool| serde_json::to_string_pretty(&tool.to_tool_spec()).unwrap_or_default())
            .collect::<Vec<String>>()
            .join("\n");

        let use_case = args.use_case.as_deref().unwrap_or_else(|| pipeline::USE_CASES.choose(&mut rng).unwrap());
        let complexity = args.complexity.as_deref().unwrap_or_else(|| pipeline::COMPLEXITIES.choose(&mut rng).unwrap());
        let user_style = args.user_style.as_deref().unwrap_or_else(|| pipeline::USER_STYLES.choose(&mut rng).unwrap());
        let context_length = args.context_length.as_deref().unwrap_or_else(|| pipeline::CONTEXT_LENGTHS.choose(&mut rng).unwrap());

        tracing::debug!(
            "Selected parameters for example {}: tool_name={}, use_case={}, complexity={}, user_style={}, context_length={}",
            i,
            tool_name,
            use_case,
            complexity,
            user_style,
            context_length
        );

        tracing::info!("PASSO 1: Generating prompt for HERO model...");
        let (narrator_response_raw, narrator_query) = match pipeline::generate_student_prompt(
            &client,
            &args.ollama_url,
            &args.narrator_model,
            tool_name,
            &tool_spec_json,
            tool_description,
            use_case,
            complexity,
            user_style,
            context_length,
            &handlebars,
        )
        .await
        {
            Ok((response, query)) => {
                tracing::debug!("PASSO 1: Prompt generated successfully: {}", response);
                (response, query)
            }
            Err(e) => {
                tracing::error!("ERROR in Step 1: {}. Skipping example.", e);
                continue;
            }
        };
        let student_prompt = narrator_response_raw.clone();

        tracing::info!("PASSO 2: Getting HERO response...");
        let (hero_response_raw, hero_system_prompt_used) = match pipeline::get_student_response(
            &client,
            &args.ollama_url,
            &args.hero_model,
            &student_prompt,
            selected_policy.as_ref(),
            &all_tools_specs_json,
            &handlebars,
        )
        .await
        {
            Ok(res) => {
                tracing::debug!("PASSO 2: HERO response received: {}", res.0);
                res
            }
            Err(e) => {
                tracing::error!("ERROR in Step 2: {}. Skipping example.", e);
                continue;
            }
        };

        tracing::info!("PASSO 3: Validating HERO response with '{}' policy...", selected_policy.name());
        let validation_result = selected_policy.validate_and_parse(&hero_response_raw);
        match &validation_result {
            Ok(parsed_calls) => {
                tracing::info!("Validation successful: {:?}", parsed_calls);
            }
            Err(e) => {
                tracing::error!("Validation FAILED: {}", e);
            }
        }

        if let Err(e) = save_test_result(&args.output_dir, &narrator_query, &narrator_response_raw, &hero_system_prompt_used, &student_prompt, &hero_response_raw, &validation_result).await {
            tracing::error!("Failed to save test result: {}", e);
        }

        tracing::info!("========== Fine Esempio {}/{} ==========", i, args.num_examples);
    }

    tracing::info!("\n--- Pipeline completata ---");
    Ok(())
}