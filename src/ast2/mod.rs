mod error;
mod types;
mod parse;

pub use error::{Ast2Error, Result};
pub use parse::parse_document;
pub use types::{
    Anchor, AnchorKind, Argument, Arguments, CommandKind, Content, Document, Parameters, Position,
    Range, Tag, Text,
};

use parse::*;
