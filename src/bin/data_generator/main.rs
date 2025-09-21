use anyhow::Result;
use reqwest::Client;
use handlebars::Handlebars;
use rand::seq::SliceRandom;
use clap::Parser;

mod cli_args;
mod ollama_client;
mod tool_definitions;
mod prompt_templates;
mod pipeline;

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli_args::CliArgs::parse();
    let client = Client::new();

    let mut handlebars = Handlebars::new();
    prompt_templates::register_all_templates(&mut handlebars)?;

    println!("--- Inizio Pipeline di Generazione Dati (Rust) ---");

    let tool_names: Vec<&str> = tool_definitions::TOOLS_DEFINITION.iter().map(|t| t.name).collect();
    let mut rng = rand::thread_rng();

    for i in 1..=args.num_examples {
        println!("\n========== Inizio Esempio {}/{} ==========", i, args.num_examples);

        let tool_to_practice = *tool_names.choose(&mut rng).unwrap();

        let student_prompt = match pipeline::generate_student_prompt(&client, &args, tool_to_practice, &handlebars).await {
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
