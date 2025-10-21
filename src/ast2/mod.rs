mod types;
mod error;
mod parse;

pub use types::{Position, Range, Text, CommandKind, Parameters, Argument, Arguments, Tag, AnchorKind, Anchor, Content, Document};
pub use error::{Ast2Error, Result};
pub use parse::parse_document;

use parse::*;


