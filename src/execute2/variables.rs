use serde::{Deserialize, Serialize};

use crate::ast2::Parameters;

use std::str::FromStr;

/// Holds variables and configuration settings available during the execution of a context.
///
/// This struct can be extended to include more dynamic settings, such as
/// model choices, timeouts, or other parameters loaded from configuration files.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Variables {
    /// The command-line string used to invoke the external model provider (e.g., an LLM agent).
    pub provider: String,
    /// The output redirection for other document modes
    pub output: Option<String>,
}

impl Variables {
    /// Creates a new `Variables` instance with default settings.
    pub fn new() -> Self {
        Variables {
            // TODO: This should be loaded from a project or user configuration file.
            provider: "gemini -p -y -m gemini-2.5-flash".to_string(),
            output: None,
        }
    }
    /// Create a new 'Variables' instance from an existing one taking values from Parameters
    pub fn update(&self, parameters: &Parameters) -> Self {
        let mut variables = self.clone();
        if let Some(x) = parameters.parameters.get("provider") {
            variables.provider = x
                .as_str()
                .unwrap_or("internal-error-variables-rs-1")
                .to_string();
        }
        if let Some(x) = parameters.parameters.get("output") {
            variables.output = Some(
                x.as_str()
                    .unwrap_or("internal-error-variables-rs-2")
                    .to_string(),
            );
        }
        variables
    }
}
