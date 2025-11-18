use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use uuid::{uuid, Uuid};

use super::range::Range;

/// A single positional argument, typically a string literal or a "nude" string.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Argument {
    /// The parsed value of the argument.
    pub value: String,
    /// The location of the argument in the source document.
    pub range: Range,
}

/// A collection of positional arguments associated with a `Tag` or `Anchor`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Arguments {
    /// The vector of parsed arguments.
    pub arguments: Vec<Argument>,
    /// The location of the arguments block in the source document.
    pub range: Range,
}

impl Arguments {
    /// Creates a new, empty set of arguments.
    pub fn new() -> Self {
        Arguments {
            arguments: Vec::new(),
            range: Range::null(),
        }
    }
    pub fn update(mut self, other: &Arguments) -> Self {
        if !other.arguments.is_empty() {
            self.arguments = other.arguments.clone();
        }
        self
    }
}

impl ToString for Arguments {
    fn to_string(&self) -> String {
        self.arguments
            .iter()
            .map(|x| x.value.clone())
            .collect::<Vec<String>>()
            .join(",")
    }
}

