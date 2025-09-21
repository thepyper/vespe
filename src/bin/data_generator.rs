use clap::Parser;
use rand::seq::SliceRandom;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use handlebars::Handlebars;
use serde_json::json;

// --- Data Structures ---

#[derive(Debug, Serialize, Clone)]
struct Tool {
    name: &'static str,
    description: &'static str,
    parameters: &'static [ToolParameter],
}

#[derive(Debug, Serialize, Clone)]
struct ToolParameter {
    name: &'static str,
    #[serde(rename = "type")]
    param_type: &'static str,
    description: &'static str,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct CliArgs {
    #[arg(long, default_value = "gpt-oss:20b")]
    big_model: String,
    #[arg(long, default_value = "gemma3:1b")]
    small_model: String,
    #[arg(long, default_value = "http://localhost:11434")]
    ollama_url: String,
    #[arg(long, default_value_t = 10)]
    num_examples: u32,
    #[arg(long, default_value = "buzz/training/generated_examples_rust")]
    output_dir: PathBuf,
    #[arg(long, value_parser = ["mcp", "json", "xml"], default_value = "mcp")]
    tool_format: String,
}

#[derive(Debug, Serialize)]
struct OllamaGenerateRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    system: Option<&'a str>,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct OllamaGenerateResponse {
    response: String,
}

const TOOLS_DEFINITION: &[Tool] = &[
    Tool {
        name: "read_file",
        description: "Legge e restituisce il contenuto di un file specificato.",
        parameters: &[
            ToolParameter {
                name: "absolute_path",
                param_type: "string",
                description: "Il percorso assoluto del file da leggere.",
            },
        ],
    },
    Tool {
        name: "write_file",
        description: "Scrive del contenuto in un file specificato, sovrascrivendolo se esiste.",
        parameters: &[
            ToolParameter {
                name: "file_path",
                param_type: "string",
                description: "Il percorso assoluto del file in cui scrivere.",
            },
            ToolParameter {
                name: "content",
                param_type: "string",
                description: "Il contenuto da scrivere nel file.",
            },
        ],
    },
];

const NORMATIVE_SYSTEM_PROMPT: &str = r#"#,
Sei un assistente AI progettato per aiutare gli utenti eseguendo task.
Per farlo, hai a disposizione una serie di tool.

Il tuo processo di pensiero deve seguire questi passi:
1.  **THOUGHT**: Analizza la richiesta dell'utente e decidi se puoi rispondere direttamente o se hai bisogno di un tool. Se scegli un tool, spiega quale e perché.
2.  **TOOL_CODE**: Se hai deciso di usare un tool, scrivi la chiamata al tool nel formato specificato. **DEVI FERMARTI SUBITO DOPO AVER CHIAMATO IL TOOL.** Non devi generare la risposta del tool (TOOL_RESPONSE) o continuare la conversazione.

Regole Assolute:
-   Usa **solo e soltanto** i tool elencati. Non inventare tool o parametri.
-   Rispetta **scrupolosamente** il formato di output richiesto per il TOOL_CODE.
-   Fermati **immediatamente** dopo aver prodotto il blocco TOOL_CODE.
"#;


// --- Pipeline Functions ---

async fn query_ollama(
    client: &Client,
    ollama_url: &str,
    model: &str,
    prompt: &str,
    system: Option<&str>,
) -> anyhow::Result<String> {
    let request_payload = OllamaGenerateRequest {
        model,
        prompt,
        system,
        stream: false,
    };
    let response = client
        .post(format!("{}/api/generate", ollama_url))
        .json(&request_payload)
        .send()
        .await?;
    let response_body = response.json::<OllamaGenerateResponse>().await?;
    Ok(response_body.response.trim().to_string())
}

async fn generate_student_prompt(
    client: &Client,
    args: &CliArgs,
    tool_name: &str,
    handlebars: &Handlebars<'_>,
) -> anyhow::Result<String> {
    let data = json!({ "tool_name": tool_name });
    let prompt = handlebars.render("meta_prompt", &data)?;
    query_ollama(client, &args.ollama_url, &args.big_model, &prompt, None).await
}

fn build_tool_spec(
    handlebars: &Handlebars<'_>,
    tool_format: &str,
) -> anyhow::Result<String> {
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


async fn get_student_response(
    client: &Client,
    args: &CliArgs,
    student_prompt: &str,
    handlebars: &Handlebars<'_>,
) -> anyhow::Result<(String, String)> {
    let tool_spec = build_tool_spec(handlebars, &args.tool_format)?;
    let system_prompt = format!("{}
{}", NORMATIVE_SYSTEM_PROMPT, tool_spec);
    let response = query_ollama(client, &args.ollama_url, &args.small_model, student_prompt, Some(&system_prompt)).await?;
    Ok((response, system_prompt))
}

async fn label_student_response(
    client: &Client,
    args: &CliArgs,
    student_prompt: &str,
    student_response: &str,
    system_prompt_used: &str,
    handlebars: &Handlebars<'_>,
) -> anyhow::Result<String> {
    let data = json!({
        "system_prompt_used": system_prompt_used,
        "student_prompt": student_prompt,
        "student_response": student_response
    });
    let prompt = handlebars.render("labeling_prompt", &data)?;
    query_ollama(client, &args.ollama_url, &args.big_model, &prompt, None).await
}

fn save_labeled_example(
    output_dir: &PathBuf,
    example_json_str: &str,
    example_index: u32,
) -> anyhow::Result<()> {
    fs::create_dir_all(output_dir)?;
    let file_path = output_dir.join(format!("example_{:04}.json", example_index));
    let parsed_json: serde_json::Value = serde_json::from_str(example_json_str)?;
    let formatted_json = serde_json::to_string_pretty(&parsed_json)?;
    fs::write(&file_path, formatted_json)?;
    println!("PASSO 4: Esempio salvato in '{}'", file_path.display());
    Ok(())
}

// --- Main Function ---

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse();
    let client = Client::new();

    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("meta_prompt", include_str!("data_generator_templates/meta_prompt.hbs"))?;
    handlebars.register_template_string("labeling_prompt", include_str!("data_generator_templates/labeling_prompt.hbs"))?;
    handlebars.register_template_string("json_spec", include_str!("data_generator_templates/json_spec.hbs"))?;
    handlebars.register_template_string("mcp_spec", include_str!("data_generator_templates/mcp_spec.hbs"))?;
    handlebars.register_template_string("xml_spec", include_str!("data_generator_templates/xml_spec.hbs"))?;

    println!("--- Inizio Pipeline di Generazione Dati (Rust) ---");

    let tool_names: Vec<&str> = TOOLS_DEFINITION.iter().map(|t| t.name).collect();
    let mut rng = rand::thread_rng();

    for i in 1..=args.num_examples {
        println!("\n========== Inizio Esempio {}/{} ==========", i, args.num_examples);
        
        let tool_to_practice = *tool_names.choose(&mut rng).unwrap();

        let student_prompt = match generate_student_prompt(&client, &args, tool_to_practice, &handlebars).await {
            Ok(prompt) => prompt,
            Err(e) => {
                eprintln!("ERRORE nel Passo 1: {}. Saltando l'esempio.", e);
                continue;
            }
        };

        let (student_response, system_prompt_used) = match get_student_response(&client, &args, &student_prompt, &handlebars).await {
            Ok(res) => res,
            Err(e) => {
                eprintln!("ERRORE nel Passo 2: {}. Saltando l'esempio.", e);
                continue;
            }
        };

        let labeled_json = match label_student_response(&client, &args, &student_prompt, &student_response, &system_prompt_used, &handlebars).await {
            Ok(json_str) => {
                json_str.trim().strip_prefix("```json").unwrap_or(&json_str).strip_suffix("```").unwrap_or(&json_str).trim().to_string()
            },
            Err(e) => {
                eprintln!("ERRORE nel Passo 3: {}. Saltando l'esempio.", e);
                continue;
            }
        };

        if let Err(e) = save_labeled_example(&args.output_dir, &labeled_json, i) {
            eprintln!("ERRORE nel Passo 4: {}. Saltando l'esempio.", e);
            continue;
        }

        println!("========== Fine Esempio {}/{} ==========", i, args.num_examples);
    }

    println!("\n--- Pipeline completata ---");
    Ok(())
}