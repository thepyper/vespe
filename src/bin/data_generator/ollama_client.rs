use reqwest::Client;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use tracing::debug;
use tokio_stream::StreamExt;
use serde_json::Value;
use std::io::{self, Write};

#[derive(Debug, Serialize)]
pub struct OllamaGenerateRequest<'a> {
    pub model: &'a str,
    pub prompt: &'a str,
    pub system: Option<&'a str>,
    pub stream: bool,
    pub options: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct OllamaGenerateResponse {
    pub model: Option<String>,
    pub created_at: Option<String>,
    pub response: Option<String>,
    pub done: bool,
    pub context: Option<Vec<i32>>,
    pub total_duration: Option<u64>,
    pub load_duration: Option<u64>,
    pub prompt_eval_count: Option<u32>,
    pub prompt_eval_duration: Option<u64>,
    pub eval_count: Option<u32>,
    pub eval_duration: Option<u64>,
}

pub async fn query_ollama(
    client: &Client,
    ollama_url: &str,
    model: &str,
    prompt: &str,
    system: Option<&str>,
) -> Result<String> {
    debug!("Ollama Request: model={}, system={:?}, prompt={}", model, system, prompt);
    let request_payload = OllamaGenerateRequest {
        model,
        prompt,
        system,
        stream: true,
        options: Some(serde_json::json!({
            "temperature": 0.0,
            "top_p": 0.0,
        })),
    };

    let mut full_response_text = String::new();

    let response = client
        .post(format!("{}/api/generate", ollama_url))
        .header("Accept", "application/x-ndjson")
        .json(&request_payload)
        .send()
        .await?;

    let mut stream = response.bytes_stream();
    let mut buffer = Vec::new();

    while let Some(chunk) = stream.next().await {
        let bytes = chunk?;
        buffer.extend_from_slice(&bytes);

        while let Some(newline_pos) = buffer.iter().position(|&b| b == b'\n') {
            let line_bytes = buffer.drain(..=newline_pos).collect::<Vec<u8>>();
            let line = String::from_utf8(line_bytes)?;

            if line.trim().is_empty() { continue; }

            let json_value: Value = serde_json::from_str(&line)?;
            if let Some(response_text) = json_value["response"].as_str() {
                print!("{}", response_text);
                io::stdout().flush()?;
                full_response_text.push_str(response_text);
            }
            if json_value["done"].as_bool().unwrap_or(false) { break; }
        }
    }
    println!(); // Add a newline after the streaming output

    debug!("Ollama Full Response: {}", full_response_text);
    Ok(full_response_text.trim().to_string())
}
