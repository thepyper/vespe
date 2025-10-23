/// Holds variables and configuration settings available during the execution of a context.
///
/// This struct can be extended to include more dynamic settings, such as
/// model choices, timeouts, or other parameters loaded from configuration files.
#[derive(Clone)]
pub struct Variables {
    /// The command-line string used to invoke the external model provider (e.g., an LLM agent).
    pub provider: String,
}

impl Variables {
    /// Creates a new `Variables` instance with default settings.
    pub fn new() -> Self {
        Variables {
            // TODO: This should be loaded from a project or user configuration file.
            provider: "gemini -p -y -m gemini-2.5-flash".to_string(),
        }
    }
}
