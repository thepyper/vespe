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
    let client = Client::new();

    let mut handlebars = Handlebars::new();
    prompt_templates::register_all_templates(&mut handlebars)?;

    println!("--- Inizio Pipeline di Generazione Dati (Rust) ---");

    let mut rng = rand::thread_rng();

    for i in 1..=args.num_examples {
        println!("\n========== Inizio Esempio {}/{} ==========", i, args.num_examples);

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

        let student_prompt = match pipeline::generate_student_prompt(
            &client,
            &args,
            tool_name,
            &tool_spec_json,
            tool_description,
            use_case,
            complexity,
            user_style,
            context_length,
            &handlebars
        ).await {
            Ok(prompt) => prompt,
            Err(e) => {
                eprintln!("ERRORE nel Passo 1: {}. Saltando l'esempio.", e);
                continue;
            }
        };

        let (student_response, system_prompt_used) = match pipeline::get_student_response(&client, &args, &student_prompt, &handlebars).await {
            Ok(res) => res,
            Err(e) => {
                eprintln!("ERRORE nel Passo 2: {}. Saltando l'esempio.", e);
                continue;
            }
        };

        let labeled_json = match pipeline::label_student_response(&client, &args, &student_prompt, &student_response, &system_prompt_used, &handlebars).await {
            Ok(json_str) => {
                json_str.trim().strip_prefix("```json").unwrap_or(&json_str).strip_suffix("```").unwrap_or(&json_str).trim().to_string()
            },
            Err(e) => {
                eprintln!("ERRORE nel Passo 3: {}. Saltando l'esempio.", e);
                continue;
            }
        };

        if let Err(e) = pipeline::save_labeled_example(&args.output_dir, &labeled_json, i) {
            eprintln!("ERRORE nel Passo 4: {}. Saltando l'esempio.", e);
            continue;
        }

        println!("========== Fine Esempio {}/{} ==========", i, args.num_examples);
    }

    println!("\n--- Pipeline completata ---");
    Ok(())
}
