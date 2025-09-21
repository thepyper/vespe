use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct Tool {
    pub name: &'static str,
    pub description: &'static str,
    pub parameters: &'static [ToolParameter],
}

#[derive(Debug, Serialize, Clone)]
pub struct ToolParameter {
    pub name: &'static str,
    #[serde(rename = "type")]
    pub param_type: &'static str,
    pub description: &'static str,
}

#[derive(Debug, Serialize)]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
    pub description: String,
    pub required: bool,
}

#[derive(Debug, Serialize)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub parameters: Vec<Parameter>,
}

impl Tool {
    pub fn to_tool_spec(&self) -> ToolSpec {
        let parameters: Vec<Parameter> = self.parameters.iter().map(|p| {
            Parameter {
                name: p.name.to_string(),
                param_type: p.param_type.to_string(),
                description: p.description.to_string(),
                // For simplicity, assuming all parameters are required for now.
                // This can be extended if the ToolParameter struct gets a 'required' field.
                required: true,
            }
        }).collect();

        ToolSpec {
            name: self.name.to_string(),
            description: self.description.to_string(),
            parameters,
        }
    }
}

pub const TOOLS_DEFINITION: &[Tool] = &[
    Tool {
        name: "read_file",
        description: "Reads and returns the content of a specified file.",
        parameters: &[
            ToolParameter {
                name: "absolute_path",
                param_type: "string",
                description: "The absolute path to the file to read.",
            },
        ],
    },
    Tool {
        name: "write_file",
        description: "Writes content to a specified file, overwriting it if it exists.",
        parameters: &[
            ToolParameter {
                name: "file_path",
                param_type: "string",
                description: "The absolute path to the file to write to.",
            },
            ToolParameter {
                name: "content",
                param_type: "string",
                description: "The content to write to the file.",
            },
        ],
    },
];
