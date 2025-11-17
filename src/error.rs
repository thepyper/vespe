use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Context name is required unless --today is specified.")]
    ContextNameRequired,
    #[error("Failed to read from stdin: {0}")]
    StdinReadError(#[from] std::io::Error),
}
