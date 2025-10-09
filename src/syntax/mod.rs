pub mod format;
pub mod parser;
pub mod types;
pub use format::*;
pub use parser::Error as ParserError;
pub use types::Error as TypesError;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Parser error: {0}")]
    ParserError(#[from] ParserError),
    #[error("Types error: {0}")]
    TypesError(#[from] TypesError),
}

pub type Result<T> = std::result::Result<T, Error>;
