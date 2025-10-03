//! This module provides the Abstract Syntax Tree (AST) structures and parsing logic
//! for processing context files.
//!
//! It defines the core data structures like `Line`, `LineKind`, `AnchorData`,
//! and `Context`, along with functions to parse text into these structures.
//! A `Resolver` trait is also provided for resolving context and snippet paths.

pub mod types;
pub mod resolver;
pub mod parser;

#[cfg(test)]
pub mod test;

pub use types::{AnchorData, AnchorKind, Context, Line, LineKind, Parameters, Snippet};
pub use resolver::Resolver;
pub use parser::{parse_context, parse_snippet, parse_line};