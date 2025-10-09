use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Agent(#[from] crate::agent::AgentError),
    // Placeholder for module-specific errors
}
