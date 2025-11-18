use serde_json::json;
use std::collections::BTreeMap;
use std::str::Chars;
use std::str::FromStr;
use uuid::Uuid;

use super::{
    Anchor, AnchorKind, Argument, Arguments, Ast2Error, CommandKind, Content, Document,
    JsonPlusEntity, JsonPlusObject, Parameters, Position, Range, Result, Tag, Text,
};
use super::parser::Parser;















#[cfg(test)]
#[path = "./tests/test_parse_anchor.rs"]
mod test_parse_anchor;

#[cfg(test)]
#[path = "./tests/test_parse_argument.rs"]
mod test_parse_argument;

#[cfg(test)]
#[path = "./tests/test_parse_arguments.rs"]
mod test_parse_arguments;

#[cfg(test)]
#[path = "./tests/test_parse_document.rs"]
mod test_parse_document;

#[cfg(test)]
#[path = "./tests/test_parse_enclosed_values.rs"]
mod test_parse_enclosed_values;

#[cfg(test)]
#[path = "./tests/test_position_range.rs"]
mod test_position_range;

#[cfg(test)]
#[path = "./tests/utils.rs"]
mod utils;

#[cfg(test)]
#[path = "./tests/test_parser_advance.rs"]
mod test_parser_advance;

#[cfg(test)]
#[path = "./tests/test_parser_consume.rs"]
mod test_parser_consume;

#[cfg(test)]
#[path = "./tests/test_parse_uuid.rs"]
mod test_parse_uuid;

#[cfg(test)]
#[path = "./tests/test_parse_text.rs"]
mod test_parse_text;

#[cfg(test)]
#[path = "./tests/test_parse_identifier.rs"]
mod test_parse_identifier;

#[cfg(test)]
#[path = "./tests/test_parse_kinds.rs"]
mod test_parse_kinds;

#[cfg(test)]
#[path = "./tests/test_parse_nude_values.rs"]
mod test_parse_nude_values;

#[cfg(test)]
#[path = "./tests/test_parse_parameters.rs"]
mod test_parse_parameters;

#[cfg(test)]
#[path = "./tests/test_parse_tag.rs"]
mod test_parse_tag;

#[cfg(test)]
#[path = "./tests/test_parse_jsonplus.rs"]
mod test_parse_jsonplus;
