pub mod types;
pub mod error;
pub mod parser;
pub mod parse_primitives;
pub mod parse_elements;
pub mod parse_structures;

pub use types::*;
pub use error::{Ast2Error, Result};
pub use parser::Parser;
pub use parse_primitives::{
    _try_parse_identifier,
    _try_parse_uuid,
    _try_parse_nude_integer,
    _try_parse_nude_float,
    _try_parse_nude_bool,
    _try_parse_nude_string,
    _try_parse_enclosed_string,
};
pub use parse_elements::{
    _try_parse_command_kind,
    _try_parse_anchor_kind,
    _try_parse_parameters,
    _try_parse_parameter,
    _try_parse_arguments,
    _try_parse_argument,
    _try_parse_value,
    _try_parse_enclosed_value,
    _try_parse_nude_value,
};
pub use parse_structures::{
    parse_document,
    parse_content,
    _try_parse_tag,
    _try_parse_anchor,
    _try_parse_text,
};

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