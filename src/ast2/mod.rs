//! ## Abstract Syntax Tree (AST) Module
//!
//! This module is responsible for defining and parsing the template language
//! used by the application. It provides the necessary structures to represent
//! the source document as a typed Abstract Syntax Tree (AST), a dedicated parser,
//! and a specific error type for parsing failures.
//!
//! ### Key Components:
//!
//! - **`types.rs`**: Contains the Rust struct and enum definitions that form the AST,
//!   such as `Document`, `Content`, `Tag`, and `Anchor`.
//! - **`parse.rs`**: Implements the hand-written parser that consumes the source text
//!   and produces a `Document` AST.
//! - **`error.rs`**: Defines the `Ast2Error` enum, which provides detailed, positioned
//!   error information for any parsing failures.

mod error;
mod parse;
mod types;

pub use error::{Ast2Error, Result};
pub use parser::document::parse_document;
pub use types::{
    Anchor, AnchorKind, Argument, Arguments, CommandKind, Content, Document, JsonPlusEntity,
    JsonPlusObject, Parameters, Position, Range, Tag, Text,
};
