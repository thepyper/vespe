use crate::ast2::{Parameters};

use std::str::FromStr;

#[derive(Clone, Debug)]
pub enum OutputMode {
    /// Output into current document, default mode
    Here,   
    /// Append to a different given document
    Append,
    /// Overwrite to a different given document
    Overwrite,
}

impl FromStr for OutputMode {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "here" => Ok(OutputMode::Here),
            "append" => Ok(OutputMode::Append),
            "overwrite" => Ok(OutputMode::Overwrite),
            _ => Err(()),
        }
    }
}

/// Holds variables and configuration settings available during the execution of a context.
///
/// This struct can be extended to include more dynamic settings, such as
/// model choices, timeouts, or other parameters loaded from configuration files.
#[derive(Clone, Debug)]
pub struct Variables {
    /// The command-line string used to invoke the external model provider (e.g., an LLM agent).
    pub provider: String,
    /// The output mode for next queries
    pub output_mode: OutputMode,
    /// The output redirection for other document modes
    pub output: String,
}

impl Variables {
    /// Creates a new `Variables` instance with default settings.
    pub fn new() -> Self {
        Variables {
            // TODO: This should be loaded from a project or user configuration file.
            provider: "gemini -p -y -m gemini-2.5-flash".to_string(),
            output_mode: OutputMode::Here,
            output: String::new(),
        }
    }
    /// Create a new 'Variables' instance from an existing one taking values from Parameters
    pub fn update(&self, parameters: &Parameters) -> Self {
        let mut variables = self.clone();
        if let Some(x) = parameters.parameters.get("provider") {
            variables.provider = x.as_str().unwrap_or("internal-error-variables-rs").to_string();
        }
        if let Some(x) = parameters.parameters.get("output_mode") {
            variables.output_mode = x.to_string().parse::<OutputMode>().unwrap_or(OutputMode::Here);
        }
        if let Some(x) = parameters.parameters.get("output") {
            variables.output = x.as_str().unwrap_or("internal-error-variables-rs").to_string();
        }
        variables
    }
}
