use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the Ollama model to use (e.g., llama3)
    #[arg(short, long)]
    model: String,

    /// User's query to send to the model
    #[arg(short, long)]
    query: String,

    /// URL of the Ollama instance
    #[arg(long, default_value = "http://localhost:11434")]
    ollama_url: String,

    /// Path to the system prompt file
    #[arg(long, default_value = "doc/normative_mcp_prompt_1.txt")]
    system_prompt_path: String,

    /// Optional path for a log file to save raw interactions
    #[arg(long)]
    log_file: Option<String>,
}

#[derive(Serialize, Debug)]
struct OllamaRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize, Debug)]
struct OllamaResponse {
    message: Message,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // 1. Read system prompt
    let system_prompt = fs::read_to_string(&args.system_prompt_path)
        .map_err(|e| format!("Failed to read system prompt from {}: {}", args.system_prompt_path, e))?;

    // 2. Prepare messages
    let messages = vec![
        Message { role: "system".to_string(), content: system_prompt },
        Message { role: "user".to_string(), content: args.query },
    ];

    // 3. Construct request
    let ollama_request = OllamaRequest {
        model: args.model.clone(),
        messages,
        stream: false,
    };

    let request_json = serde_json::to_string_pretty(&ollama_request)?;

    // 4. Log raw request
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let log_entry_request = format!("[{}] Sending request:\n{}\n", timestamp, request_json);
    println!("{}", log_entry_request);
    if let Some(log_file_path) = &args.log_file {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file_path)?;
        file.write_all(log_entry_request.as_bytes())?;
    }

    // 5. Send request to Ollama
    let client = reqwest::blocking::Client::new();
    let response = client.post(format!("{}/api/chat", args.ollama_url))
        .json(&ollama_request)
        .send()?;

    let response_text = response.text()?;

    // 6. Log raw response
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let log_entry_response = format!("[{}] Received response:\n{}\n", timestamp, response_text);
    println!("{}", log_entry_response);
    if let Some(log_file_path) = &args.log_file {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file_path)?;
        file.write_all(log_entry_response.as_bytes())?;
    }

    // 7. Parse and print model's message
    let ollama_response: OllamaResponse = serde_json::from_str(&response_text)?;
    println!("\nModel's response: {}", ollama_response.message.content);

    Ok(())
}