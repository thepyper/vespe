use clap::Parser;
use rand::seq::SliceRandom;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// --- Strutture Dati per l'Applicazione ---

/// Definizione di un tool che l'assistente può usare.
#[derive(Debug, Serialize, Clone)]
struct Tool {
    name: &'static str,
    description: &'static str,
    parameters: &'static [ToolParameter],
}

/// Definizione di un parametro per un tool.
#[derive(Debug, Serialize, Clone)]
struct ToolParameter {
    name: &'static str,
    #[serde(rename = "type")]
    param_type: &'static str,
    description: &'static str,
}

/// Argomenti da riga di comando parsati da clap.
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

/// Struttura per la richiesta all'API di Ollama.
#[derive(Debug, Serialize)]
struct OllamaGenerateRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    system: Option<&'a str>,
    stream: bool,
}

/// Struttura per la risposta dall'API di Ollama.
#[derive(Debug, Deserialize)]
struct OllamaGenerateResponse {
    response: String,
}

/// Struttura per l'esempio etichettato da salvare in JSON.
#[derive(Debug, Serialize)]
struct LabeledExample<'a> {
    full_text: &'a str,
    spans: Vec<Span>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Span {
    label: String,
    start: usize,
    end: usize,
}

// --- Costanti e Definizioni Statiche ---

const NORMATIVE_SYSTEM_PROMPT: &str = r#""
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

const TOOLS_DEFINITION: &[Tool] = &[
    Tool {
        name: "read_file",
        description: "Legge e restituisce il contenuto di un file specificato.",
        parameters: &[ToolParameter {
            name: "absolute_path",
            param_type: "string",
            description: "Il percorso assoluto del file da leggere.",
        }],
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

// --- Funzioni per Costruire i System Prompt ---

fn build_mcp_tool_spec(tools: &[Tool]) -> String {
    let mut spec = "I tool disponibili sono:\n".to_string();
    for tool in tools {
        spec.push_str(&format!("- {}: {}\\n", tool.name, tool.description));
    }

    let example = serde_json::json!({
        "role": "assistant",
        "content": [{
            "type": "tool_use",
            "id": "call_1",
            "name": "read_file",
            "input": { "absolute_path": "/path/to/file.txt" }
        }]
    });

    spec.push_str("\nIl blocco TOOL_CODE deve essere un singolo oggetto JSON che segue la specifica 'Model Context Protocol'.\n");
    spec.push_str("Ogni chiamata a tool deve essere un oggetto JSON con `type: 'tool_use'`.
");
    spec.push_str("Ecco un esempio di chiamata singola:\n");
    spec.push_str("```json\n");
    spec.push_str(&serde_json::to_string_pretty(&example).unwrap());
    spec.push_str("\n```\n");
    spec.push_str("\nRegole per il formato MCP:\n");
    spec.push_str("- L'output deve essere un JSON valido.\n");
    spec.push_str("- `role` deve essere `assistant`.\n");
    spec.push_str("- `content` è una lista che contiene oggetti di tipo `tool_use`.\n");
    spec.push_str("- Ogni `tool_use` deve avere un `id` univoco e semplice (es. 'call_1'), il `name` del tool, e un oggetto `input` con i parametri.");

    spec
}

fn build_json_tool_spec(tools: &[Tool]) -> String {
    let mut spec = "Definizione dei tool in formato JSON:\n".to_string();
    spec.push_str(&serde_json::to_string_pretty(tools).unwrap());
    
    let example = serde_json::json!({
        "tool_name": "<nome del tool>",
        "parameters": {
            "<nome parametro>": "<valore>"
        }
    });

    spec.push_str("\n\nFormato di output per TOOL_CODE (JSON):\n");
    spec.push_str("```json\n");
    spec.push_str(&serde_json::to_string_pretty(&example).unwrap());
    spec.push_str("\n```\n");
    
    spec
}

fn build_xml_tool_spec(tools: &[Tool]) -> String {
    let mut spec = "<tools>\n".to_string();
    for tool in tools {
        spec.push_str(&format!("  <tool name=\"{}\" description=\"{}|\" >\n", tool.name, tool.description));
        for param in tool.parameters {
            spec.push_str(&format!("    <parameter name=\"{}|\" type=\"{}|\" description=\"{}|\" />\n", param.name, param.param_type, param.description));
        }
        spec.push_str("  </tool>\n");
    }
    spec.push_str("</tools>\n\n");
    spec.push_str("Formato di output per TOOL_CODE (XML):\n");
    spec.push_str("<tool_call>\n");
    spec.push_str("  <tool_name>nome_del_tool</tool_name>\n");
    spec.push_str("  <parameters>\n");
    spec.push_str("    <param_name>valore</param_name>\n");
    spec.push_str("  </parameters>\n");
    spec.push_str("</tool_call>\n");

    spec
}

// --- Funzioni della Pipeline ---

async fn query_ollama(
    client: &Client,
    ollama_url: &str,
    model: &str,
    prompt: &str,
    system: Option<&str>,
) -> anyhow::Result<String> {
    println!("\n---\nQuery al modello: {}", model);
    if let Some(sys) = system {
        println!("System Prompt:\n{}...", &sys[..std::cmp::min(sys.len(), 400)]);
    }

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
) -> anyhow::Result<String> {
    println!("PASSO 1: Il modello '{}' genera un prompt per il tool '{}'...", &args.big_model, tool_name);
    let meta_prompt = format!(
        "Sei un generatore di prompt per un assistente AI. Il tuo compito è creare un prompt utente che richieda all'assistente di usare un tool specifico. Il prompt deve essere in linguaggio naturale e non deve contenere chiamate dirette a tool.\nGenera un prompt che richieda l'uso del tool '{}'.\nOutput solo il prompt utente.",
        tool_name
    );

    query_ollama(client, &args.ollama_url, &args.big_model, &meta_prompt, None).await
}

async fn get_student_response(
    client: &Client,
    args: &CliArgs,
    student_prompt: &str,
) -> anyhow::Result<(String, String)> {
    println!("PASSO 2: Il modello '{}' risponde (formato: {})...", &args.small_model, &args.tool_format);
    
    let tool_spec_builder = match args.tool_format.as_str() {
        "mcp" => build_mcp_tool_spec,
        "json" => build_json_tool_spec,
        "xml" => build_xml_tool_spec,
        _ => unreachable!(),
    };
    let tool_specification = tool_spec_builder(TOOLS_DEFINITION);
    let system_prompt = format!("{}
{}", NORMATIVE_SYSTEM_PROMPT, tool_specification);

    let response = query_ollama(client, &args.ollama_url, &args.small_model, student_prompt, Some(&system_prompt)).await?;
    Ok((response, system_prompt))
}

async fn label_student_response(
    client: &Client,
    args: &CliArgs,
    student_prompt: &str,
    student_response: &str,
    system_prompt_used: &str,
) -> anyhow::Result<String> {
    println!("PASSO 3: Il modello '{}' etichetta la risposta...", &args.big_model);
    let labeling_prompt = format!(
        "Sei un etichettatore di output di assistenti AI. Il tuo compito è segmentare la risposta dell'assistente nelle categorie funzionali: THOUGHT, TOOL_CODE, TEXT.\nFornisci l'etichetta e le posizioni esatte (start/end) in formato JSON.\n\n--- CONTESTO FORNITO ALL'ASSISTENTE ---\nSystem Prompt:\n{}\n\n--- TASK DI ETICHETTATURA ---\nPrompt Utente:\n{}\n\nRisposta Assistente AI:\n{}\n\n--- OUTPUT RICHIESTO ---\nGenera un oggetto JSON con chiavi 'full_text' e 'spans'.\nOutput JSON:",
        system_prompt_used, student_prompt, student_response
    );

    query_ollama(client, &args.ollama_url, &args.big_model, &labeling_prompt, None).await
}

fn save_labeled_example(
    output_dir: &PathBuf,
    example_json_str: &str,
    example_index: u32,
) -> anyhow::Result<()> {
    fs::create_dir_all(output_dir)?;
    let file_path = output_dir.join(format!("example_{:04}.json", example_index));
    
    // Re-parse to format it nicely
    let parsed_json: serde_json::Value = serde_json::from_str(example_json_str)?;
    let formatted_json = serde_json::to_string_pretty(&parsed_json)?;

    fs::write(&file_path, formatted_json)?;
    println!("PASSO 4: Esempio salvato in '{}'", file_path.display());
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse();
    let client = Client::new();

    println!("--- Inizio Pipeline di Generazione Dati (Rust) ---");
    println!("Modello Insegnante: {}", &args.big_model);
    println!("Modello Studente:   {}", &args.small_model);
    println!("Formato Tool:       {}", &args.tool_format);
    println!("-------------------------------------------------");

    let tool_names: Vec<&str> = TOOLS_DEFINITION.iter().map(|t| t.name).collect();
    let mut rng = rand::thread_rng();

    for i in 1..=args.num_examples {
        println!("\n========== Inizio Esempio {}/{} ==========", i, args.num_examples);
        
        let tool_to_practice = *tool_names.choose(&mut rng).unwrap();

        let student_prompt = match generate_student_prompt(&client, &args, tool_to_practice).await {
            Ok(prompt) => prompt,
            Err(e) => {
                eprintln!("ERRORE nel Passo 1: {}. Saltando l'esempio.", e);
                continue;
            }
        };

        let (student_response, system_prompt_used) = match get_student_response(&client, &args, &student_prompt).await {
            Ok(res) => res,
            Err(e) => {
                eprintln!("ERRORE nel Passo 2: {}. Saltando l'esempio.", e);
                continue;
            }
        };

        let labeled_json = match label_student_response(&client, &args, &student_prompt, &student_response, &system_prompt_used).await {
            Ok(json_str) => {
                // Clean up potential markdown code fences
                json_str.trim().trim_start_matches("```json").trim_end_matches("```").trim().to_string()
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
