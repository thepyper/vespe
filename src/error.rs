use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),
    #[error("Git error: {0}")]
    Git(#[from] crate::git::GitError),
    #[error("Project error: {0}")]
    Project(#[from] crate::project::ProjectError),
    #[error("Config error: {0}")]
    Config(#[from] crate::config::ConfigError),
    #[error("Editor error: {0}")]
    Editor(#[from] crate::editor::EditorError),
    #[error("Execute error: {0}")]
    Execute(#[from] crate::execute::ExecuteError),
    #[error("Semantic error: {0}")]
    Semantic(#[from] crate::semantic::SemanticError),
    #[error("Syntax error: {0}")]
    Syntax(#[from] crate::syntax::SyntaxError),
    #[error("Agent error: {0}")]
    Agent(#[from] crate::agent::AgentError),
    #[error("Utils error: {0}")]
    Utils(#[from] crate::utils::UtilsError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serde JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("UUID error: {0}")]
    Uuid(#[from] uuid::Error),
    #[error("Git2 error: {0}")]
    Git2(#[from] git2::Error),
    #[error("Walkdir error: {0}")]
    Walkdir(#[from] walkdir::Error),
    #[error("Notify error: {0}")]
    Notify(#[from] notify::Error),
    #[error("Handlebars error: {0}")]
    Handlebars(#[from] handlebars::RenderError),
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
    #[error("Portable Pty error: {0}")]
    PortablePty(#[from] portable_pty::Error),
    #[error("String conversion error: {0}")]
    FromUtf8(#[from] std::string::FromUtf8Error),
    #[error("Path conversion error: {0}")]
    StripPrefix(#[from] std::path::StripPrefixError),
    #[error("Clap error: {0}")]
    Clap(#[from] clap::Error),
    #[error("Chrono Parse error: {0}")]
    ChronoParse(#[from] chrono::ParseError),
    #[error("Unknown error")]
    Unknown,
}
