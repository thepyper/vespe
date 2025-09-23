#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ParserSource {
    Json(JsonMatchMode),
    Xml(XmlMatchMode),
    // Add other parsers here as they are implemented
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum JsonMatchMode {
    FencedCodeBlock, // Match inside a ```json ... ``` block
    RawObject,       // Match of a raw JSON object { ... }
    RawArray,        // Match of a raw JSON array [ ... ]
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum XmlMatchMode {
    FencedCodeBlock,
    ToolCodeBlock, // Match inside a <tool_code> ... </tool_code> tag
}
