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
mod parser;
mod model;

pub use model::anchor::{Anchor, AnchorKind};
pub use model::arguments::{Argument, Arguments};
pub use model::command_kind::{CommandKind};
pub use model::content::Content;
pub use model::document::Document;
pub use model::json_plus::{JsonPlusEntity, JsonPlusObject};
pub use model::parameters::Parameters;
pub use model::position::Position;
pub use model::range::Range;
pub use model::tag::Tag;
pub use model::text::Text;

pub use error::{Ast2Error, Result};
pub use parser::document::parse_document;
