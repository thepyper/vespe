#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParserSource {
    Json(JsonMatchMode),
    Xml(XmlMatchMode),
    // Add other parsers here as they are implemented
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum JsonMatchMode {
    FencedCodeBlock, // Match inside a ```json ... ``` block
    RawObject,       // Match of a raw JSON object { ... }
    RawArray,        // Match of a raw JSON array [ ... ]
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum XmlMatchMode {
    ToolCallTag, // Match inside a <tool_call> ... </tool_call> tag
}