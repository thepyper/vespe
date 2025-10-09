use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Agent(#[from] crate::agent::AgentError),
    #[error(transparent)]
    Editor(#[from] crate::editor::EditorError),
    #[error(transparent)]
    Project(#[from] crate::project::ProjectError),
    // Placeholder for module-specific errors
}
