use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    System(String),
    User(String),
    Assistant(Vec<AssistantContent>),
    Tool(ToolOutput),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssistantContent {
    Text(String),
    Thought(String),
    ToolCall(ToolCall),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ToolOutput {
    pub tool_name: String,
    pub output: Value,
}
