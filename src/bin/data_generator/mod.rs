pub mod cli_args;
pub mod ollama_client;
pub mod tool_definitions;
pub mod prompt_templates;
pub mod pipeline;

pub use cli_args::CliArgs;
pub use ollama_client::query_ollama;
pub use tool_definitions::TOOLS_DEFINITION;
pub use prompt_templates::register_all_templates;
pub use prompt_templates::NORMATIVE_SYSTEM_PROMPT;
pub use pipeline::{generate_student_prompt, build_tool_spec, get_student_response, label_student_response, save_labeled_example};
