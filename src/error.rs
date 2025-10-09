use thiserror::Error;

use crate::syntax;
use crate::semantic;
use crate::semantic::context;
use crate::project;
use crate::git;
use crate::execute;
use crate::execute::states;
use crate::config;
use crate::editor;
use crate::editor::lockfile;
use crate::agent;
use crate::utils;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Syntax error: {0}")]
    Syntax(#[from] syntax::Error),
    #[error("Semantic error: {0}")]
    Semantic(#[from] semantic::Error),
    #[error("Semantic context error: {0}")]
    SemanticContext(#[from] context::Error),
    #[error("Project error: {0}")]
    Project(#[from] project::Error),
    #[error("Git error: {0}")]
    Git(#[from] git::Error),
    #[error("Execute error: {0}")]
    Execute(#[from] execute::Error),
    #[error("Execute states error: {0}")]
    ExecuteStates(#[from] states::Error),
    #[error("Config error: {0}")]
    Config(#[from] config::Error),
    #[error("Editor error: {0}")]
    Editor(#[from] editor::Error),
    #[error("Editor lockfile error: {0}")]
    EditorLockfile(#[from] lockfile::Error),
    #[error("Agent error: {0}")]
    Agent(#[from] agent::Error),
    #[error("Utils error: {0}")]
    Utils(#[from] utils::Error),
}
