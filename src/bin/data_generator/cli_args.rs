use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    #[arg(long, default_value = "gpt-oss:20b")]
    pub big_model: String,
    #[arg(long, default_value = "gemma3:1b")]
    pub small_model: String,
    #[arg(long, default_value = "http://localhost:11434")]
    pub ollama_url: String,
    #[arg(long, default_value_t = 10)]
    pub num_examples: u32,
    #[arg(long, default_value = "buzz/training/generated_examples_rust")]
    pub output_dir: PathBuf,
    #[arg(long, value_parser = ["mcp", "json", "xml"], default_value = "mcp")]
    pub tool_format: String,
}
