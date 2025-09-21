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

pub const TOOLS_DEFINITION: &[Tool] = &[
    Tool {
        name: "read_file",
        description: "Legge e restituisce il contenuto di un file specificato.",
        parameters: &[
            ToolParameter {
                name: "absolute_path",
                param_type: "string",
                description: "Il percorso assoluto del file da leggere.",
            },
        ],
    },
    Tool {
        name: "write_file",
        description: "Scrive del contenuto in un file specificato, sovrascrivendolo se esiste.",
        parameters: &[
            ToolParameter {
                name: "file_path",
                param_type: "string",
                description: "Il percorso assoluto del file in cui scrivere.",
            },
            ToolParameter {
                name: "content",
                param_type: "string",
                description: "Il contenuto da scrivere nel file.",
            },
        ],
    },
];
