use std::fs::File;
use std::io::Write;
use chrono::Local;
use crate::llm::models::ChatMessage;

pub struct Logger {
    file: File,
}

impl Logger {
    pub fn new(file_path: &str) -> Self {
        let file = File::create(file_path).expect("Failed to create log file");
        Self { file }
    }

    fn log(&mut self, message: &str) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        writeln!(self.file, "[{}] {}", timestamp, message).expect("Failed to write to log file");
    }

    pub fn log_llm_query(&mut self, messages: &Vec<ChatMessage>) {
        let query = serde_json::to_string_pretty(messages).unwrap_or_else(|_| "Failed to serialize query".to_string());
        self.log(&format!("LLM Query:\n{}", query));
    }

    pub fn log_llm_response(&mut self, response: &str) {
        self.log(&format!("LLM Response:\n{}", response));
    }

    pub fn log_tool_call(&mut self, tool_name: &str, args: &serde_json::Value) {
        let args_str = serde_json::to_string_pretty(args).unwrap_or_else(|_| "Failed to serialize args".to_string());
        self.log(&format!("Tool Call: {}({})
", tool_name, args_str));
    }

    pub fn log_tool_return(&mut self, tool_name: &str, output: &serde_json::Value) {
        let output_str = serde_json::to_string_pretty(output).unwrap_or_else(|_| "Failed to serialize output".to_string());
        self.log(&format!("Tool Return [{}]:\n{}\n", tool_name, output_str));
    }
}
