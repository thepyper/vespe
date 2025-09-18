use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use chrono::Local;

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

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Message {
    role: String,
    content: String,
}

// --- MCP Response Parsing Structures ---

#[derive(Deserialize, Debug)]
#[serde(tag = "method", content = "params")]
enum MCPResponse {
    #[serde(rename = "tools/call")]
    ToolCall(ToolCallParams),
    #[serde(rename = "response")]
    RegularResponse(RegularResponseParams),
    #[serde(rename = "thinking")]
    Thinking(ThinkingParams),
}

#[derive(Deserialize, Debug)]
struct ToolCallParams {
    name: String,
    arguments: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize, Debug)]
struct RegularResponseParams {
    content: String,
}

#[derive(Deserialize, Debug)]
struct ThinkingParams {
    thoughts: String,
}

// --- Tool Implementations ---

fn execute_echo(text: &str) -> String {
    format!("Echo: {}", text)
}

fn execute_read_file(path: &str, encoding: Option<&str>) -> Result<String, String> {
    let file_content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file '{}': {}", path, e))?;
    // For simplicity, we ignore encoding for now and assume UTF-8
    Ok(format!("File content of '{}':\n{}", path, file_content))
}

fn execute_write_file_faked(path: &str, content: &str, mode: &str) -> String {
    format!("Faking write to '{}' with content: '{}' in mode: '{}'", path, content, mode)
}

// --- Logging Function ---

fn log_interaction(log_file: &Option<String>, message: &str) -> Result<(), Box<dyn std::error::Error>> {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let log_entry = format!("[{}] {}\n", timestamp, message);
    print!("{}", log_entry);

    if let Some(path) = log_file {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        file.write_all(log_entry.as_bytes())?;
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Read system prompt
    let system_prompt_content = fs::read_to_string(&args.system_prompt_path)
        .map_err(|e| format!("Failed to read system prompt from {}: {}", args.system_prompt_path, e))?;

    let mut messages = vec![
        Message { role: "system".to_string(), content: system_prompt_content },
        Message { role: "user".to_string(), content: args.query.clone() },
    ];

    let client = reqwest::blocking::Client::new();
    let ollama_url = args.ollama_url.clone();

    loop {
        let ollama_request = OllamaRequest {
            model: args.model.clone(),
            messages: messages.clone(),
            stream: false,
        };

        let request_json = serde_json::to_string_pretty(&ollama_request)?;
        log_interaction(&args.log_file, &format!("Sending request:\n{}", request_json))?;

        let response = client.post(format!("{}/api/chat", ollama_url))
            .json(&ollama_request)
            .send()?;

        let response_text = response.text()?;
        log_interaction(&args.log_file, &format!("Received raw response:\n{}", response_text))?;

        let mcp_response: MCPResponse = match serde_json::from_str(&response_text) {
            Ok(resp) => resp,
            Err(e) => {
                log_interaction(&args.log_file, &format!("ERROR: Failed to parse MCP response: {}\nRaw response: {}", e, response_text))?;
                return Err(format!("Failed to parse MCP response: {}", e).into());
            }
        };

        log_interaction(&args.log_file, &format!("Parsed MCP response: {:#?}", mcp_response))?;

        match mcp_response {
            MCPResponse::ToolCall(params) => {
                let tool_output = match params.name.as_str() {
                    "echo" => {
                        let text = params.arguments.get("text")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| "Missing 'text' argument for echo tool".to_string())?;
                        execute_echo(text)
                    },
                    "read_file" => {
                        let path = params.arguments.get("path")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| "Missing 'path' argument for read_file tool".to_string())?;
                        let encoding = params.arguments.get("encoding").and_then(|v| v.as_str());
                        execute_read_file(path, encoding)?
                    },
                    "write_file" => {
                        let path = params.arguments.get("path")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| "Missing 'path' argument for write_file tool".to_string())?;
                        let content = params.arguments.get("content")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| "Missing 'content' argument for write_file tool".to_string())?;
                        let mode = params.arguments.get("mode")
                            .and_then(|v| v.as_str())
                            .unwrap_or("overwrite"); // Default mode
                        execute_write_file_faked(path, content, mode)
                    },
                    _ => return Err(format!("Unknown tool: {}", params.name).into()),
                };
                log_interaction(&args.log_file, &format!("Tool output: {}", tool_output))?;
                messages.push(Message { role: "user".to_string(), content: format!("TOOL_OUTPUT: {}", tool_output) });
            },
            MCPResponse::RegularResponse(params) => {
                log_interaction(&args.log_file, &format!("Model Response: {}", params.content))?;
                break;
            },
            MCPResponse::Thinking(params) => {
                log_interaction(&args.log_file, &format!("Model Thinking: {}", params.thoughts))?;
                // Continue loop, as thinking usually precedes a tool call or final response
            },
        }
    }

    Ok(())
}
