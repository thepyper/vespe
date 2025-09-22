use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, serde::Serialize)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    #[arg(long, default_value = "llama3.1:8b")]
    pub narrator_model: String,
    #[arg(long, default_value = "gpt-oss:20b")]
    pub marker_model: String,
    #[arg(long, default_value = "gemma3:1b")]
    pub hero_model: String,
    #[arg(long, default_value = "http://localhost:11434")]
    pub ollama_url: String,
    #[arg(long, default_value_t = 10)]
    pub num_examples: u32,
    #[arg(long, default_value = "buzz/training/generated_examples_rust")]
    pub output_dir: PathBuf,
    #[arg(long, value_parser = ["mcp", "json", "xml"], default_value = "mcp")]
    pub tool_format: String,
    #[arg(long, value_parser = ["json"], default_value = "json")]
    pub policy: String,
    #[arg(long)]
    pub tool_name: Option<String>,
    #[arg(long)]
    pub use_case: Option<String>,
    #[arg(long)]
    pub complexity: Option<String>,
    #[arg(long)]
    pub user_style: Option<String>,
    #[arg(long)]
    pub context_length: Option<String>,
}
