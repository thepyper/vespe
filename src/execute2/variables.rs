use serde::{Deserialize, Serialize};

use crate::ast2::{JsonPlusEntity, Parameters};

use std::str::FromStr;

/// Holds variables and configuration settings available during the execution of a context.
///
/// This struct can be extended to include more dynamic settings, such as
/// model choices, timeouts, or other parameters loaded from configuration files.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Variables {
    /// The command-line string used to invoke the external model provider (e.g., an LLM agent).
    pub provider: String,
    /// The system prompt string used as prelude to context
    pub system: Option<String>,
    /// The output redirection for other document modes
    pub output: Option<String>,
}

impl Variables {
    /// Creates a new `Variables` instance with default settings.
    pub fn new() -> Self {
        Variables {
            // TODO: This should be loaded from a project or user configuration file.
            provider: "gemini -y -m gemini-2.5-flash".to_string(),
            system: None,
            output: None,
        }
    }    
}
