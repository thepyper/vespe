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
    mod test_parser_advance;
    mod test_parser_consume;
    mod test_parse_identifier;
    mod test_parse_nude_values;
    mod test_parse_enclosed_values;
    mod test_parse_argument;
    mod test_parse_arguments;
    mod test_parse_parameters;
    mod test_parse_kinds;
    mod test_parse_uuid;
    mod test_parse_tag;
    mod test_parse_anchor;
    mod test_parse_text;
    mod test_parse_document;
}