pub mod file;
pub mod git;
pub mod path;
pub mod task;

use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error(transparent)]
    File(#[from] file::Error),
    #[error(transparent)]
    Git(#[from] git::Error),
    #[error(transparent)]
    Path(#[from] path::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
