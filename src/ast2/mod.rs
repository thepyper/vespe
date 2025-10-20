pub mod types;
pub mod error;
pub(crate) mod parser;
pub(crate) mod parse_primitives;
pub(crate) mod parse_elements;
pub(crate) mod parse_structures;

pub use types::*;
pub use error::{Ast2Error, Result};
pub use parse_structures::parse_document;

#[cfg(test)]
mod tests {
    mod utils;
    mod test_position_range;
    mod test_parse_document;
}