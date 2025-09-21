use anyhow::Result;
use reqwest::Client;
use handlebars::Handlebars;
use rand::seq::SliceRandom;
use clap::Parser;
use tracing_subscriber::{fmt, EnvFilter};
use tracing_subscriber::prelude::*;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use chrono::Local;
use std::path::PathBuf;

mod cli_args;
mod ollama_client;
mod tool_definitions;
mod prompt_templates;
mod pipeline;

const LOG_DIR: &str = ".vespe/log";

#[tokio::main]
async fn main() -> Result<()> {
    let file_name = format!("data_generator_{}.log", Local::now().format("%Y%m%d%H%M%S"));
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

    let client = Client::new();

    let mut handlebars = Handlebars::new();
    tracing::info!("Registering Handlebars templates...");
    prompt_templates::register_all_templates(&mut handlebars)?;
    tracing::info!("Handlebars templates registered.");

    tracing::info!("--- Inizio Pipeline di Generazione Dati (Rust) ---");

    let mut rng = rand::thread_rng();

    for i in 1..=args.num_examples {
        tracing::info!("========== Inizio Esempio {}/{} ==========", i, args.num_examples);

        let selected_tool = if let Some(tool_name_arg) = &args.tool_name {
            tool_definitions::TOOLS_DEFINITION.iter()
                .find(|t| t.name == tool_name_arg)
                .ok_or_else(|| anyhow::anyhow!("Tool '{}' not found", tool_name_arg))?
        } else {
            tool_definitions::TOOLS_DEFINITION.choose(&mut rng).unwrap()
        };

        let tool_name = selected_tool.name;
        let tool_description = selected_tool.description;
        let tool_spec_json = serde_json::to_string_pretty(&selected_tool.to_tool_spec())?;

        let use_case = args.use_case.as_deref().unwrap_or_else(|| pipeline::USE_CASES.choose(&mut rng).unwrap());
        let complexity = args.complexity.as_deref().unwrap_or_else(|| pipeline::COMPLEXITIES.choose(&mut rng).unwrap());
        let user_style = args.user_style.as_deref().unwrap_or_else(|| pipeline::USER_STYLES.choose(&mut rng).unwrap());
        let context_length = args.context_length.as_deref().unwrap_or_else(|| pipeline::CONTEXT_LENGTHS.choose(&mut rng).unwrap());

        tracing::debug!(
            "Selected parameters for example {}: tool_name={}, use_case={}, complexity={}, user_style={}, context_length={}",
            i, tool_name, use_case, complexity, user_style, context_length
        );

        tracing::info!("PASSO 1: Generating student prompt...");
        let student_prompt = match pipeline::generate_student_prompt(
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
            &handlebars
        ).await {
            Ok(prompt) => {
                tracing::debug!("PASSO 1: Student prompt generated successfully. Prompt: {}", prompt);
                prompt
            },
            Err(e) => {
                tracing::error!("ERRORE nel Passo 1: {}. Saltando l'esempio.", e);
                continue;
            }
        };

        tracing::info!("PASSO 2: Getting student response...");
        let (student_response, system_prompt_used) = match pipeline::get_student_response(
            &client,
            &args.ollama_url,
            &args.hero_model,
            &student_prompt,
            &args.tool_format,
            &handlebars
        ).await {
            Ok(res) => {
                tracing::debug!("PASSO 2: Student response received. Response: {}", res.0);
                res
            },
            Err(e) => {
                tracing::error!("ERRORE nel Passo 2: {}. Saltando l'esempio.", e);
                continue;
            }
        };

        tracing::info!("PASSO 3: Labeling student response...");
        let labeled_json = match pipeline::label_student_response(
            &client,
            &args.ollama_url,
            &args.marker_model,
            tool_name,
            tool_description,
            &tool_spec_json,
            &student_prompt,
            &student_response,
            &system_prompt_used,
            &handlebars
        ).await {
            Ok(json_str) => {
                let trimmed_json = json_str.trim().strip_prefix("```json").unwrap_or(&json_str).strip_suffix("```").unwrap_or(&json_str).trim().to_string();
                tracing::debug!("PASSO 3: Student response labeled. Labeled JSON: {}", trimmed_json);
                trimmed_json
            },
            Err(e) => {
                tracing::error!("ERRORE nel Passo 3: {}. Saltando l'esempio.", e);
                continue;
            }
        };

        let output_file_path = match pipeline::save_labeled_example(&args.output_dir, &labeled_json, i) {
            Ok(path) => path,
            Err(e) => {
                tracing::error!("ERRORE nel Passo 4: {}. Saltando l'esempio.", e);
                continue;
            }
        };
        tracing::info!("PASSO 4: Labeled example saved successfully to {}", output_file_path.display());

        tracing::info!("========== Fine Esempio {}/{} ==========", i, args.num_examples);
    }

    tracing::info!("\n--- Pipeline completata ---");
    Ok(())
}
