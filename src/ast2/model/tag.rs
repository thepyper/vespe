use serde::{Deserialize, Serialize};

use super::arguments::Arguments;
use super::command_kind::CommandKind;
use super::parameters::Parameters;
use super::range::Range;

/// Represents a command tag, starting with `@`.
///
/// Example: `@include 'path/to/file.ctx'`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tag {
    /// The command to be executed.
    pub command: CommandKind,
    /// Key-value parameters for the command.
    pub parameters: Parameters,
    /// Positional arguments for the command.
    pub arguments: Arguments,
    /// The location of the tag in the source document.
    pub range: Range,
}

impl Tag {
    pub fn integrate(mut self, other: &Parameters) -> Self {
        self.parameters = self.parameters.integrate(other);
        self
    }
}

impl ToString for Tag {
    fn to_string(&self) -> String {
        format!(
            "@{} {} {}",
            self.command.to_string(),
            self.parameters.to_string(),
            self.arguments.to_string(),
        )
    }
}
